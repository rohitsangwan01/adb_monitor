import 'package:flutter/material.dart';
import 'package:adb_monitor/adb_monitor.dart';

Future<void> main() async {
  await AdbMonitor.init();
  runApp(
    const MaterialApp(
      debugShowCheckedModeBanner: false,
      home: MyApp(),
    ),
  );
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  List<String> devices = [];

  @override
  void initState() {
    AdbMonitor.devices.listen((String device) {
      setState(() {
        devices.add(device);
      });
    });
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Adb Monitor'),
        elevation: 4,
      ),
      body: Column(
        children: [
          const SizedBox(height: 10),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              ElevatedButton(
                  onPressed: () {
                    AdbMonitor.start();
                  },
                  child: const Text('Start')),
              ElevatedButton(
                  onPressed: () {
                    AdbMonitor.stop();
                  },
                  child: const Text('Stop')),
            ],
          ),
          const Divider(),
          Expanded(
            child: ListView.builder(
              itemCount: devices.length,
              itemBuilder: (BuildContext context, int index) {
                return ListTile(
                  title: Text(devices[index]),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
