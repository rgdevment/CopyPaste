import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';
import 'package:core/repository/i_clipboard_repository.dart';

class _FailingRepo implements IClipboardRepository {
  bool failGetImagePaths = false;

  @override
  Future<int> clearOldItems(int days, {bool excludePinned = true}) =>
      Future.error(Exception('forced clearOldItems error'));

  @override
  Future<List<String>> getImagePaths() {
    if (failGetImagePaths) {
      return Future.error(Exception('forced getImagePaths error'));
    }
    return Future.value([]);
  }

  @override
  Future<void> save(ClipboardItem item) => Future.value();
  @override
  Future<void> update(ClipboardItem item) => Future.value();
  @override
  Future<ClipboardItem?> getById(String id) => Future.value(null);
  @override
  Future<ClipboardItem?> getLatest() => Future.value(null);
  @override
  Future<ClipboardItem?> findByContentAndType(
    String content,
    ClipboardContentType type,
  ) => Future.value(null);
  @override
  Future<ClipboardItem?> findByContentHash(String hash) => Future.value(null);
  @override
  Future<List<ClipboardItem>> getAll() => Future.value([]);
  @override
  Future<void> delete(String id) => Future.value();
  @override
  Future<int> deleteAllUnpinned() => Future.value(0);
  @override
  Future<int> count() => Future.value(0);
  @override
  Future<List<ClipboardItem>> search(
    String q, {
    int limit = 50,
    int skip = 0,
  }) => Future.value([]);
  @override
  Future<List<ClipboardItem>> searchAdvanced({
    String? query,
    List<ClipboardContentType>? types,
    List<CardColor>? colors,
    bool? isPinned,
    required int limit,
    required int skip,
  }) => Future.value([]);
  @override
  Future<void> walCheckpoint() => Future.value();
  @override
  Future<void> close() => Future.value();
}

