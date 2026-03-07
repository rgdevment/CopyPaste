import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:app/shell/single_instance.dart';

void main() {
  group('SingleInstance – Unix (macOS / Linux)', () {
    setUp(() {
      // Ensure a clean state before every test.
      SingleInstance.release();
    });

    tearDown(() {
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
      // Must not throw.
      SingleInstance.release();
      SingleInstance.release();
    });

    test('lock file contains the process pid', () {
      if (!Platform.isMacOS && !Platform.isLinux) return;
      SingleInstance.acquire();
      final lockPath = '${Directory.systemTemp.path}/copypaste.lock';
      final content = File(lockPath).readAsStringSync().trim();
      expect(content, equals('$pid'));
    });
  });
}
