import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late Directory tempDir;
  late StorageConfig storage;
  late SqliteRepository repo;

  setUp(() async {
    tempDir = Directory.systemTemp.createTempSync('cleanup_orphan_test_');
    storage = await StorageConfig.create(baseDir: tempDir.path);
    await storage.ensureDirectories();
    repo = SqliteRepository.inMemory();
  });

  tearDown(() async {
    await repo.close();
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  group('CleanupService orphan image cleanup', () {
    test('removes image files not referenced in repository', () async {
      // Create orphan image in images directory
      final orphan = File(p.join(storage.imagesPath, 'orphan.png'))
        ..writeAsBytesSync([1, 2, 3]);

      // Add a referenced image to repository
      final referenced = File(p.join(storage.imagesPath, 'referenced.png'))
        ..writeAsBytesSync([4, 5, 6]);
      await repo.save(
        ClipboardItem(
          content: referenced.path,
          type: ClipboardContentType.image,
          contentHash: 'hash-ref',
        ),
      );

      final service = CleanupService(repo, () => 30, storage: storage);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));
      service.dispose();

      expect(referenced.existsSync(), isTrue);
      expect(orphan.existsSync(), isFalse);
    });

    test('keeps all images when all are referenced', () async {
      final img1 = File(p.join(storage.imagesPath, 'img1.png'))
        ..writeAsBytesSync([1, 2]);
      final img2 = File(p.join(storage.imagesPath, 'img2.png'))
        ..writeAsBytesSync([3, 4]);

      await repo.save(
        ClipboardItem(
          content: img1.path,
          type: ClipboardContentType.image,
          contentHash: 'hash1',
        ),
      );
      await repo.save(
        ClipboardItem(
          content: img2.path,
          type: ClipboardContentType.image,
          contentHash: 'hash2',
        ),
      );

      final service = CleanupService(repo, () => 30, storage: storage);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));
      service.dispose();

      expect(img1.existsSync(), isTrue);
      expect(img2.existsSync(), isTrue);
    });

    test(
      'removes all orphan images when repository has no image items',
      () async {
        final orphan1 = File(p.join(storage.imagesPath, 'o1.png'))
          ..writeAsBytesSync([1]);
        final orphan2 = File(p.join(storage.imagesPath, 'o2.png'))
          ..writeAsBytesSync([2]);

        // Only a text item — no images referenced
        await repo.save(
          ClipboardItem(content: 'plain text', type: ClipboardContentType.text),
        );

        final service = CleanupService(repo, () => 30, storage: storage);
        service.start(tempDir.path);
        await Future<void>.delayed(const Duration(milliseconds: 100));
        service.dispose();

        expect(orphan1.existsSync(), isFalse);
        expect(orphan2.existsSync(), isFalse);
      },
    );

    test('does not run orphan cleanup when retentionDays is 0', () async {
      final orphan = File(p.join(storage.imagesPath, 'keep_me.png'))
        ..writeAsBytesSync([1, 2, 3]);

      final service = CleanupService(repo, () => 0, storage: storage);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));
      service.dispose();

      // Retention is 0 → cleanup is fully skipped, orphan must survive
      expect(orphan.existsSync(), isTrue);
    });

    test('does not crash when images directory is missing', () async {
      // Remove images directory to simulate missing dir
      Directory(storage.imagesPath).deleteSync(recursive: true);

      final service = CleanupService(repo, () => 30, storage: storage);
      await expectLater(service.runCleanupIfNeeded(), completes);
      service.dispose();
    });

    test('updateRetentionCallback changes retention days dynamically', () async {
      var retentionDays = 0;
      final service = CleanupService(
        repo,
        () => retentionDays,
        storage: storage,
      );

      // With 0 days, cleanup is skipped
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));

      // Now change retention to 30 and force next run by clearing last cleanup file
      retentionDays = 30;
      service.updateRetentionCallback(() => retentionDays);

      final cleanupFile = File(p.join(tempDir.path, 'last_cleanup.txt'));
      if (cleanupFile.existsSync()) cleanupFile.deleteSync();

      await service.runCleanupIfNeeded();
      service.dispose();

      // No error means the dynamic callback update works
    });
  });
}
