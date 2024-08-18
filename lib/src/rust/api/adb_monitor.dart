// This file is automatically generated, so please do not edit it.
// Generated by `flutter_rust_bridge`@ 2.3.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

// These functions are ignored because they are not marked as `pub`: `available_packet_length`, `binary_to_string`, `consume_okay`, `consume`, `fill_buffer_from`, `handle_packet`, `monitor`, `new`, `new`, `next_packet`, `on_new_device_connected`, `parse_connected_devices`, `parse_length`, `peek_mut`, `peek`, `read_from`, `read_packet`, `repair_adb_daemon`, `start_adb_daemon`, `stop_monitor`, `track_devices_on_stream`, `track_devices`
// These types are ignored because they are not used by any `pub` functions: `AdbMonitor`, `ByteBuffer`

/// Start Monitoring Adb Devices
Stream<String> initialize() =>
    RustLib.instance.api.crateApiAdbMonitorInitialize();

Future<void> startMonitor() =>
    RustLib.instance.api.crateApiAdbMonitorStartMonitor();

Future<void> stopMonitor() =>
    RustLib.instance.api.crateApiAdbMonitorStopMonitor();
