# Adb Monitor

Monitor Adb devices connection using Rust

## Getting Started

### Installation

1. Install Rust via [rustup](https://rustup.rs/)
2. Add `adb_monitor` to `pubspec.yaml`:

```yaml
dependencies:
  adb_monitor: <version>
```

### Platforms Setup

on `MacOS` set sandbox to `false` in `macos/Runner/*.entitlements`

```xml
<key>com.apple.security.app-sandbox</key>
<false/>
```

### Initialization

```dart
import 'package:adb_monitor/adb_monitor.dart';

void main() async {
  await AdbMonitor.init();
  runApp(MyApp());
}
```

### Usage

Listen to Adb Devices

```dart
AdbMonitor.devices.listen((String device) {
    // Handle devices
});
```

Start Monitoring

```dart
AdbMonitor.start();
```

Stop Monitoring

```dart
AdbMonitor.stop()
```

## Note

This package uses FFI with [flutter_rust_bridge](https://pub.dev/packages/flutter_rust_bridge) to call Rust code.

On Rust's side, the [autoadb](https://github.com/rom1v/autoadb) is used to detect adb devices.
