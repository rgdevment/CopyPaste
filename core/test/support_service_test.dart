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
      expect(
        SupportService.correctWindowsVersion(raw),
        contains('Windows 11'),
      );
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

      final archive = ZipDecoder().decodeBytes(File(savePath).readAsBytesSync());
      final names = archive.map((f) => f.name).toList();
      expect(names, contains('app.log'));
      expect(names, isNot(contains('readme.txt')));
    });

    test('ZIP always contains device_info.txt', () async {
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.5.1', savePath);

      final archive = ZipDecoder().decodeBytes(File(savePath).readAsBytesSync());
      final infoFile = archive.firstWhere((f) => f.name == 'device_info.txt');
      final content = String.fromCharCodes(infoFile.content as List<int>);
      expect(content, contains('CopyPaste v2.5.1'));
    });

    test('device_info.txt contains platform and Dart version', () async {
      final savePath = p.join(tempDir.path, 'out.zip');
      await SupportService.exportLogs(storage, '2.0.0', savePath);

      final archive = ZipDecoder().decodeBytes(File(savePath).readAsBytesSync());
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

      final archive = ZipDecoder().decodeBytes(File(savePath).readAsBytesSync());
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
  });
}
