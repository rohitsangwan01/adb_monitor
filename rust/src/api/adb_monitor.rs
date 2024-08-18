// Copied from gnirehtet/relay-rust/src/adb_monitor.rs and modified
/*
 * Copyright (C) 2017 Genymobile
 * Copyright (C) 2019 Romain Vimont
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream};
use std::process;
use std::ptr;
use std::str;
use std::thread;
use std::time::Duration;

use crate::frb_generated::StreamSink;

static mut ADB_MONITOR: Option<AdbMonitor> = None;
static mut IS_MONITORING: bool = false;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// Start Monitoring Adb Devices
pub fn initialize(stream: StreamSink<String>) {
    unsafe {
        if !ADB_MONITOR.is_none() {
            println!("Already initialized")
        } else {
            ADB_MONITOR = Some(AdbMonitor::new(stream))
        }
    };
}

pub fn start_monitor() {
    unsafe {
        if IS_MONITORING {
            return;
        }
        IS_MONITORING = true;
        ADB_MONITOR.as_mut().unwrap().monitor();
    };
}

pub fn stop_monitor() {
    unsafe {
        if !IS_MONITORING {
            return;
        }
        ADB_MONITOR.as_mut().unwrap().stop_monitor();
        IS_MONITORING = false;
    };
}

// Adb Monitor
struct AdbMonitor {
    buf: ByteBuffer,
    connected_devices: Vec<String>,
    stream: StreamSink<String>,
    stop_monitor: bool,
}

impl AdbMonitor {
    const TRACK_DEVICES_REQUEST: &'static [u8] = b"0012host:track-devices";
    const BUFFER_SIZE: usize = 1024;
    const RETRY_DELAY_ADB_DAEMON_OK: u64 = 1000;
    const RETRY_DELAY_ADB_DAEMON_KO: u64 = 5000;

    fn on_new_device_connected(&mut self, serial: &str) {
        println!("Detected device {}", serial);
        match self.stream.add(serial.to_string()) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to notify dart {e}")
            }
        }
    }

    fn new(stream: StreamSink<String>) -> Self {
        Self {
            buf: ByteBuffer::new(Self::BUFFER_SIZE),
            connected_devices: Vec::new(),
            stream,
            stop_monitor: false,
        }
    }

    fn monitor(&mut self) {
        self.stop_monitor = false;
        loop {
            if self.stop_monitor {
                break;
            }
            if let Err(err) = self.track_devices() {
                println!("Failed to monitor adb devices: {}", err);
                Self::repair_adb_daemon();
            }
        }
    }

    fn stop_monitor(&mut self) {
        self.stop_monitor = true;
    }

    fn track_devices(&mut self) -> io::Result<()> {
        let abd_addr = SocketAddr::from(([127, 0, 0, 1], 5037));
        let mut stream = TcpStream::connect(abd_addr)?;
        self.track_devices_on_stream(&mut stream)
    }

    fn track_devices_on_stream(&mut self, stream: &mut TcpStream) -> io::Result<()> {
        stream.write_all(Self::TRACK_DEVICES_REQUEST)?;
        if self.consume_okay(stream)? {
            loop {
                if self.stop_monitor {
                    break;
                }
                let packet = self.next_packet(stream)?;
                self.handle_packet(packet.as_str());
            }
        }
        Ok(())
    }

    fn consume_okay(&mut self, stream: &mut TcpStream) -> io::Result<bool> {
        while self.buf.peek().len() < 4 {
            self.buf.read_from(stream)?;
        }
        let ok = b"OKAY" == &self.buf.peek()[0..4];
        self.buf.consume(4);
        Ok(ok)
    }

    fn read_packet(buf: &mut ByteBuffer) -> io::Result<Option<String>> {
        let packet_length = Self::available_packet_length(buf.peek())?;
        if let Some(len) = packet_length {
            let data = Self::binary_to_string(&buf.peek()[4..len])?;
            buf.consume(len);
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    fn next_packet(&mut self, stream: &mut TcpStream) -> io::Result<String> {
        loop {
            let packet = Self::read_packet(&mut self.buf)?;
            if let Some(packet) = packet {
                return Ok(packet);
            } else {
                self.fill_buffer_from(stream)?;
            }
        }
    }

    fn fill_buffer_from(&mut self, stream: &mut TcpStream) -> io::Result<()> {
        match self.buf.read_from(stream) {
            Ok(false) => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "ADB daemon closed the track-devices connection",
            )),
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn available_packet_length(input: &[u8]) -> io::Result<Option<usize>> {
        if input.len() < 4 {
            Ok(None)
        } else {
            // each packet contains 4 bytes representing the String length in hexa, followed by a
            // list of device information;
            // each line contains: the device serial, `\t', the state, '\n'
            // for example:
            // "00360123456789abcdef\tdevice\nfedcba9876543210\tunauthorized\n":
            //  - 0036 indicates that the data is 0x36 (54) bytes length
            //  - the device with serial 0123456789abcdef is connected
            //  - the device with serial fedcba9876543210 is unauthorized
            let len = Self::parse_length(&input[0..4])?;
            if len > Self::BUFFER_SIZE as u32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Packet size should not be that big: {}", len),
                ));
            }
            if input.len() - 4usize >= len as usize {
                Ok(Some(4usize + len as usize))
            } else {
                // not enough data
                Ok(None)
            }
        }
    }

    fn handle_packet(&mut self, packet: &str) {
        let current_connected_devices = self.parse_connected_devices(packet);
        for serial in &current_connected_devices {
            if !self.connected_devices.contains(serial) {
                self.on_new_device_connected(serial.as_str());
            }
        }
        self.connected_devices = current_connected_devices;
    }

    fn parse_connected_devices(&self, packet: &str) -> Vec<String> {
        packet
            .lines()
            .filter_map(|line| {
                let mut split = line.split_whitespace();
                if let Some(serial) = split.next() {
                    if let Some(state) = split.next() {
                        if state == "device" {
                            return Some(serial.to_string());
                        }
                    }
                }
                None
            })
            .collect()
    }

    fn parse_length(data: &[u8]) -> io::Result<u32> {
        assert!(data.len() == 4, "Invalid length field value");
        let hexa = str::from_utf8(data).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot read hexa length as UTF-8 ({})", err),
            )
        })?;
        u32::from_str_radix(hexa, 0x10).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Cannot parse hexa length ({})", err),
            )
        })
    }

    fn repair_adb_daemon() {
        if Self::start_adb_daemon() {
            thread::sleep(Duration::from_millis(Self::RETRY_DELAY_ADB_DAEMON_OK));
        } else {
            thread::sleep(Duration::from_millis(Self::RETRY_DELAY_ADB_DAEMON_KO));
        }
    }

    fn start_adb_daemon() -> bool {
        println!("Restarting adb daemon");
        match process::Command::new("adb")
            .args(&["start-server"])
            .status()
        {
            Ok(exit_status) => {
                if exit_status.success() {
                    true
                } else {
                    println!("Could not restart adb daemon (exited on error)");
                    false
                }
            }
            Err(err) => {
                println!("Could not restart adb daemon: {}", err);
                false
            }
        }
    }

    fn binary_to_string(data: &[u8]) -> io::Result<String> {
        let raw_content = data.to_vec();
        let content = String::from_utf8(raw_content);
        if let Ok(content) = content {
            Ok(content)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Track-devices string is not valid UTF-8",
            ))
        }
    }
}

// Byte Buffer
struct ByteBuffer {
    buf: Box<[u8]>,
    head: usize,
}

impl ByteBuffer {
    fn new(length: usize) -> Self {
        Self {
            buf: vec![0; length].into_boxed_slice(),
            head: 0,
        }
    }

    fn read_from<R: io::Read>(&mut self, source: &mut R) -> io::Result<bool> {
        let target_slice = &mut self.buf[self.head..];
        let r = source.read(target_slice)?;
        self.head += r;
        Ok(r > 0)
    }

    fn peek(&self) -> &[u8] {
        &self.buf[..self.head]
    }

    #[allow(unused)]
    fn peek_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.head]
    }

    fn consume(&mut self, length: usize) {
        assert!(self.head >= length);
        self.head -= length;
        if self.head > 0 {
            unsafe {
                let buf_ptr = self.buf.as_mut_ptr();
                ptr::copy(buf_ptr.add(length), buf_ptr, self.head);
            }
        }
    }
}
