import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late Directory tempDir;
  late StorageConfig storage;
  late SqliteRepository repo;

  setUp(() async {
    tempDir = Directory.systemTemp.createTempSync('cleanup_quota_test_');
    storage = await StorageConfig.create(baseDir: tempDir.path);
    await storage.ensureDirectories();
    repo = SqliteRepository.inMemory();
  });

  tearDown(() async {
    await repo.close();
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  Future<File> writeOwned(String id, int sizeBytes) async {
    final f = File(p.join(storage.imagesPath, '$id.png'));
    f.writeAsBytesSync(List<int>.filled(sizeBytes, 0xAA));
    return f;
  }

  Future<ClipboardItem> saveItem({
    required String id,
    required String filePath,
    required DateTime createdAt,
    bool isPinned = false,
    String? thumbPath,
  }) async {
    final item = ClipboardItem(
      id: id,
      content: filePath,
      type: ClipboardContentType.image,
      contentHash: 'hash-$id',
      createdAt: createdAt,
      modifiedAt: createdAt,
      isPinned: isPinned,
      thumbPath: thumbPath,
    );
    await repo.save(item);
    return item;
  }

  Future<void> runCleanup(CleanupService service) async {
    service.start(tempDir.path);
    // Give the async chain (`runCleanupIfNeeded`) time to drain.
    await Future<void>.delayed(const Duration(milliseconds: 200));
    service.dispose();
  }

  group('CleanupService images quota (LRU purge)', () {
    test('does nothing when quotaMB <= 0', () async {
      final f1 = await writeOwned('a', 2 * 1024 * 1024);
      await saveItem(
        id: 'a',
        filePath: f1.path,
        createdAt: DateTime.utc(2024, 1, 1),
      );

      final service = CleanupService(
        repo,
        () => 0,
        storage: storage,
        getImagesQuotaMB: () => 0,
      );
      await runCleanup(service);

      expect(f1.existsSync(), isTrue);
      expect(await repo.count(), 1);
    });

    test('purges oldest unpinned items until under the cap', () async {
      // 3 items, ~600 KB each, cap = 1 MB → must drop the two oldest.
      final f1 = await writeOwned('old', 600 * 1024);
      final f2 = await writeOwned('mid', 600 * 1024);
      final f3 = await writeOwned('new', 600 * 1024);
      await saveItem(
        id: 'old',
        filePath: f1.path,
        createdAt: DateTime.utc(2024, 1, 1),
      );
      await saveItem(
        id: 'mid',
        filePath: f2.path,
        createdAt: DateTime.utc(2024, 6, 1),
      );
      await saveItem(
        id: 'new',
        filePath: f3.path,
        createdAt: DateTime.utc(2024, 12, 1),
      );

      final service = CleanupService(
        repo,
        () => 0,
        storage: storage,
        getImagesQuotaMB: () => 1,
      );
      await runCleanup(service);

      expect(f1.existsSync(), isFalse, reason: 'oldest must be purged');
      expect(f2.existsSync(), isFalse, reason: 'mid must be purged');
      expect(f3.existsSync(), isTrue, reason: 'newest must survive');
      expect(await repo.getById('old'), isNull);
      expect(await repo.getById('mid'), isNull);
      expect(await repo.getById('new'), isNotNull);
    });

    test('skips pinned items even when oldest', () async {
      final f1 = await writeOwned('pinned', 600 * 1024);
      final f2 = await writeOwned('mid', 600 * 1024);
      final f3 = await writeOwned('new', 600 * 1024);
      await saveItem(
        id: 'pinned',
        filePath: f1.path,
        createdAt: DateTime.utc(2024, 1, 1),
        isPinned: true,
      );
      await saveItem(
        id: 'mid',
        filePath: f2.path,
        createdAt: DateTime.utc(2024, 6, 1),
      );
      await saveItem(
        id: 'new',
        filePath: f3.path,
        createdAt: DateTime.utc(2024, 12, 1),
      );

      final service = CleanupService(
        repo,
        () => 0,
        storage: storage,
        getImagesQuotaMB: () => 1,
      );
      await runCleanup(service);

      expect(f1.existsSync(), isTrue, reason: 'pinned must survive');
      expect(f2.existsSync(), isFalse, reason: 'unpinned mid purged');
      expect(await repo.getById('pinned'), isNotNull);
    });

    test('also deletes the per-item thumbnail when present', () async {
      final f1 = await writeOwned('a', 800 * 1024);
      final thumb = File(p.join(storage.imagesPath, 'a_thumb.png'))
        ..writeAsBytesSync(List<int>.filled(300 * 1024, 0xBB));
      await saveItem(
        id: 'a',
        filePath: f1.path,
        createdAt: DateTime.utc(2024, 1, 1),
        thumbPath: thumb.path,
      );
      // newer item to keep
      final f2 = await writeOwned('b', 200 * 1024);
      await saveItem(
        id: 'b',
        filePath: f2.path,
        createdAt: DateTime.utc(2024, 12, 1),
      );

      final service = CleanupService(
        repo,
        () => 0,
        storage: storage,
        getImagesQuotaMB: () => 1,
      );
      await runCleanup(service);

      expect(f1.existsSync(), isFalse);
      expect(thumb.existsSync(), isFalse, reason: 'thumb must be deleted too');
      expect(f2.existsSync(), isTrue);
    });

    test('never touches external paths referenced by items', () async {
      // External "user file" outside images/.
      final external = File(p.join(tempDir.path, 'user_photo.png'))
        ..writeAsBytesSync(List<int>.filled(900 * 1024, 0xCC));
      await saveItem(
        id: 'ext',
        filePath: external.path,
        createdAt: DateTime.utc(2024, 1, 1),
      );
      // Plus an oversized owned snippet.
      final owned = await writeOwned('owned', 1500 * 1024);
      await saveItem(
        id: 'owned',
        filePath: owned.path,
        createdAt: DateTime.utc(2024, 6, 1),
      );

      final service = CleanupService(
        repo,
        () => 0,
        storage: storage,
        getImagesQuotaMB: () => 1,
      );
      await runCleanup(service);

      expect(
        external.existsSync(),
        isTrue,
        reason: 'external user file must never be deleted',
      );
      // owned snippet purged because total > 1MB and it is the oldest with
      // owned bytes inside images/.
      expect(owned.existsSync(), isFalse);
    });

    test('updateImagesQuotaCallback swaps the limit live', () async {
      final f1 = await writeOwned('a', 600 * 1024);
      final f2 = await writeOwned('b', 600 * 1024);
      await saveItem(
        id: 'a',
        filePath: f1.path,
        createdAt: DateTime.utc(2024, 1, 1),
      );
      await saveItem(
        id: 'b',
        filePath: f2.path,
        createdAt: DateTime.utc(2024, 12, 1),
      );

      var quota = 0; // initially disabled
      final service = CleanupService(
        repo,
        () => 0,
        storage: storage,
        getImagesQuotaMB: () => quota,
      );
      await runCleanup(service);
      expect(f1.existsSync(), isTrue);
      expect(f2.existsSync(), isTrue);

      // Activate the cap and run again on a fresh service so the
      // last-cleanup gate doesn't skip the run.
      quota = 1;
      final marker = File(p.join(tempDir.path, 'last_cleanup.txt'));
      if (marker.existsSync()) marker.deleteSync();
      final service2 = CleanupService(
        repo,
        () => 0,
        storage: storage,
        getImagesQuotaMB: () => quota,
      );
      await runCleanup(service2);
      expect(f1.existsSync(), isFalse);
      expect(f2.existsSync(), isTrue);
    });
  });
}
