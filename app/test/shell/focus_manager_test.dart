import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/shell/focus_manager.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('copypaste/clipboard_writer');

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, (call) async {
          switch (call.method) {
            case 'captureFrontmostApp':
              return 'com.apple.finder';
            case 'activateAndPaste':
              return true;
            default:
              return null;
          }
        });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  group('WindowFocusManager – macOS', () {
    test('capturePreviousWindow calls captureFrontmostApp', () async {
      if (!Platform.isMacOS) return;

      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            if (call.method == 'captureFrontmostApp') return 'com.test.app';
            return null;
          });

      final manager = WindowFocusManager();
      await manager.capturePreviousWindow();

      expect(captured, isNotNull);
      expect(captured!.method, equals('captureFrontmostApp'));
    });

    test(
      'restoreAndPaste returns early when no bundle id was captured',
      () async {
        if (!Platform.isMacOS) return;

        bool activateCalled = false;
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'activateAndPaste') activateCalled = true;
              return true;
            });

        final manager = WindowFocusManager();
        // capturePreviousWindow NOT called → _previousBundleId is null
        await manager.restoreAndPaste(
          delayBeforeFocusMs: 0,
          maxFocusVerifyAttempts: 1,
          delayBeforePasteMs: 0,
        );

        expect(activateCalled, isFalse);
      },
    );

    test(
      'restoreAndPaste returns early when captureFrontmostApp returned null',
      () async {
        if (!Platform.isMacOS) return;

        bool activateCalled = false;
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'captureFrontmostApp') return null;
              if (call.method == 'activateAndPaste') activateCalled = true;
              return null;
            });

        final manager = WindowFocusManager();
        await manager.capturePreviousWindow();
        await manager.restoreAndPaste(
          delayBeforeFocusMs: 0,
          maxFocusVerifyAttempts: 1,
          delayBeforePasteMs: 0,
        );

        expect(activateCalled, isFalse);
      },
    );

    test(
      'restoreAndPaste calls activateAndPaste with correct arguments',
      () async {
        if (!Platform.isMacOS) return;

        final calls = <MethodCall>[];
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              calls.add(call);
              if (call.method == 'captureFrontmostApp') {
                return 'com.apple.safari';
              }
              if (call.method == 'activateAndPaste') return true;
              return null;
            });

        final manager = WindowFocusManager();
        await manager.capturePreviousWindow();
        await manager.restoreAndPaste(
          delayBeforeFocusMs: 0,
          maxFocusVerifyAttempts: 1,
          delayBeforePasteMs: 250,
        );

        final pasteCall = calls.firstWhere(
          (c) => c.method == 'activateAndPaste',
        );
        expect(pasteCall.arguments['bundleId'], equals('com.apple.safari'));
        expect(pasteCall.arguments['delayMs'], equals(250));
      },
    );

    test(
      'clear() resets bundle id so restoreAndPaste becomes a no-op',
      () async {
        if (!Platform.isMacOS) return;

        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'captureFrontmostApp') {
                return 'com.apple.finder';
              }
              return null;
            });

        final manager = WindowFocusManager();
        await manager.capturePreviousWindow();
        manager.clear();

        bool activateCalled = false;
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'activateAndPaste') activateCalled = true;
              return true;
            });

        await manager.restoreAndPaste(
          delayBeforeFocusMs: 0,
          maxFocusVerifyAttempts: 1,
          delayBeforePasteMs: 0,
        );

        expect(activateCalled, isFalse);
      },
    );

    test(
      'multiple capturePreviousWindow calls keep the last bundle id',
      () async {
        if (!Platform.isMacOS) return;

        int callCount = 0;
        final bundleIds = ['com.first.app', 'com.second.app'];
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'captureFrontmostApp') {
                return bundleIds[callCount++];
              }
              if (call.method == 'activateAndPaste') return true;
              return null;
            });

        final manager = WindowFocusManager();
        await manager.capturePreviousWindow(); // stores com.first.app
        await manager.capturePreviousWindow(); // stores com.second.app

        final calls = <MethodCall>[];
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              calls.add(call);
              if (call.method == 'activateAndPaste') return true;
              return null;
            });

        await manager.restoreAndPaste(
          delayBeforeFocusMs: 0,
          maxFocusVerifyAttempts: 1,
          delayBeforePasteMs: 0,
        );

        final pasteCall = calls.firstWhere(
          (c) => c.method == 'activateAndPaste',
        );
        expect(pasteCall.arguments['bundleId'], equals('com.second.app'));
      },
    );

    test(
      'restoreAndPaste propagates ACCESSIBILITY_DENIED PlatformException',
      () async {
        if (!Platform.isMacOS) return;

        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'captureFrontmostApp') {
                return 'com.apple.safari';
              }
              if (call.method == 'activateAndPaste') {
                throw PlatformException(
                  code: 'ACCESSIBILITY_DENIED',
                  message: 'Accessibility permission not granted',
                );
              }
              return null;
            });

        final manager = WindowFocusManager();
        await manager.capturePreviousWindow();

        expect(
          () => manager.restoreAndPaste(
            delayBeforeFocusMs: 0,
            maxFocusVerifyAttempts: 1,
            delayBeforePasteMs: 0,
          ),
          throwsA(
            isA<PlatformException>().having(
              (e) => e.code,
              'code',
              equals('ACCESSIBILITY_DENIED'),
            ),
          ),
        );
      },
    );
  });
}
