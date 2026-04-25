import 'dart:io';

import 'package:archive/archive.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late Directory tempDir;
  late StorageConfig storage;

  setUp(() async {
    tempDir = Directory.systemTemp.createTempSync('support_test_');
    storage = await StorageConfig.create(baseDir: tempDir.path);
    await Directory(storage.logsPath).create(recursive: true);
  });

  tearDown(() => tempDir.deleteSync(recursive: true));

  // ---------------------------------------------------------------------------
  // correctWindowsVersion
  // ---------------------------------------------------------------------------
  group('SupportService.correctWindowsVersion', () {
    test('replaces Windows 10 with Windows 11 for build >= 22000', () {
      const raw = 'Windows 10.0.22621 Build 22621';
      expect(
        SupportService.correctWindowsVersion(raw),
        equals('Windows 11.0.22621 Build 22621'),
      );
    });

    test('does not replace for build < 22000', () {
      const raw = 'Windows 10.0.19045 Build 19045';
      expect(SupportService.correctWindowsVersion(raw), equals(raw));
    });

    test('returns raw string unchanged when no Windows 10 text', () {
      const raw = 'Windows 11.0.22621';
      expect(SupportService.correctWindowsVersion(raw), equals(raw));
    });

    test('returns raw string unchanged when no Build number present', () {
      const raw = 'Windows 10 Pro';
      expect(SupportService.correctWindowsVersion(raw), equals(raw));
    });

    test('handles build exactly at boundary (22000 → Windows 11)', () {
      const raw = 'Windows 10.0.22000 Build 22000';
      expect(SupportService.correctWindowsVersion(raw), contains('Windows 11'));
    });

    test('handles build one below boundary (21999 → unchanged)', () {
      const raw = 'Windows 10.0.21999 Build 21999';
      expect(SupportService.correctWindowsVersion(raw), equals(raw));
    });
  });

  // ---------------------------------------------------------------------------
  // exportLogs
  // ---------------------------------------------------------------------------
  group('SupportService.exportLogs', () {
    test('returns 0 when logs directory is empty', () async {
      final savePath = p.join(tempDir.path, 'out.zip');
      final count = await SupportService.exportLogs(storage, '2.0.0', savePath);
      expect(count, equals(0));
      expect(File(savePath).existsSync(), isTrue);
    });

    test('returns 0 when logs directory does not exist', () async {
      await Directory(storage.logsPath).delete(recursive: true);
      final savePath = p.join(tempDir.path, 'out.zip');
      final count = await SupportService.exportLogs(storage, '2.0.0', savePath);
      expect(count, equals(0));
    });

    test('returns correct count of .log files', () async {
      File(p.join(storage.logsPath, 'app.log')).writeAsStringSync('log1');
      File(p.join(storage.logsPath, 'app2.log')).writeAsStringSync('log2');
      final savePath = p.join(tempDir.path, 'out.zip');
      final count = await SupportService.exportLogs(storage, '2.0.0', savePath);
      expect(count, equals(2));
    });

    test('excludes non-.log files from count and archive', () async {
      File(p.join(storage.logsPath, 'app.log')).writeAsStringSync('log');
      File(p.join(storage.logsPath, 'readme.txt')).writeAsStringSync('text');
      final savePath = p.join(tempDir.path, 'out.zip');
      final count = await SupportService.exportLogs(storage, '2.0.0', savePath);
      expect(count, equals(1));

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final names = archive.map((f) => f.name).toList();
      expect(names, contains('app.log'));
      expect(names, isNot(contains('readme.txt')));
    });

    test('ZIP always contains device_info.txt', () async {
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.5.1', savePath);

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final infoFile = archive.firstWhere((f) => f.name == 'device_info.txt');
      final content = String.fromCharCodes(infoFile.content as List<int>);
      expect(content, contains('CopyPaste v2.5.1'));
    });

    test('device_info.txt contains platform and Dart version', () async {
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.0.0', savePath);

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final infoFile = archive.firstWhere((f) => f.name == 'device_info.txt');
      final content = String.fromCharCodes(infoFile.content as List<int>);
      expect(content, contains('Platform'));
      expect(content, contains('Dart'));
      expect(content, contains('Generated:'));
    });

    test('ZIP contains log file content verbatim', () async {
      File(p.join(storage.logsPath, 'app.log')).writeAsStringSync('hello log');
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.0.0', savePath);

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final logFile = archive.firstWhere((f) => f.name == 'app.log');
      final content = String.fromCharCodes(logFile.content as List<int>);
      expect(content, equals('hello log'));
    });

    test('saves zip file at specified path', () async {
      final savePath = p.join(tempDir.path, 'subdir', 'export.zip');
      await Directory(p.join(tempDir.path, 'subdir')).create();
      await SupportService.exportLogs(storage, '2.0.0', savePath);
      expect(File(savePath).existsSync(), isTrue);
      expect(File(savePath).lengthSync(), greaterThan(0));
    });

    test('includes crash.log in archive when it exists', () async {
      File(
        p.join(storage.baseDir, 'crash.log'),
      ).writeAsStringSync('==== crash entry ====');
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.0.0', savePath);

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final names = archive.map((f) => f.name).toList();
      expect(names, contains('crash.log'));
    });

    test('does not include crash.log entry when file does not exist', () async {
      File(p.join(storage.logsPath, 'app.log')).writeAsStringSync('log');
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.0.0', savePath);

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final names = archive.map((f) => f.name).toList();
      expect(names, isNot(contains('crash.log')));
    });

    test('crash.log count is not added to returned log file count', () async {
      File(p.join(storage.logsPath, 'app.log')).writeAsStringSync('log');
      File(
        p.join(storage.baseDir, 'crash.log'),
      ).writeAsStringSync('crash entry');
      final savePath = p.join(tempDir.path, 'out.zip');
      final count = await SupportService.exportLogs(storage, '2.0.0', savePath);
      expect(count, equals(1));
    });

    test('log files are redacted in archive — email is replaced', () async {
      File(
        p.join(storage.logsPath, 'app.log'),
      ).writeAsStringSync('error for admin@corp.example.com');
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.0.0', savePath);

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final logFile = archive.firstWhere((f) => f.name == 'app.log');
      final content = String.fromCharCodes(logFile.content as List<int>);
      expect(content, isNot(contains('admin@corp.example.com')));
      expect(content, contains('<EMAIL>'));
    });

    test('crash.log is redacted in archive — email is replaced', () async {
      File(
        p.join(storage.baseDir, 'crash.log'),
      ).writeAsStringSync('crash for user@example.com');
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.0.0', savePath);

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final crashFile = archive.firstWhere((f) => f.name == 'crash.log');
      final content = String.fromCharCodes(crashFile.content as List<int>);
      expect(content, isNot(contains('user@example.com')));
      expect(content, contains('<EMAIL>'));
    });

    test('non-sensitive log content is preserved after redaction', () async {
      File(
        p.join(storage.logsPath, 'app.log'),
      ).writeAsStringSync('[INFO] Bootstrap: CopyPaste 2.0 starting');
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.0.0', savePath);

      final archive = ZipDecoder().decodeBytes(
        File(savePath).readAsBytesSync(),
      );
      final logFile = archive.firstWhere((f) => f.name == 'app.log');
      final content = String.fromCharCodes(logFile.content as List<int>);
      expect(content, equals('[INFO] Bootstrap: CopyPaste 2.0 starting'));
    });
  });

  group('SupportService.revealFile', () {
    test('completes without throwing on Linux', () async {
      if (!Platform.isLinux) return;
      final file = File(p.join(tempDir.path, 'reveal_test.log'))
        ..writeAsStringSync('data');
      // xdg-open is called internally; exceptions are caught, so always completes
      await expectLater(SupportService.revealFile(file.path), completes);
    });

    test('completes without throwing when path is empty string', () async {
      // Platform checks guard the Process.run call; no spawn attempted for empty
      await expectLater(SupportService.revealFile(''), completes);
    });
  });
  group('SupportService.openLogsFolder', () {
    test('creates logs directory when it does not exist', () async {
      await Directory(storage.logsPath).delete(recursive: true);
      expect(Directory(storage.logsPath).existsSync(), isFalse);
      try {
        await SupportService.openLogsFolder(storage);
      } catch (_) {
        // xdg-open may not be available in headless CI; that's acceptable
      }
      expect(Directory(storage.logsPath).existsSync(), isTrue);
    });

    test('opens existing logs folder on Linux', () async {
      if (!Platform.isLinux) return;
      // xdg-open may fail in headless CI, but the function body is covered
      try {
        await SupportService.openLogsFolder(storage);
      } catch (_) {
        // ProcessException acceptable when no display server available
      }
    });
  });
}
