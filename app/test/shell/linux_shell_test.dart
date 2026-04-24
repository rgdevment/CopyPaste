import 'dart:async';

import 'package:copypaste/shell/linux_shell.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const eventChannelName = 'copypaste/linux_shell/events';
  StreamController<dynamic>? controller;

  Future<void> emit(Object event) async {
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    final data = const StandardMethodCodec().encodeSuccessEnvelope(event);
    await messenger.handlePlatformMessage(eventChannelName, data, (_) {});
  }

  setUp(() {
    controller = StreamController<dynamic>.broadcast();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockStreamHandler(
          const EventChannel(eventChannelName),
          MockStreamHandler.inline(
            onListen: (_, sink) {
              controller!.stream.listen(sink.success);
            },
            onCancel: (_) {},
          ),
        );
  });

  tearDown(() async {
    await LinuxShell.dispose();
    await controller?.close();
    controller = null;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockStreamHandler(const EventChannel(eventChannelName), null);
  });

  group('LinuxShell.awaitEvent', () {
    test('completes true when matching event arrives', () async {
      final future = LinuxShell.awaitEvent(
        'unmapped',
        timeout: const Duration(seconds: 1),
      );
      await Future<void>.delayed(const Duration(milliseconds: 20));
      await emit({'type': 'unmapped'});
      expect(await future, isTrue);
    });

    test('completes false on timeout when event never arrives', () async {
      final result = await LinuxShell.awaitEvent(
        'unmapped',
        timeout: const Duration(milliseconds: 50),
      );
      expect(result, isFalse);
    });

    test('ignores non-matching events and times out', () async {
      final future = LinuxShell.awaitEvent(
        'unmapped',
        timeout: const Duration(milliseconds: 80),
      );
      await Future<void>.delayed(const Duration(milliseconds: 10));
      await emit({'type': 'mapped'});
      await emit({'type': 'hotkey'});
      expect(await future, isFalse);
    });
  });

  group('LinuxShell.getCursorMonitor', () {
    const channel = MethodChannel('copypaste/linux_shell');

    tearDown(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, null);
    });

    test('parses Map response into CursorMonitorInfo', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method != 'getCursorMonitor') return null;
            return <String, Object?>{
              'cursorX': 800.0,
              'cursorY': 450.0,
              'x': 0.0,
              'y': 0.0,
              'width': 1920.0,
              'height': 1080.0,
              'scaleFactor': 2.0,
            };
          });
      final info = await LinuxShell.getCursorMonitor();
      expect(info, isNotNull);
      expect(info!.cursorX, equals(800.0));
      expect(info.width, equals(1920.0));
      expect(info.scaleFactor, equals(2.0));
    });

    test('returns null when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async => null);
      expect(await LinuxShell.getCursorMonitor(), isNull);
    });
  });

  group('LinuxShell.getInputFocus', () {
    const channel = MethodChannel('copypaste/linux_shell');

    tearDown(() {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, null);
    });

    test('parses Map response into InputFocusInfo', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method != 'getInputFocus') return null;
            return <String, Object?>{
              'ownsFocus': true,
              'focusWindow': 0xabc,
              'ownWindow': 0xabc,
            };
          });
      final info = await LinuxShell.getInputFocus();
      expect(info, isNotNull);
      expect(info!.ownsFocus, isTrue);
      expect(info.focusWindow, equals(0xabc));
    });

    test('returns null when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async => null);
      expect(await LinuxShell.getInputFocus(), isNull);
    });
  });
}
