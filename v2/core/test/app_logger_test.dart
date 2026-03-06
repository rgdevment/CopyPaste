import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  // AppLogger uses static state: each test file is its own Dart isolate,
  // so state starts fresh here.

  group('AppLogger before initialization', () {
    test('info/warn/error do nothing before init', () {
      expect(() => AppLogger.info('pre-init'), returnsNormally);
      expect(() => AppLogger.warn('pre-init'), returnsNormally);
      expect(() => AppLogger.error('pre-init'), returnsNormally);
    });

    test('exception returns early when not initialized', () {
      expect(
        () => AppLogger.exception(Exception('test')),
        returnsNormally,
      );
    });

    test('logFilePath is null before init', () {
      expect(AppLogger.logFilePath, isNull);
    });

    test('logDirectory is null before init', () {
      expect(AppLogger.logDirectory, isNull);
    });
  });

  group('AppLogger initialization', () {
    late Directory tempDir;

    setUpAll(() {
      tempDir = Directory.systemTemp.createTempSync('logger_test_init_');
      // Create an old log file to test cleanup
      final oldFile = File(p.join(tempDir.path, 'copypaste_2020-01-01.log'));
      oldFile.writeAsStringSync('old log entry\n');
      // Set modification time to 10 days ago
      final oldTime = DateTime.now().subtract(const Duration(days: 10));
      oldFile.setLastModifiedSync(oldTime);

      // Create a non-log file (should not be deleted)
      File(p.join(tempDir.path, 'other.txt')).writeAsStringSync('other');

      AppLogger.initialize(tempDir.path);
    });

    tearDownAll(() {
      AppLogger.isEnabled = false;
      tempDir.deleteSync(recursive: true);
    });

    test('logFilePath is set after init', () {
      expect(AppLogger.logFilePath, isNotNull);
      expect(AppLogger.logFilePath, contains('copypaste_'));
      expect(AppLogger.logFilePath, endsWith('.log'));
    });

    test('logDirectory matches provided path', () {
      expect(AppLogger.logDirectory, equals(tempDir.path));
    });

    test('isEnabled is true after successful init', () {
      expect(AppLogger.isEnabled, isTrue);
    });

    test('old log files are cleaned up during init', () {
      final oldFile = File(p.join(tempDir.path, 'copypaste_2020-01-01.log'));
      expect(oldFile.existsSync(), isFalse);
    });

    test('non-log files are not cleaned up', () {
      final other = File(p.join(tempDir.path, 'other.txt'));
      expect(other.existsSync(), isTrue);
    });

    test('second call to initialize is a no-op', () {
      final pathBefore = AppLogger.logFilePath;
      AppLogger.initialize('/some/other/path');
      expect(AppLogger.logFilePath, equals(pathBefore));
    });

    test('info writes INFO entry to log', () {
      AppLogger.info('test info msg');
      final content = File(AppLogger.logFilePath!).readAsStringSync();
      expect(content, contains('test info msg'));
      expect(content, contains('INFO'));
    });

    test('warn writes WARN entry to log', () {
      AppLogger.warn('test warn msg');
      final content = File(AppLogger.logFilePath!).readAsStringSync();
      expect(content, contains('test warn msg'));
      expect(content, contains('WARN'));
    });

    test('error writes ERROR entry to log', () {
      AppLogger.error('test error msg');
      final content = File(AppLogger.logFilePath!).readAsStringSync();
      expect(content, contains('test error msg'));
      expect(content, contains('ERROR'));
    });

    test('exception with context writes context and error', () {
      AppLogger.exception(
        Exception('test exception'),
        null,
        'TestContext',
      );
      final content = File(AppLogger.logFilePath!).readAsStringSync();
      expect(content, contains('TestContext'));
      expect(content, contains('test exception'));
    });

    test('exception with stackTrace includes stack trace', () {
      try {
        throw Exception('stacktrace test');
      } catch (e, s) {
        AppLogger.exception(e, s, 'StackContext');
      }
      final content = File(AppLogger.logFilePath!).readAsStringSync();
      expect(content, contains('stacktrace test'));
    });

    test('exception without context writes error only', () {
      AppLogger.exception(Exception('no context'));
      final content = File(AppLogger.logFilePath!).readAsStringSync();
      expect(content, contains('no context'));
    });

    test('exception with null error does nothing', () {
      final before = File(AppLogger.logFilePath!).readAsStringSync();
      AppLogger.exception(null);
      final after = File(AppLogger.logFilePath!).readAsStringSync();
      expect(after, equals(before));
    });

    test('log entry format includes timestamp brackets', () {
      AppLogger.info('format check');
      final lines = File(AppLogger.logFilePath!).readAsLinesSync();
      final line = lines.lastWhere((l) => l.contains('format check'));
      expect(line, matches(r'^\[\d{2}:\d{2}:\d{2}\.\d{3}\] \[INFO\]'));
    });

    test('isEnabled=false suppresses all logging', () {
      AppLogger.isEnabled = false;
      final before = File(AppLogger.logFilePath!).readAsStringSync();
      AppLogger.info('should not appear');
      AppLogger.warn('should not appear');
      AppLogger.error('should not appear');
      AppLogger.exception(Exception('should not appear'));
      final after = File(AppLogger.logFilePath!).readAsStringSync();
      expect(after, equals(before));
      AppLogger.isEnabled = true; // restore
    });

    test('log rotation triggered when file exceeds max size', () {
      final logFile = File(AppLogger.logFilePath!);
      // Write 11MB to trigger rotation (max is 10MB)
      logFile.writeAsBytesSync(Uint8List(11 * 1024 * 1024));
      final originalPath = AppLogger.logFilePath;

      AppLogger.info('post rotation');

      // The original large content should be gone (renamed or deleted)
      final currentSize = File(AppLogger.logFilePath!).existsSync()
          ? File(AppLogger.logFilePath!).lengthSync()
          : 0;
      expect(
        currentSize,
        lessThan(11 * 1024 * 1024),
        reason: 'Log should have been rotated, original file gone or small',
      );
      expect(originalPath, isNotNull);
    });
  });
}
