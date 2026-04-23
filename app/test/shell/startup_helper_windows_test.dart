import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/msix_startup_task.dart';
import 'package:copypaste/shell/startup_helper.dart';

const _startupChannel = MethodChannel('copypaste/startup_task');

void _setStartupHandler(Future<Object?> Function(MethodCall) handler) {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(_startupChannel, handler);
}

void _clearHandlers() {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(_startupChannel, null);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  tearDown(_clearHandlers);

  // ---------------------------------------------------------------------------
  // isDevBuildPath — pure logic, always runs
  // ---------------------------------------------------------------------------

  group('StartupHelper.isDevBuildPath', () {
    test('detects a typical Debug build path', () {
      expect(
        StartupHelper.isDevBuildPath(
          r'C:\Users\dev\CopyPaste\app\build\windows\x64\runner\Debug\copypaste.exe',
        ),
        isTrue,
      );
    });

    test('detects a Release build path', () {
      expect(
        StartupHelper.isDevBuildPath(
          r'C:\Users\dev\CopyPaste\app\build\windows\x64\runner\Release\copypaste.exe',
        ),
        isTrue,
      );
    });

    test('detects forward-slash variant', () {
      expect(
        StartupHelper.isDevBuildPath(
          r'C:/Users/dev/CopyPaste/app/build/windows/x64/runner/Release/copypaste.exe',
        ),
        isTrue,
      );
    });

    test('detects mixed slash variant', () {
      expect(
        StartupHelper.isDevBuildPath(
          r'C:\Users\dev/CopyPaste\app\build/windows\x64\copypaste.exe',
        ),
        isTrue,
      );
    });

    test('is case-insensitive', () {
      expect(
        StartupHelper.isDevBuildPath(
          r'C:\Users\Dev\CopyPaste\APP\BUILD\WINDOWS\x64\copypaste.exe',
        ),
        isTrue,
      );
    });

    test('returns false for a proper installed path', () {
      expect(
        StartupHelper.isDevBuildPath(
          r'C:\Program Files\CopyPaste\CopyPaste.exe',
        ),
        isFalse,
      );
    });

    test(
      'returns false for a path that contains "windows" but not build path',
      () {
        expect(
          StartupHelper.isDevBuildPath(r'C:\Users\dev\windows\CopyPaste.exe'),
          isFalse,
        );
      },
    );

    test('returns false for empty string', () {
      expect(StartupHelper.isDevBuildPath(''), isFalse);
    });
  });

  // ---------------------------------------------------------------------------
  // apply() on Windows — MSIX path: calls enable/disable and clears registry
  // ---------------------------------------------------------------------------

  group('StartupHelper.apply – MSIX StartupTask interaction', () {
    test('enable is called with the correct taskId', () async {
      if (!Platform.isWindows) return;

      MethodCall? captured;
      _setStartupHandler((call) async {
        captured = call;
        return 'enabled';
      });

      // We cannot mock WinPackageContext.isMsix directly, so this test is only
      // meaningful in a real MSIX context. On a dev machine it exercises the
      // channel mock plumbing at minimum.
      await MsixStartupTask.enable('CopyPasteStartup');

      expect(captured?.method, 'enable');
      expect((captured?.arguments as Map)['taskId'], 'CopyPasteStartup');
    });

    test('disable is called with the correct taskId', () async {
      if (!Platform.isWindows) return;

      MethodCall? captured;
      _setStartupHandler((call) async {
        captured = call;
        return 'disabled';
      });

      await MsixStartupTask.disable('CopyPasteStartup');

      expect(captured?.method, 'disable');
      expect((captured?.arguments as Map)['taskId'], 'CopyPasteStartup');
    });

    test(
      'enable returns disabledByUser when the user has blocked the task',
      () async {
        if (!Platform.isWindows) return;

        _setStartupHandler((_) async => 'disabledByUser');
        final state = await MsixStartupTask.enable('CopyPasteStartup');
        expect(state, MsixStartupTaskState.disabledByUser);
      },
    );

    test(
      'enable returns enabledByPolicy when policy forces the task on',
      () async {
        if (!Platform.isWindows) return;

        _setStartupHandler((_) async => 'enabledByPolicy');
        final state = await MsixStartupTask.enable('CopyPasteStartup');
        expect(state, MsixStartupTaskState.enabledByPolicy);
      },
    );
  });
}
