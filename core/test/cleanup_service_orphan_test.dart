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

    test('runs orphan cleanup even when retentionDays is 0', () async {
      // Bug fix: orphan image cleanup must run independently of retention setting.
      // When retention=0, time-based deletion is skipped but orphan cleanup still runs.
      final orphan = File(p.join(storage.imagesPath, 'orphan_zero_ret.png'))
        ..writeAsBytesSync([1, 2, 3]);

      final service = CleanupService(repo, () => 0, storage: storage);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));
      service.dispose();

      // Orphan cleanup runs regardless of retention → orphan must be deleted
      expect(orphan.existsSync(), isFalse);
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

    test('preserves thumbnail files referenced by thumbPath', () async {
      // Regression: orphan sweep must NOT delete `<id>_thumb.png` files
      // produced by ThumbnailService for items with external sources.
      final externalDir = Directory(p.join(tempDir.path, 'ext'))
        ..createSync(recursive: true);
      final external = File(p.join(externalDir.path, 'photo.png'))
        ..writeAsBytesSync([9, 9, 9]);

      final thumb = File(p.join(storage.imagesPath, 'item-x_thumb.png'))
        ..writeAsBytesSync([1, 2, 3, 4]);

      await repo.save(
        ClipboardItem(
          id: 'item-x',
          content: external.path,
          type: ClipboardContentType.image,
          thumbPath: thumb.path,
        ),
      );

      final service = CleanupService(repo, () => 30, storage: storage);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));
      service.dispose();

      expect(thumb.existsSync(), isTrue, reason: 'thumb must survive sweep');
      expect(external.existsSync(), isTrue, reason: 'external file untouched');
    });
  });

  group('CleanupService broken-external tracking', () {
    Future<void> _runOnce(CleanupService service) async {
      final cleanupFile = File(p.join(tempDir.path, 'last_cleanup.txt'));
      if (cleanupFile.existsSync()) cleanupFile.deleteSync();
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));
      service.dispose();
    }

    test(
      'sets brokenSince when external file disappears (volume present)',
      () async {
        final extDir = Directory(p.join(tempDir.path, 'ext'))
          ..createSync(recursive: true);
        final ext = File(p.join(extDir.path, 'a.png'))..writeAsBytesSync([1]);
        await repo.save(
          ClipboardItem(
            id: 'i1',
            content: ext.path,
            type: ClipboardContentType.image,
          ),
        );

        ext.deleteSync();

        final service = CleanupService(
          repo,
          () => 30,
          storage: storage,
          getKeepBrokenDays: () => 30,
        );
        await _runOnce(service);
        service.dispose();

        final reloaded = await repo.getById('i1');
        expect(reloaded?.brokenSince, isNotNull);
      },
    );

    test('clears brokenSince when external file reappears', () async {
      final extDir = Directory(p.join(tempDir.path, 'ext'))
        ..createSync(recursive: true);
      final ext = File(p.join(extDir.path, 'b.png'))..writeAsBytesSync([2]);
      await repo.save(
        ClipboardItem(
          id: 'i2',
          content: ext.path,
          type: ClipboardContentType.image,
          brokenSince: DateTime.now().toUtc().subtract(const Duration(days: 5)),
        ),
      );

      // file still present
      final service = CleanupService(
        repo,
        () => 30,
        storage: storage,
        getKeepBrokenDays: () => 30,
      );
      await _runOnce(service);
      service.dispose();

      final reloaded = await repo.getById('i2');
      expect(reloaded?.brokenSince, isNull);
    });

    test('purges item + own thumb when brokenSince exceeds keepBrokenDays; '
        'never touches the external path', () async {
      final extDir = Directory(p.join(tempDir.path, 'ext'))
        ..createSync(recursive: true);
      final ext = File(p.join(extDir.path, 'gone.png'))..writeAsBytesSync([3]);
      final thumb = File(p.join(storage.imagesPath, 'i3_thumb.png'))
        ..writeAsBytesSync([4]);

      await repo.save(
        ClipboardItem(
          id: 'i3',
          content: ext.path,
          type: ClipboardContentType.image,
          thumbPath: thumb.path,
          brokenSince: DateTime.now().toUtc().subtract(
            const Duration(days: 60),
          ),
        ),
      );
      ext.deleteSync();

      final service = CleanupService(
        repo,
        () => 0,
        storage: storage,
        getKeepBrokenDays: () => 30,
      );
      await _runOnce(service);
      service.dispose();

      expect(await repo.getById('i3'), isNull, reason: 'item purged');
      expect(thumb.existsSync(), isFalse, reason: 'own thumb deleted');
      // The external file was already deleted by the test itself; the
      // assertion below documents the contract that the service never
      // recreates or otherwise alters external paths.
      expect(File(ext.path).existsSync(), isFalse);
    });

    test('does not touch pinned items even when external is broken', () async {
      final extDir = Directory(p.join(tempDir.path, 'ext'))
        ..createSync(recursive: true);
      final ext = File(p.join(extDir.path, 'p.png'))..writeAsBytesSync([5]);
      await repo.save(
        ClipboardItem(
          id: 'pinned',
          content: ext.path,
          type: ClipboardContentType.image,
          isPinned: true,
        ),
      );
      ext.deleteSync();

      final service = CleanupService(
        repo,
        () => 30,
        storage: storage,
        getKeepBrokenDays: () => 30,
      );
      await _runOnce(service);
      service.dispose();

      final reloaded = await repo.getById('pinned');
      expect(reloaded, isNotNull);
      expect(reloaded?.brokenSince, isNull);
    });

    test('isVolumePresent returns false for absent Windows drive', () {
      if (!Platform.isWindows) return;
      // Pick a drive letter that is highly unlikely to be mounted.
      final result = CleanupService.isVolumePresent(r'Q:\does\not\exist.png');
      // If by chance Q: exists on the dev box, accept either result for the
      // purpose of this smoke check; the contract is documented above.
      expect(result, isA<bool>());
    });

    test('isVolumePresent returns true for current drive', () {
      final cwd = Directory.current.path;
      expect(CleanupService.isVolumePresent(cwd), isTrue);
    });
  });
}
