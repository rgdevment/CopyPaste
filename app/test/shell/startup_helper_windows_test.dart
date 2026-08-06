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

  group('StartupHelper.stableExecutablePath', () {
    late Directory root;

    setUp(() {
      root = Directory.systemTemp.createTempSync('scoop_layout_');
    });

    tearDown(() => root.deleteSync(recursive: true));

    String seed(String versionDir, {bool withCurrent = true}) {
      final versioned = Directory(
        '${root.path}${Platform.pathSeparator}apps'
        '${Platform.pathSeparator}copypaste'
        '${Platform.pathSeparator}$versionDir',
      )..createSync(recursive: true);
      final exe = File(
        '${versioned.path}${Platform.pathSeparator}CopyPaste.exe',
      )..writeAsStringSync('');
      if (withCurrent) {
        final current = Directory(
          '${root.path}${Platform.pathSeparator}apps'
          '${Platform.pathSeparator}copypaste'
          '${Platform.pathSeparator}current',
        )..createSync(recursive: true);
        File(
          '${current.path}${Platform.pathSeparator}CopyPaste.exe',
        ).writeAsStringSync('');
      }
      return exe.path;
    }

    test('rewrites a versioned Scoop path to current', () {
      final resolved = StartupHelper.stableExecutablePath(seed('2.9.0'));
      expect(resolved, contains('current'));
      expect(resolved, isNot(contains('2.9.0')));
    });

    test('keeps the versioned path when current does not exist', () {
      final versioned = seed('2.9.0', withCurrent: false);
      expect(StartupHelper.stableExecutablePath(versioned), versioned);
    });

    test('leaves a path already on current untouched', () {
      seed('2.9.0');
      final currentExe =
          '${root.path}${Platform.pathSeparator}apps'
          '${Platform.pathSeparator}copypaste'
          '${Platform.pathSeparator}current'
          '${Platform.pathSeparator}CopyPaste.exe';
      expect(StartupHelper.stableExecutablePath(currentExe), currentExe);
    });

    test('leaves a standalone install untouched', () {
      const standalone = r'C:\Users\dev\AppData\Local\CopyPaste\CopyPaste.exe';
      expect(StartupHelper.stableExecutablePath(standalone), standalone);
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
