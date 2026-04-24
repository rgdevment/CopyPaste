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
              controller!.stream.listen(sink.success, onError: sink.error);
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
}
