import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/focus_manager.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('copypaste/clipboard_writer');

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, (call) async {
          switch (call.method) {
            case 'captureFrontmostApp':
              return 'org.gnome.Nautilus';
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

  group('WindowFocusManager – Linux', () {
    test('capturePreviousWindow calls captureFrontmostApp', () async {
      if (!Platform.isLinux) return;

      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            if (call.method == 'captureFrontmostApp') return 'org.gnome.gedit';
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
        if (!Platform.isLinux) return;

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
        if (!Platform.isLinux) return;

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
      'restoreAndPaste calls activateAndPaste with correct bundleId',
      () async {
        if (!Platform.isLinux) return;

        final calls = <MethodCall>[];
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              calls.add(call);
              if (call.method == 'captureFrontmostApp') {
                return 'org.gnome.gedit';
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
        expect(pasteCall.arguments['bundleId'], equals('org.gnome.gedit'));
        expect(pasteCall.arguments['delayMs'], equals(250));
      },
    );

    test('restoreAndPaste passes delayBeforePasteMs correctly', () async {
      if (!Platform.isLinux) return;

      final calls = <MethodCall>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            calls.add(call);
            if (call.method == 'captureFrontmostApp') {
              return 'org.kde.dolphin';
            }
            if (call.method == 'activateAndPaste') return true;
            return null;
          });

      final manager = WindowFocusManager();
      await manager.capturePreviousWindow();
      await manager.restoreAndPaste(
        delayBeforeFocusMs: 0,
        maxFocusVerifyAttempts: 1,
        delayBeforePasteMs: 100,
      );

      final pasteCall = calls.firstWhere((c) => c.method == 'activateAndPaste');
      expect(pasteCall.arguments['delayMs'], equals(100));
    });

    test(
      'clear() resets bundle id so restoreAndPaste becomes a no-op',
      () async {
        if (!Platform.isLinux) return;

        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'captureFrontmostApp') {
                return 'org.gnome.Nautilus';
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
        if (!Platform.isLinux) return;

        int callCount = 0;
        final bundleIds = ['org.first.app', 'org.second.app'];
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'captureFrontmostApp') {
                return bundleIds[callCount++];
              }
              if (call.method == 'activateAndPaste') return true;
              return null;
            });

        final manager = WindowFocusManager();
        await manager.capturePreviousWindow(); // stores org.first.app
        await manager.capturePreviousWindow(); // stores org.second.app

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
        expect(pasteCall.arguments['bundleId'], equals('org.second.app'));
      },
    );

    test(
      'restoreAndPaste propagates ACCESSIBILITY_DENIED PlatformException',
      () async {
        if (!Platform.isLinux) return;

        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              if (call.method == 'captureFrontmostApp') {
                return 'org.gnome.Nautilus';
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

    test('restoreAndPaste clears bundle id after completing paste', () async {
      if (!Platform.isLinux) return;

      final calls = <MethodCall>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            calls.add(call);
            if (call.method == 'captureFrontmostApp') {
              return 'org.gnome.gedit';
            }
            if (call.method == 'activateAndPaste') return true;
            return null;
          });

      final manager = WindowFocusManager();
      await manager.capturePreviousWindow();
      await manager.restoreAndPaste(
        delayBeforeFocusMs: 0,
        maxFocusVerifyAttempts: 1,
        delayBeforePasteMs: 0,
      );

      // Second restoreAndPaste must be a no-op (bundle id was cleared)
      final countBefore = calls.length;
      await manager.restoreAndPaste(
        delayBeforeFocusMs: 0,
        maxFocusVerifyAttempts: 1,
        delayBeforePasteMs: 0,
      );
      expect(
        calls.where((c) => c.method == 'activateAndPaste').length,
        equals(
          countBefore -
              calls.where((c) => c.method != 'activateAndPaste').length,
        ),
        reason: 'activateAndPaste should not be called again after clear',
      );
    });

    test(
      'clear() is idempotent — safe to call without prior capture',
      () async {
        if (!Platform.isLinux) return;

        final manager = WindowFocusManager();
        manager.clear(); // no prior capture
        manager.clear(); // double clear

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
  });
}
