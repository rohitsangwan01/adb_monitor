library adb_monitor;

import 'rust/api/adb_monitor.dart' as rust;
import 'rust/frb_generated.dart' show RustLib;

class AdbMonitor {
  static Stream<String>? _devicesStream;

  static Future<void> init() async {
    await RustLib.init();
    _devicesStream = rust.initialize();
  }

  static Stream<String> get devices {
    _ensureInitialize();
    return _devicesStream!;
  }

  static void start() {
    _ensureInitialize();
    rust.startMonitor();
  }

  static void stop() {
    _ensureInitialize();
    rust.stopMonitor();
  }

  static void _ensureInitialize() {
    if (_devicesStream == null) {
      throw "Please initialize first";
    }
  }
}
