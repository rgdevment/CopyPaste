import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('crash_logger_test_');
    CrashLogger.initialize(tempDir.path);
  });

  tearDown(() {
    CrashLogger.initialize('');
    tempDir.deleteSync(recursive: true);
  });

  group('CrashLogger.initialize', () {
    test('sets filePath after initialize', () {
      expect(CrashLogger.filePath, equals(p.join(tempDir.path, 'crash.log')));
    });

    test('creates the base directory if it does not exist', () async {
      final sub = Directory(p.join(tempDir.path, 'newdir'));
      expect(sub.existsSync(), isFalse);
      CrashLogger.initialize(sub.path);
      expect(sub.existsSync(), isTrue);
    });

    test('does not throw on invalid path', () {
      expect(() => CrashLogger.initialize('\x00invalid\x00'), returnsNormally);
    });
  });

  group('CrashLogger.report', () {
    test('creates crash.log on first report', () {
      final path = p.join(tempDir.path, 'crash.log');
      expect(File(path).existsSync(), isFalse);
      CrashLogger.report(Exception('boom'), null);
      expect(File(path).existsSync(), isTrue);
    });

    test('written content contains timestamp marker', () {
      CrashLogger.report(Exception('test'), null);
      final content = File(
        p.join(tempDir.path, 'crash.log'),
      ).readAsStringSync();
      expect(content, contains('===='));
    });

    test('written content contains platform name', () {
      CrashLogger.report(Exception('test'), null);
      final content = File(
        p.join(tempDir.path, 'crash.log'),
      ).readAsStringSync();
      expect(content, contains('Platform:'));
    });

    test('written content contains Dart version', () {
      CrashLogger.report(Exception('test'), null);
      final content = File(
        p.join(tempDir.path, 'crash.log'),
      ).readAsStringSync();
      expect(content, contains('Dart:'));
    });

    test('written content includes the error message', () {
      CrashLogger.report(Exception('specific_error_XYZ'), null);
      final content = File(
        p.join(tempDir.path, 'crash.log'),
      ).readAsStringSync();
      expect(content, contains('specific_error_XYZ'));
    });

    test('written content includes context when provided', () {
      CrashLogger.report(Exception('e'), null, context: 'myContext');
      final content = File(
        p.join(tempDir.path, 'crash.log'),
      ).readAsStringSync();
      expect(content, contains('myContext'));
    });

    test('written content omits context line when context is empty', () {
      CrashLogger.report(Exception('e'), null);
      final content = File(
        p.join(tempDir.path, 'crash.log'),
      ).readAsStringSync();
      expect(content, isNot(contains('Context:')));
    });

    test('written content includes stack trace when provided', () {
      final stack = StackTrace.fromString('frame at crash_logger_test.dart:1');
      CrashLogger.report(Exception('e'), stack);
      final content = File(
        p.join(tempDir.path, 'crash.log'),
      ).readAsStringSync();
      expect(content, contains('Stack:'));
      expect(content, contains('crash_logger_test.dart'));
    });

    test('multiple reports are appended', () {
      CrashLogger.report(Exception('first'), null);
      CrashLogger.report(Exception('second'), null);
      final content = File(
        p.join(tempDir.path, 'crash.log'),
      ).readAsStringSync();
      expect(content, contains('first'));
      expect(content, contains('second'));
    });

    test('overridePath writes to custom path', () {
      final custom = p.join(tempDir.path, 'custom.log');
      CrashLogger.report(Exception('override'), null, overridePath: custom);
      expect(File(custom).existsSync(), isTrue);
      final content = File(custom).readAsStringSync();
      expect(content, contains('override'));
      expect(File(p.join(tempDir.path, 'crash.log')).existsSync(), isFalse);
    });

    test('truncates file when it exceeds max size', () {
      final path = p.join(tempDir.path, 'crash.log');
      final filler = 'x' * (512 * 1024 + 1);
      File(path).writeAsStringSync(filler);
      CrashLogger.report(Exception('after_truncate'), null);
      final content = File(path).readAsStringSync();
      expect(content, isNot(contains('x' * 100)));
      expect(content, contains('after_truncate'));
    });

    test(
      'does not throw when filePath is null and bootstrap path unavailable',
      () {
        CrashLogger.initialize('');
        expect(
          () => CrashLogger.report(Exception('e'), null, overridePath: null),
          returnsNormally,
        );
      },
    );
  });

  group('CrashLogger.redact — HOME substitution', () {
    test('replaces USERPROFILE/HOME value with <HOME>', () {
      final home =
          Platform.environment['USERPROFILE'] ??
          Platform.environment['HOME'] ??
          '';
      if (home.isEmpty) return;
      final input = 'path is $home\\something';
      expect(CrashLogger.redact(input), isNot(contains(home)));
      expect(CrashLogger.redact(input), contains('<HOME>'));
    });

    test('does not modify string with no sensitive data', () {
      expect(CrashLogger.redact('hello world'), equals('hello world'));
    });
  });

  group('CrashLogger.redact — username substitution', () {
    final username =
        Platform.environment['USERNAME'] ?? Platform.environment['USER'] ?? '';

    test('replaces /home/<user> on posix-style paths', () {
      if (username.isEmpty || username.length <= 1) return;
      final input = '/home/$username/config/file.db';
      expect(CrashLogger.redact(input), isNot(contains('/home/$username')));
    });

    test('replaces /Users/<user> on macOS-style paths', () {
      if (username.isEmpty || username.length <= 1) return;
      final input = '/Users/$username/Library/file.db';
      expect(CrashLogger.redact(input), isNot(contains('/Users/$username')));
    });

    test('replaces \\Users\\<user> on Windows-style paths', () {
      if (username.isEmpty || username.length <= 1) return;
      final input = 'C:\\Users\\$username\\AppData\\file.db';
      expect(CrashLogger.redact(input), isNot(contains('\\$username\\')));
    });

    test('does not crash on empty string', () {
      expect(() => CrashLogger.redact(''), returnsNormally);
      expect(CrashLogger.redact(''), equals(''));
    });
  });

  group('CrashLogger.redact — email', () {
    test('replaces plain email address', () {
      final result = CrashLogger.redact('contact user@example.com for help');
      expect(result, contains('<EMAIL>'));
      expect(result, isNot(contains('user@example.com')));
    });

    test('replaces email in a stack trace line', () {
      const line = 'Exception: auth failed for admin@corp.io at line 42';
      final result = CrashLogger.redact(line);
      expect(result, contains('<EMAIL>'));
      expect(result, isNot(contains('admin@corp.io')));
    });

    test('replaces multiple emails in one string', () {
      const input = 'a@a.com and b@b.org both failed';
      final result = CrashLogger.redact(input);
      expect(result, isNot(contains('@')));
    });

    test('does not alter strings without an @', () {
      const input = 'no email here, just text';
      expect(CrashLogger.redact(input), equals(input));
    });

    test('does not treat non-email @ as email', () {
      const input = '@handle is not an email';
      expect(CrashLogger.redact(input), equals(input));
    });
  });

  group('CrashLogger.redact — no false positives', () {
    test('preserves normal log lines', () {
      const line = '[12:00:01.123] [INFO] Bootstrap: CopyPaste 2.0 starting';
      expect(CrashLogger.redact(line), equals(line));
    });

    test('preserves error codes and hex addresses', () {
      const line = 'Error 0x80070005 at kernel32.dll+0x1234';
      expect(CrashLogger.redact(line), equals(line));
    });
  });
}
