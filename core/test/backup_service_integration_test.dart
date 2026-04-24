/// Integration tests for BackupService with a live SqliteRepository.
/// Verifies that a backup created from real repository data can be fully
/// restored and the contents are intact across all platforms.
library;

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late Directory sourceDir;
  late Directory destDir;
  late StorageConfig sourceStorage;
  late StorageConfig destStorage;
  late SqliteRepository repo;

  setUp(() async {
    sourceDir = Directory.systemTemp.createTempSync('backup_int_src_');
    destDir = Directory.systemTemp.createTempSync('backup_int_dst_');
    sourceStorage = await StorageConfig.create(baseDir: sourceDir.path);
    destStorage = await StorageConfig.create(baseDir: destDir.path);
    await sourceStorage.ensureDirectories();
    await destStorage.ensureDirectories();
    repo = SqliteRepository.fromPath(sourceStorage.databasePath);
  });

  tearDown(() async {
    await repo.close();
    if (sourceDir.existsSync()) sourceDir.deleteSync(recursive: true);
    if (destDir.existsSync()) destDir.deleteSync(recursive: true);
  });

  group('BackupService integration – full round-trip', () {
    test('backup captures item count from service', () async {
      final service = ClipboardService(repo);
      await service.processText('item one', ClipboardContentType.text);
      await service.processText('item two', ClipboardContentType.text);
      service.dispose();

      final backupPath = p.join(sourceDir.path, 'snapshot.zip');
      final manifest = await BackupService.createBackup(
        backupPath,
        sourceStorage,
        '2.0.0',
        itemCount: await repo.count(),
        hasPinnedItems: false,
        walCheckpoint: () => repo.walCheckpoint(),
      );

      expect(manifest.itemCount, equals(2));
    });

    test('restored database contains original clipboard items', () async {
      final service = ClipboardService(repo);
      await service.processText('first entry', ClipboardContentType.text);
      await service.processText('second entry', ClipboardContentType.text);
      service.dispose();

      final backupPath = p.join(sourceDir.path, 'full_restore.zip');
      await BackupService.createBackup(
        backupPath,
        sourceStorage,
        '2.0.0',
        walCheckpoint: () => repo.walCheckpoint(),
      );

      await repo.close();

      final restored = await BackupService.restoreBackup(
        backupPath,
        destStorage,
      );
      expect(restored, isNotNull);

      final destRepo = SqliteRepository.fromPath(destStorage.databasePath);
      try {
        final items = await destRepo.getAll();
        final contents = items.map((i) => i.content).toSet();
        expect(contents, contains('first entry'));
        expect(contents, contains('second entry'));
      } finally {
        await destRepo.close();
      }
    });

    test('restored database preserves pinned status', () async {
      final service = ClipboardService(repo);
      final pinned = await service.processText(
        'pinned item',
        ClipboardContentType.text,
      );
      await service.updatePin(pinned!.id, true);

      await service.processText('normal item', ClipboardContentType.text);
      service.dispose();

      final backupPath = p.join(sourceDir.path, 'pinned_restore.zip');
      await BackupService.createBackup(
        backupPath,
        sourceStorage,
        '2.1.0',
        walCheckpoint: () => repo.walCheckpoint(),
      );
      await repo.close();

      await BackupService.restoreBackup(backupPath, destStorage);

      final destRepo = SqliteRepository.fromPath(destStorage.databasePath);
      try {
        final items = await destRepo.getAll();
        final pinnedRestored = items.firstWhere(
          (i) => i.content == 'pinned item',
        );
        final normalRestored = items.firstWhere(
          (i) => i.content == 'normal item',
        );
        expect(pinnedRestored.isPinned, isTrue);
        expect(normalRestored.isPinned, isFalse);
      } finally {
        await destRepo.close();
      }
    });

    test('backup captures image files', () async {
      final imgFile = File(p.join(sourceStorage.imagesPath, 'test.png'))
        ..writeAsBytesSync([137, 80, 78, 71, 13, 10, 26, 10]); // PNG header

      final backupPath = p.join(sourceDir.path, 'image_backup.zip');
      final manifest = await BackupService.createBackup(
        backupPath,
        sourceStorage,
        '2.0.0',
      );

      expect(manifest.imageCount, equals(1));
      expect(imgFile.existsSync(), isTrue);
    });

    test('restored images directory has correct files', () async {
      File(
        p.join(sourceStorage.imagesPath, 'img1.png'),
      ).writeAsBytesSync([1, 2, 3]);
      File(
        p.join(sourceStorage.imagesPath, 'img2.png'),
      ).writeAsBytesSync([4, 5, 6]);

      final backupPath = p.join(sourceDir.path, 'img_restore.zip');
      await BackupService.createBackup(backupPath, sourceStorage, '2.0.0');

      await BackupService.restoreBackup(backupPath, destStorage);

      expect(
        File(p.join(destStorage.imagesPath, 'img1.png')).existsSync(),
        isTrue,
      );
      expect(
        File(p.join(destStorage.imagesPath, 'img2.png')).existsSync(),
        isTrue,
      );
    });

    test('backup includes <id>_thumb.png companion files', () async {
      // Regression: BackupService relies on directory listing to bundle
      // images/, so thumbs produced by ThumbnailService must round-trip.
      final main = File(p.join(sourceStorage.imagesPath, 'item-7.png'))
        ..writeAsBytesSync([10, 20, 30]);
      final thumb = File(p.join(sourceStorage.imagesPath, 'item-7_thumb.png'))
        ..writeAsBytesSync([40, 50, 60, 70]);

      final backupPath = p.join(sourceDir.path, 'thumb_restore.zip');
      final manifest = await BackupService.createBackup(
        backupPath,
        sourceStorage,
        '2.0.0',
      );
      expect(manifest.imageCount, greaterThanOrEqualTo(2));

      await BackupService.restoreBackup(backupPath, destStorage);

      final restoredMain = File(p.join(destStorage.imagesPath, 'item-7.png'));
      final restoredThumb = File(
        p.join(destStorage.imagesPath, 'item-7_thumb.png'),
      );
      expect(restoredMain.existsSync(), isTrue);
      expect(restoredThumb.existsSync(), isTrue);
      expect(restoredMain.readAsBytesSync(), main.readAsBytesSync());
      expect(restoredThumb.readAsBytesSync(), thumb.readAsBytesSync());
    });

    test('backup includes config files', () async {
      File(
        p.join(sourceStorage.configPath, 'app_config.json'),
      ).writeAsStringSync('{"theme":"dark"}');

      final backupPath = p.join(sourceDir.path, 'config_backup.zip');
      await BackupService.createBackup(backupPath, sourceStorage, '2.0.0');

      await BackupService.restoreBackup(backupPath, destStorage);

      final restoredConfig = File(
        p.join(destStorage.configPath, 'app_config.json'),
      );
      expect(restoredConfig.existsSync(), isTrue);
      expect(restoredConfig.readAsStringSync(), equals('{"theme":"dark"}'));
    });

    test(
      'validateBackup returns correct manifest for integration backup',
      () async {
        final service = ClipboardService(repo);
        await service.processText('validate me', ClipboardContentType.text);
        service.dispose();

        final backupPath = p.join(sourceDir.path, 'validate.zip');
        await BackupService.createBackup(
          backupPath,
          sourceStorage,
          '2.5.0',
          itemCount: 1,
          hasPinnedItems: false,
          walCheckpoint: () => repo.walCheckpoint(),
        );

        final manifest = await BackupService.validateBackup(backupPath);
        expect(manifest, isNotNull);
        expect(manifest!.appVersion, equals('2.5.0'));
        expect(manifest.version, equals(BackupManifest.currentVersion));
        expect(manifest.itemCount, equals(1));
      },
    );

    test(
      'hasPinnedItems is true in manifest when pinned items exist',
      () async {
        final service = ClipboardService(repo);
        final item = await service.processText(
          'pinned',
          ClipboardContentType.text,
        );
        await service.updatePin(item!.id, true);
        service.dispose();

        final backupPath = p.join(sourceDir.path, 'pinned_manifest.zip');
        final manifest = await BackupService.createBackup(
          backupPath,
          sourceStorage,
          '2.0.0',
          hasPinnedItems: true,
        );

        expect(manifest.hasPinnedItems, isTrue);
      },
    );
  });
}