void main() {
  late SqliteRepository repo;
  late Directory tempDir;

  setUp(() {
    repo = SqliteRepository.inMemory();
    tempDir = Directory.systemTemp.createTempSync('cleanup_test_');
  });

  tearDown(() async {
    await repo.close();
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  group('CleanupService', () {
    test('runCleanupIfNeeded via start() clears old items', () async {
      await repo.save(
        ClipboardItem(
          content: 'old',
          type: ClipboardContentType.text,
          createdAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
          modifiedAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
        ),
      );

      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final count = await repo.count();
      expect(count, equals(0));
    });

    test('skips cleanup when same day as last run', () async {
      final file = File('${tempDir.path}/last_cleanup.txt');
      file.writeAsStringSync(DateTime.now().toUtc().toIso8601String());

      await repo.save(
        ClipboardItem(
          content: 'old',
          type: ClipboardContentType.text,
          createdAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
          modifiedAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
        ),
      );

      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      // Item should still be there because cleanup was skipped
      final count = await repo.count();
      expect(count, equals(1));
    });

    test('skips cleanup when retentionDays is 0', () async {
      await repo.save(
        ClipboardItem(
          content: 'old',
          type: ClipboardContentType.text,
          createdAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
          modifiedAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
        ),
      );

      final service = CleanupService(repo, () => 0);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final count = await repo.count();
      expect(count, equals(1));
    });

    test('skips cleanup when retentionDays is negative', () async {
      await repo.save(
        ClipboardItem(
          content: 'item',
          type: ClipboardContentType.text,
          createdAt: DateTime.now().toUtc().subtract(const Duration(days: 100)),
          modifiedAt: DateTime.now().toUtc().subtract(
            const Duration(days: 100),
          ),
        ),
      );

      final service = CleanupService(repo, () => -1);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final count = await repo.count();
      expect(count, equals(1));
    });

    test('preserves pinned items during cleanup', () async {
      final pinned = ClipboardItem(
        content: 'pinned old',
        type: ClipboardContentType.text,
        isPinned: true,
        createdAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
        modifiedAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
      );
      await repo.save(pinned);

      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final found = await repo.getById(pinned.id);
      expect(found, isNotNull);
    });

    test('writes last cleanup date to file after running', () async {
      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final file = File('${tempDir.path}/last_cleanup.txt');
      expect(file.existsSync(), isTrue);

      final content = file.readAsStringSync().trim();
      final parsed = DateTime.tryParse(content);
      expect(parsed, isNotNull);
      // Should be today's date
      final now = DateTime.now().toUtc();
      expect(parsed!.year, equals(now.year));
      expect(parsed.month, equals(now.month));
      expect(parsed.day, equals(now.day));
    });

    test('runs cleanup with previous-day date file', () async {
      final yesterday = DateTime.now().toUtc().subtract(
        const Duration(days: 1),
      );
      final file = File('${tempDir.path}/last_cleanup.txt');
      file.writeAsStringSync(yesterday.toIso8601String());

      await repo.save(
        ClipboardItem(
          content: 'old',
          type: ClipboardContentType.text,
          createdAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
          modifiedAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
        ),
      );

      final service = CleanupService(repo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();

      final count = await repo.count();
      expect(count, equals(0));
    });

    test('dispose prevents further cleanup', () async {
      final service = CleanupService(repo, () => 30);
      service.dispose();
      // After dispose, runCleanupIfNeeded is a no-op

      await repo.save(
        ClipboardItem(
          content: 'old',
          type: ClipboardContentType.text,
          createdAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
          modifiedAt: DateTime.now().toUtc().subtract(const Duration(days: 40)),
        ),
      );

      await service.runCleanupIfNeeded();

      final count = await repo.count();
      expect(count, equals(1)); // item was NOT deleted
    });

    test('does not crash on missing base dir', () async {
      final service = CleanupService(repo, () => 30);
      await expectLater(service.runCleanupIfNeeded(), completes);
      service.dispose();
    });

    test('logs error when clearOldItems throws', () async {
      final failingRepo = _FailingRepo();
      final service = CleanupService(failingRepo, () => 30);
      service.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      service.dispose();
    });

    test('logs error when getImagePaths throws during orphan cleanup', () async {
      final storage = await StorageConfig.create(baseDir: tempDir.path);
      await storage.ensureDirectories();

      final repoWithPassingClear = SqliteRepository.inMemory();
      final hybridRepo = _HybridRepo(repoWithPassingClear);
      final hybridService = CleanupService(
        hybridRepo,
        () => 30,
        storage: storage,
      );

      final yesterday = DateTime.now().toUtc().subtract(
        const Duration(days: 1),
      );
      File(
        '${tempDir.path}/last_cleanup.txt',
      ).writeAsStringSync(yesterday.toIso8601String());

      hybridService.start(tempDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 50));
      hybridService.dispose();
      await repoWithPassingClear.close();
    });
  });
}

class _HybridRepo implements IClipboardRepository {
  _HybridRepo(this._inner);
  final IClipboardRepository _inner;

  @override
  Future<List<String>> getImagePaths() =>
      Future.error(Exception('forced getImagePaths error'));

  @override
  Future<int> clearOldItems(int days, {bool excludePinned = true}) =>
      _inner.clearOldItems(days, excludePinned: excludePinned);
  @override
  Future<void> save(ClipboardItem item) => _inner.save(item);
  @override
  Future<void> update(ClipboardItem item) => _inner.update(item);
  @override
  Future<ClipboardItem?> getById(String id) => _inner.getById(id);
  @override
  Future<ClipboardItem?> getLatest() => _inner.getLatest();
  @override
  Future<ClipboardItem?> findByContentAndType(
    String content,
    ClipboardContentType type,
  ) => _inner.findByContentAndType(content, type);
  @override
  Future<ClipboardItem?> findByContentHash(String hash) =>
      _inner.findByContentHash(hash);
  @override
  Future<List<ClipboardItem>> getAll() => _inner.getAll();
  @override
  Future<void> delete(String id) => _inner.delete(id);
  @override
  Future<int> deleteAllUnpinned() => _inner.deleteAllUnpinned();
  @override
  Future<int> count() => _inner.count();
  @override
  Future<List<ClipboardItem>> search(
    String q, {
    int limit = 50,
    int skip = 0,
  }) => _inner.search(q, limit: limit, skip: skip);
  @override
  Future<List<ClipboardItem>> searchAdvanced({
    String? query,
    List<ClipboardContentType>? types,
    List<CardColor>? colors,
    bool? isPinned,
    required int limit,
    required int skip,
  }) => _inner.searchAdvanced(
    query: query,
    types: types,
    colors: colors,
    isPinned: isPinned,
    limit: limit,
    skip: skip,
  );
  @override
  Future<void> walCheckpoint() => _inner.walCheckpoint();
  @override
  Future<void> close() => _inner.close();
}
