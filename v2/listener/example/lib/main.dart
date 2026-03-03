import 'dart:async';

import 'package:flutter/material.dart';
import 'package:listener/listener.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  final _listener = WindowsClipboardListener();
  final _events = <String>[];
  StreamSubscription<ClipboardEvent>? _sub;

  @override
  void initState() {
    super.initState();
    _sub = _listener.onEvent.listen((event) {
      setState(() {
        _events.insert(
          0,
          '[${event.type.name}] ${event.text ?? event.files?.join(', ') ?? 'image'} (${event.source ?? '?'})',
        );
        if (_events.length > 50) _events.removeLast();
      });
    });
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('Clipboard Listener')),
        body: _events.isEmpty
            ? const Center(child: Text('Copy something...'))
            : ListView.builder(
                itemCount: _events.length,
                itemBuilder: (_, i) => ListTile(
                  dense: true,
                  title: Text(_events[i]),
                ),
              ),
      ),
    );
  }
}
