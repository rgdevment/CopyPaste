import 'dart:io';

import 'package:archive/archive.dart';
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

    test(
      'does not leave temp files in systemTemp after successful backup',
      () async {
        final outputPath = p.join(tempDir.path, 'backup_no_leak.zip');

        // Snapshot temp files before
        final before = Directory.systemTemp
            .listSync()
            .whereType<File>()
            .where((f) => p.basename(f.path).startsWith('copypaste_backup_'))
            .map((f) => f.path)
            .toSet();

        await BackupService.createBackup(outputPath, storage, '2.0.0');

        // No new copypaste_backup_* files should remain
        final after = Directory.systemTemp
            .listSync()
            .whereType<File>()
            .where((f) => p.basename(f.path).startsWith('copypaste_backup_'))
            .map((f) => f.path)
            .toSet();

        final leaked = after.difference(before);
        expect(leaked, isEmpty, reason: 'Temp backup file was not cleaned up');
      },
    );
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
        final restoreStorage = await StorageConfig.create(
          baseDir: restoreDir.path,
        );
        final manifest = await BackupService.restoreBackup(
          outputPath,
          restoreStorage,
        );

        expect(manifest, isNotNull);
        expect(manifest!.appVersion, equals('2.0.0'));
        expect(File(restoreStorage.databasePath).existsSync(), isTrue);
      } finally {
        restoreDir.deleteSync(recursive: true);
      }
    });

    test('restore creates and cleans up pre-restore snapshot', () async {
      File(storage.databasePath).writeAsBytesSync([83, 81, 76]);

      final outputPath = p.join(tempDir.path, 'snapshot_test.zip');
      await BackupService.createBackup(outputPath, storage, '2.0.0');

      final restoreDir = Directory.systemTemp.createTempSync('snapshot_');
      try {
        final restoreStorage = await StorageConfig.create(
          baseDir: restoreDir.path,
        );
        await restoreStorage.ensureDirectories();
        File(restoreStorage.databasePath).writeAsBytesSync([1, 2, 3]);

        await BackupService.restoreBackup(outputPath, restoreStorage);

        final snapshotDirs = Directory(restoreDir.path)
            .listSync()
            .whereType<Directory>()
            .where((d) => p.basename(d.path).startsWith('.pre-restore-'));
        expect(snapshotDirs, isEmpty);
      } finally {
        restoreDir.deleteSync(recursive: true);
      }
    });
  });

  group('BackupService.validateBackup', () {
    test('returns manifest for valid backup', () async {
      File(storage.databasePath).writeAsBytesSync([83, 81, 76, 105]);

      final outputPath = p.join(tempDir.path, 'validate.zip');
      await BackupService.createBackup(outputPath, storage, '2.0.0');

      final manifest = await BackupService.validateBackup(outputPath);
      expect(manifest, isNotNull);
      expect(manifest!.appVersion, equals('2.0.0'));
    });

    test('returns null for nonexistent file', () async {
      final manifest = await BackupService.validateBackup(
        p.join(tempDir.path, 'missing.zip'),
      );
      expect(manifest, isNull);
    });

    test('returns null for invalid zip', () async {
      final badFile = File(p.join(tempDir.path, 'bad.zip'));
      badFile.writeAsBytesSync([0, 1, 2, 3]);

      final manifest = await BackupService.validateBackup(badFile.path);
      expect(manifest, isNull);
    });
  });

  group('BackupService additional coverage', () {
    test('createBackup calls walCheckpoint when provided', () async {
      var checkpointed = false;
      final outputPath = p.join(tempDir.path, 'wal_backup.zip');
      await BackupService.createBackup(
        outputPath,
        storage,
        '2.0.0',
        walCheckpoint: () async {
          checkpointed = true;
        },
      );
      expect(checkpointed, isTrue);
      expect(File(outputPath).existsSync(), isTrue);
    });

    test(
      'createBackup includes itemCount and hasPinnedItems in manifest',
      () async {
        final outputPath = p.join(tempDir.path, 'metadata_backup.zip');
        final manifest = await BackupService.createBackup(
          outputPath,
          storage,
          '2.0.0',
          itemCount: 42,
          hasPinnedItems: true,
        );
        expect(manifest.itemCount, equals(42));
        expect(manifest.hasPinnedItems, isTrue);
      },
    );

    test('createBackup includes config files', () async {
      await storage.ensureDirectories();
      File(p.join(storage.configPath, 'config.json')).writeAsStringSync('{}');

      final outputPath = p.join(tempDir.path, 'config_backup.zip');
      await BackupService.createBackup(outputPath, storage, '2.0.0');

      final manifest = await BackupService.validateBackup(outputPath);
      expect(manifest, isNotNull);
    });

    test('restoreBackup returns null for version greater than current', () async {
      // Create a backup with a valid archive but version > current
      File(storage.databasePath).writeAsBytesSync([83, 81, 76]);
      final outputPath = p.join(tempDir.path, 'v99_backup.zip');
      // Create a fake zip with version 99 manifest
      final archive = Archive();
      final manifestJson =
          '{"version":99,"appVersion":"99.0","createdAtUtc":"${DateTime.now().toUtc().toIso8601String()}","itemCount":0,"imageCount":0,"hasPinnedItems":false,"machineName":"test"}';
      final manifestBytes = manifestJson.codeUnits;
      archive.addFile(
        ArchiveFile('manifest.json', manifestBytes.length, manifestBytes),
      );
      await File(outputPath).writeAsBytes(ZipEncoder().encode(archive));

      final restoreDir = Directory.systemTemp.createTempSync('restore_v99_');
      try {
        final restoreStorage = await StorageConfig.create(
          baseDir: restoreDir.path,
        );
        final result = await BackupService.restoreBackup(
          outputPath,
          restoreStorage,
        );
        expect(result, isNull);
      } finally {
        restoreDir.deleteSync(recursive: true);
      }
    });

    test('restoreBackup skips files with path traversal', () async {
      final archive = Archive();
      final manifestJson =
          '{"version":1,"appVersion":"2.0","createdAtUtc":"${DateTime.now().toUtc().toIso8601String()}","itemCount":0,"imageCount":0,"hasPinnedItems":false,"machineName":"test"}';
      final manifestBytes = manifestJson.codeUnits;
      archive.addFile(
        ArchiveFile('manifest.json', manifestBytes.length, manifestBytes),
      );
      // Add a file with path traversal
      archive.addFile(ArchiveFile('../evil.txt', 5, [104, 101, 108, 108, 111]));

      final zipPath = p.join(tempDir.path, 'traversal.zip');
      await File(zipPath).writeAsBytes(ZipEncoder().encode(archive));

      final restoreDir = Directory.systemTemp.createTempSync('traversal_r_');
      try {
        final restoreStorage = await StorageConfig.create(
          baseDir: restoreDir.path,
        );
        final result = await BackupService.restoreBackup(
          zipPath,
          restoreStorage,
        );
        expect(result, isNotNull); // succeeds but skips traversal file
        final evil = File(p.join(restoreDir.path, '..', 'evil.txt'));
        expect(evil.existsSync(), isFalse);
      } finally {
        restoreDir.deleteSync(recursive: true);
      }
    });

    test('restoreBackup with images directory restores images', () async {
      await storage.ensureDirectories();
      File(storage.databasePath).writeAsBytesSync([83, 81, 76]);
      File(p.join(storage.imagesPath, 'img.png')).writeAsBytesSync([1, 2, 3]);

      final outputPath = p.join(tempDir.path, 'with_images.zip');
      await BackupService.createBackup(outputPath, storage, '2.0.0');

      final restoreDir = Directory.systemTemp.createTempSync('img_restore_');
      try {
        final restoreStorage = await StorageConfig.create(
          baseDir: restoreDir.path,
        );
        final manifest = await BackupService.restoreBackup(
          outputPath,
          restoreStorage,
        );
        expect(manifest, isNotNull);
        expect(manifest!.imageCount, equals(1));
        expect(
          File(p.join(restoreStorage.imagesPath, 'img.png')).existsSync(),
          isTrue,
        );
      } finally {
        restoreDir.deleteSync(recursive: true);
      }
    });

    test('restoreBackup returns null for invalid zip', () async {
      // Triggers AppLogger.error('restoreBackup failed: $e') in the catch block.
      // snapshotDir is null here because ZipDecoder throws before _createPreRestoreSnapshot.
      final badFile = File(p.join(tempDir.path, 'bad_restore.zip'));
      badFile.writeAsBytesSync([0, 1, 2, 3]);

      final result = await BackupService.restoreBackup(badFile.path, storage);
      expect(result, isNull);
    });

    test('restoreBackup triggers rollback when file extraction fails', () async {
      // Build a valid zip with a clipboard.db entry.
      final archive = Archive();
      final manifestJson =
          '{"version":1,"appVersion":"2.0","createdAtUtc":"${DateTime.now().toUtc().toIso8601String()}","itemCount":0,"imageCount":0,"hasPinnedItems":false,"machineName":"test"}';
      final manifestBytes = manifestJson.codeUnits;
      archive.addFile(
        ArchiveFile('manifest.json', manifestBytes.length, manifestBytes),
      );
      const dbBytes = [83, 81, 76, 105]; // fake SQLite header bytes
      archive.addFile(ArchiveFile('clipboard.db', dbBytes.length, dbBytes));

      final zipPath = p.join(tempDir.path, 'extraction_fail.zip');
      await File(zipPath).writeAsBytes(ZipEncoder().encode(archive));

      final restoreDir = Directory.systemTemp.createTempSync('rollback_t_');
      try {
        final restoreStorage = await StorageConfig.create(
          baseDir: restoreDir.path,
        );
        // Create a DIRECTORY at the path where clipboard.db would be written.
        // File.create(recursive: true) on a directory path throws EISDIR,
        // which causes the catch block to fire with snapshotDir != null,
        // triggering _rollbackFromSnapshot.
        Directory(restoreStorage.databasePath).createSync(recursive: true);

        final result = await BackupService.restoreBackup(
          zipPath,
          restoreStorage,
        );
        // The error is caught; rollback runs; null is returned.
        expect(result, isNull);
      } finally {
        restoreDir.deleteSync(recursive: true);
      }
    });
  });
}
