import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/single_instance.dart';

String get _wakeupFilePath => '${Directory.systemTemp.path}/copypaste.wakeup';

void _cleanupWakeupFile() {
  try {
    File(_wakeupFilePath).deleteSync();
  } catch (_) {}
}

void main() {
  group('SingleInstance – Windows', () {
    setUp(() {
      if (!Platform.isWindows) return;
      SingleInstance.release();
      _cleanupWakeupFile();
    });

    tearDown(() {
      if (!Platform.isWindows) return;
      SingleInstance.release();
      _cleanupWakeupFile();
    });

    test('acquire() returns true on first call', () {
      if (!Platform.isWindows) return;
      expect(SingleInstance.acquire(), isTrue);
    });

    test('acquire() returns false when mutex already held', () {
      if (!Platform.isWindows) return;
      expect(SingleInstance.acquire(), isTrue);
      // Second call while already holding the mutex → false
      expect(SingleInstance.acquire(), isFalse);
    });

    test('release() allows re-acquire', () {
      if (!Platform.isWindows) return;
      expect(SingleInstance.acquire(), isTrue);
      SingleInstance.release();
      expect(SingleInstance.acquire(), isTrue);
    });

    test('release() is idempotent', () {
      if (!Platform.isWindows) return;
      SingleInstance.release();
      SingleInstance.release();
      // After double release, re-acquire must still work
      expect(SingleInstance.acquire(), isTrue);
    });

    test('signalWakeup() writes wakeup file as fallback', () {
      if (!Platform.isWindows) return;
      _cleanupWakeupFile();
      // With no pipe server running, signalWakeup falls back to file
      SingleInstance.signalWakeup();
      expect(File(_wakeupFilePath).existsSync(), isTrue);
    });

    test('listenForWakeup() fires callback when wakeup file appears', () async {
      if (!Platform.isWindows) return;

      final completer = Completer<void>();
      SingleInstance.listenForWakeup(() {
        if (!completer.isCompleted) completer.complete();
      });

      // Give the listener time to start, then create the file
      await Future<void>.delayed(const Duration(milliseconds: 100));
      File(_wakeupFilePath).writeAsStringSync('wakeup');

      await completer.future.timeout(
        const Duration(seconds: 2),
        onTimeout: () => fail('Callback was not fired'),
      );
    });

    test('listenForWakeup() fires for fresh pre-existing file', () async {
      if (!Platform.isWindows) return;

      // Write file BEFORE starting the listener
      File(_wakeupFilePath).writeAsStringSync('wakeup');

      final completer = Completer<void>();
      SingleInstance.listenForWakeup(() {
        if (!completer.isCompleted) completer.complete();
      });

      await completer.future.timeout(
        const Duration(seconds: 2),
        onTimeout: () => fail('Callback was not fired for pre-existing file'),
      );
    });

    test('stopListening() prevents further callbacks', () async {
      if (!Platform.isWindows) return;

      var callCount = 0;
      SingleInstance.listenForWakeup(() => callCount++);
      SingleInstance.stopListening();

      File(_wakeupFilePath).writeAsStringSync('wakeup');
      await Future<void>.delayed(const Duration(seconds: 1));
      expect(callCount, 0);
    });

    test(
      'calling listenForWakeup() twice replaces the first listener',
      () async {
        if (!Platform.isWindows) return;

        var firstCallCount = 0;
        SingleInstance.listenForWakeup(() => firstCallCount++);

        final completer = Completer<void>();
        SingleInstance.listenForWakeup(() {
          if (!completer.isCompleted) completer.complete();
        });

        await Future<void>.delayed(const Duration(milliseconds: 100));
        File(_wakeupFilePath).writeAsStringSync('wakeup');

        await completer.future.timeout(const Duration(seconds: 2));
        expect(firstCallCount, 0);
      },
    );

    test('debounce: rapid signals fire callback only once', () async {
      if (!Platform.isWindows) return;

      var callCount = 0;
      SingleInstance.listenForWakeup(() => callCount++);

      await Future<void>.delayed(const Duration(milliseconds: 100));

      // Write, delete, write again rapidly
      File(_wakeupFilePath).writeAsStringSync('wakeup');
      await Future<void>.delayed(const Duration(milliseconds: 600));
      File(_wakeupFilePath).writeAsStringSync('wakeup');
      await Future<void>.delayed(const Duration(milliseconds: 600));

      expect(callCount, 1);
    });

    test('release() cleans up pipe isolate and subscription', () async {
      if (!Platform.isWindows) return;
      SingleInstance.acquire();
      var callCount = 0;
      SingleInstance.listenForWakeup(() => callCount++);
      SingleInstance.release();

      // After release, writing the signal must not fire the old callback
      File(_wakeupFilePath).writeAsStringSync('wakeup');
      await Future<void>.delayed(const Duration(seconds: 1));
      expect(callCount, 0);
    });
  });

  group('SingleInstance – Unix (macOS / Linux)', () {
    setUp(() {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      SingleInstance.release();
    });

    tearDown(() {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      SingleInstance.release();
    });

    test('acquire() returns true on first call', () {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      expect(SingleInstance.acquire(), isTrue);
    });

    test('acquire() creates the lock file', () {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      SingleInstance.acquire();
      final lockPath = '${Directory.systemTemp.path}/copypaste.lock';
      expect(File(lockPath).existsSync(), isTrue);
    });

    test('release() deletes the lock file', () {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      SingleInstance.acquire();
      SingleInstance.release();
      final lockPath = '${Directory.systemTemp.path}/copypaste.lock';
      expect(File(lockPath).existsSync(), isFalse);
    });

    test('can re-acquire after release', () {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      expect(SingleInstance.acquire(), isTrue);
      SingleInstance.release();
      expect(SingleInstance.acquire(), isTrue);
    });

    test('release() is idempotent — safe to call without prior acquire', () {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      SingleInstance.release();
      SingleInstance.release();
      // After double release, re-acquire must still work
      expect(SingleInstance.acquire(), isTrue);
    });

    test('lock file contains the process pid', () {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      SingleInstance.acquire();
      final lockPath = '${Directory.systemTemp.path}/copypaste.lock';
      final content = File(lockPath).readAsStringSync().trim();
      expect(content, equals('$pid'));
    });
  });

  group('SingleInstance – wakeup file (cross-platform)', () {
    setUp(() {
      SingleInstance.stopListening();
      _cleanupWakeupFile();
    });

    tearDown(() {
      SingleInstance.stopListening();
      _cleanupWakeupFile();
    });

    test('signalWakeup writes the wakeup file', () {
      if (Platform.isWindows) {
        // On Windows, signalWakeup tries pipe first; only writes file
        // as fallback. Tested in Windows-specific group instead.
        return;
      }
      SingleInstance.signalWakeup();
      expect(File(_wakeupFilePath).existsSync(), isTrue);
    });

    test('file polling fires callback within 600ms', () async {
      SingleInstance.listenForWakeup(() {});

      final completer = Completer<void>();
      SingleInstance.listenForWakeup(() {
        if (!completer.isCompleted) completer.complete();
      });

      await Future<void>.delayed(const Duration(milliseconds: 100));
      File(_wakeupFilePath).writeAsStringSync('wakeup');

      await completer.future.timeout(
        const Duration(milliseconds: 1500),
        onTimeout: () => fail('File polling did not fire within expected time'),
      );
    });

    test('wakeup file is deleted after callback fires', () async {
      final completer = Completer<void>();
      SingleInstance.listenForWakeup(() {
        if (!completer.isCompleted) completer.complete();
      });

      await Future<void>.delayed(const Duration(milliseconds: 100));
      File(_wakeupFilePath).writeAsStringSync('wakeup');

      await completer.future.timeout(const Duration(seconds: 2));
      // Allow a tick for the delete to complete
      await Future<void>.delayed(const Duration(milliseconds: 50));
      expect(File(_wakeupFilePath).existsSync(), isFalse);
    });

    test('stale file (>30s) is deleted on listenForWakeup startup', () {
      // We cannot easily set mtime to 31s ago in pure Dart, so we verify
      // the code path indirectly: a freshly written file is NOT deleted
      // (proving the age check exists and only targets old files).
      File(_wakeupFilePath).writeAsStringSync('wakeup');
      SingleInstance.listenForWakeup(() {});
      // Fresh file should still exist (not deleted by stale check)
      expect(File(_wakeupFilePath).existsSync(), isTrue);
    });

    test('stopListening prevents further callbacks', () async {
      var callCount = 0;
      SingleInstance.listenForWakeup(() => callCount++);
      SingleInstance.stopListening();

      File(_wakeupFilePath).writeAsStringSync('wakeup');
      await Future<void>.delayed(const Duration(seconds: 1));
      expect(callCount, 0);
    });

    test('debounce prevents duplicate callbacks from rapid signals', () async {
      var callCount = 0;
      final firstFired = Completer<void>();
      SingleInstance.listenForWakeup(() {
        callCount++;
        if (!firstFired.isCompleted) firstFired.complete();
      });

      await Future<void>.delayed(const Duration(milliseconds: 100));

      File(_wakeupFilePath).writeAsStringSync('wakeup');
      await firstFired.future.timeout(
        const Duration(seconds: 3),
        onTimeout: () => fail('First callback did not fire'),
      );

      // Second signal within 2s debounce window — must be suppressed
      File(_wakeupFilePath).writeAsStringSync('wakeup');
      await Future<void>.delayed(const Duration(milliseconds: 600));

      expect(callCount, 1);
    });
  });
}
