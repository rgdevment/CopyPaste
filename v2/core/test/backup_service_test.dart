import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late Directory tempDir;
  late StorageConfig storage;

  setUp(() async {
    tempDir = Directory.systemTemp.createTempSync('backup_test_');
    storage = await StorageConfig.create(baseDir: tempDir.path);
    await storage.ensureDirectories();
  });

  tearDown(() => tempDir.deleteSync(recursive: true));

  group('BackupManifest', () {
    test('toJson and fromJson round-trip', () {
      final now = DateTime.utc(2026, 3, 1);
      final manifest = BackupManifest(
        version: 1,
        appVersion: '2.0.0',
        createdAtUtc: now,
        itemCount: 10,
        imageCount: 2,
        hasPinnedItems: true,
        machineName: 'DESKTOP-TEST',
      );
      final restored = BackupManifest.fromJson(manifest.toJson());
      expect(restored.version, equals(1));
      expect(restored.appVersion, equals('2.0.0'));
      expect(restored.itemCount, equals(10));
      expect(restored.imageCount, equals(2));
      expect(restored.hasPinnedItems, isTrue);
      expect(restored.machineName, equals('DESKTOP-TEST'));
      expect(restored.createdAtUtc, equals(now));
    });

    test('fromJson uses defaults for missing fields', () {
      final manifest = BackupManifest.fromJson({});
      expect(manifest.version, equals(1));
      expect(manifest.appVersion, equals(''));
      expect(manifest.itemCount, equals(0));
      expect(manifest.imageCount, equals(0));
      expect(manifest.hasPinnedItems, isFalse);
    });
  });

  group('BackupService.createBackup', () {
    test('creates zip file at outputPath', () async {
      File(storage.databasePath).writeAsBytesSync([83, 81, 76, 105]);

      final outputPath = p.join(tempDir.path, 'backup.zip');
      await BackupService.createBackup(outputPath, storage, '2.0.0');

      expect(File(outputPath).existsSync(), isTrue);
    });

    test('manifest has correct appVersion', () async {
      final outputPath = p.join(tempDir.path, 'backup2.zip');
      final manifest = await BackupService.createBackup(
        outputPath,
        storage,
        '2.1.0',
      );
      expect(manifest.appVersion, equals('2.1.0'));
    });

    test('counts image files correctly', () async {
      File(p.join(storage.imagesPath, 'a.png')).writeAsBytesSync([1, 2]);
      File(p.join(storage.imagesPath, 'b.png')).writeAsBytesSync([3, 4]);

      final outputPath = p.join(tempDir.path, 'backup3.zip');
      final manifest = await BackupService.createBackup(
        outputPath,
        storage,
        '2.0.0',
      );
      expect(manifest.imageCount, equals(2));
    });

    test('works without database file', () async {
      final outputPath = p.join(tempDir.path, 'backup_empty.zip');
      final manifest = await BackupService.createBackup(
        outputPath,
        storage,
        '2.0.0',
      );
      expect(manifest.imageCount, equals(0));
    });
  });

  group('BackupService.restoreBackup', () {
    test('returns null for nonexistent backup file', () async {
      final result = await BackupService.restoreBackup(
        p.join(tempDir.path, 'missing.zip'),
        storage,
      );
      expect(result, isNull);
    });

    test('round-trip create and restore', () async {
      File(storage.databasePath).writeAsBytesSync([83, 81, 76]);

      final outputPath = p.join(tempDir.path, 'roundtrip.zip');
      await BackupService.createBackup(outputPath, storage, '2.0.0');

      final restoreDir = Directory.systemTemp.createTempSync('restore_');
      try {
        final restoreStorage =
            await StorageConfig.create(baseDir: restoreDir.path);
        final manifest =
            await BackupService.restoreBackup(outputPath, restoreStorage);

        expect(manifest, isNotNull);
        expect(manifest!.appVersion, equals('2.0.0'));
        expect(File(restoreStorage.databasePath).existsSync(), isTrue);
      } finally {
        restoreDir.deleteSync(recursive: true);
      }
    });
  });
}
