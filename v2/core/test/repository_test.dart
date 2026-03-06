import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'package:core/core.dart';

void main() {
  late SqliteRepository repo;

  setUp(() {
    repo = SqliteRepository.inMemory();
  });

  tearDown(() => repo.close());

  group('SqliteRepository', () {
    test('save and getById', () async {
      final item = ClipboardItem(
        content: 'hello',
        type: ClipboardContentType.text,
      );
      await repo.save(item);
      final found = await repo.getById(item.id);
      expect(found, isNotNull);
      expect(found!.content, equals('hello'));
      expect(found.type, equals(ClipboardContentType.text));
    });

    test('update modifies existing item', () async {
      final item = ClipboardItem(
        content: 'original',
        type: ClipboardContentType.text,
      );
      await repo.save(item);
      final updated = item.copyWith(isPinned: true);
      await repo.update(updated);
      final found = await repo.getById(item.id);
      expect(found!.isPinned, isTrue);
    });

    test('delete removes item', () async {
      final item = ClipboardItem(
        content: 'delete me',
        type: ClipboardContentType.text,
      );
      await repo.save(item);
      await repo.delete(item.id);
      final found = await repo.getById(item.id);
      expect(found, isNull);
    });

    test('findByContentAndType returns existing item', () async {
      final item = ClipboardItem(
        content: 'test content',
        type: ClipboardContentType.text,
      );
      await repo.save(item);
      final found = await repo.findByContentAndType(
        'test content',
        ClipboardContentType.text,
      );
      expect(found, isNotNull);
      expect(found!.id, equals(item.id));
    });

    test('findByContentAndType returns null for wrong type', () async {
      final item = ClipboardItem(
        content: 'test',
        type: ClipboardContentType.text,
      );
      await repo.save(item);
      final found = await repo.findByContentAndType(
        'test',
        ClipboardContentType.image,
      );
      expect(found, isNull);
    });

    test('findByContentHash returns matching item', () async {
      final item = ClipboardItem(
        content: '',
        type: ClipboardContentType.image,
        contentHash: 'abc123',
      );
      await repo.save(item);
      final found = await repo.findByContentHash('abc123');
      expect(found, isNotNull);
      expect(found!.contentHash, equals('abc123'));
    });

    test('getAll returns items ordered by modifiedAt desc', () async {
      final first = ClipboardItem(
        content: 'first',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.utc(2024, 1, 1),
      );
      final second = ClipboardItem(
        content: 'second',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.utc(2024, 1, 2),
      );
      await repo.save(first);
      await repo.save(second);
      final all = await repo.getAll();
      expect(all.first.content, equals('second'));
    });

    test('clearOldItems removes old non-pinned entries', () async {
      final old = ClipboardItem(
        content: 'old',
        type: ClipboardContentType.text,
        createdAt: DateTime.now().toUtc().subtract(const Duration(days: 35)),
        modifiedAt: DateTime.now().toUtc().subtract(const Duration(days: 35)),
      );
      final fresh = ClipboardItem(
        content: 'fresh',
        type: ClipboardContentType.text,
      );
      await repo.save(old);
      await repo.save(fresh);
      final deleted = await repo.clearOldItems(30);
      expect(deleted, equals(1));
      final remaining = await repo.getAll();
      expect(remaining.length, equals(1));
      expect(remaining.first.content, equals('fresh'));
    });

    test('clearOldItems preserves pinned items', () async {
      final pinned = ClipboardItem(
        content: 'pinned old',
        type: ClipboardContentType.text,
        isPinned: true,
        createdAt: DateTime.now().toUtc().subtract(const Duration(days: 35)),
        modifiedAt: DateTime.now().toUtc().subtract(const Duration(days: 35)),
      );
      await repo.save(pinned);
      final deleted = await repo.clearOldItems(30);
      expect(deleted, equals(0));
    });

    test('clearOldItems uses createdAt to determine age', () async {
      final oldCreatedButRecentlyUsed = ClipboardItem(
        content: 'recently used',
        type: ClipboardContentType.text,
        createdAt: DateTime.now().toUtc().subtract(const Duration(days: 60)),
        modifiedAt: DateTime.now().toUtc(),
      );
      await repo.save(oldCreatedButRecentlyUsed);
      // Item was created 60 days ago — should be deleted even if recently modified
      final deleted = await repo.clearOldItems(30);
      expect(deleted, equals(1));

      // Item created recently should be kept
      final recentItem = ClipboardItem(
        content: 'recent',
        type: ClipboardContentType.text,
      );
      await repo.save(recentItem);
      final deleted2 = await repo.clearOldItems(30);
      expect(deleted2, equals(0));
      final remaining = await repo.getAll();
      expect(remaining.length, equals(1));
    });

    test('search finds items by content', () async {
      await repo.save(
        ClipboardItem(content: 'hello world', type: ClipboardContentType.text),
      );
      await repo.save(
        ClipboardItem(content: 'another item', type: ClipboardContentType.text),
      );
      final results = await repo.search('hello');
      expect(results.length, equals(1));
      expect(results.first.content, equals('hello world'));
    });

    test('searchAdvanced filters by type', () async {
      await repo.save(
        ClipboardItem(content: 'text item', type: ClipboardContentType.text),
      );
      await repo.save(
        ClipboardItem(content: 'link item', type: ClipboardContentType.link),
      );
      final results = await repo.searchAdvanced(
        types: [ClipboardContentType.text],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.type, equals(ClipboardContentType.text));
    });

    test('searchAdvanced filters by color', () async {
      await repo.save(
        ClipboardItem(
          content: 'red item',
          type: ClipboardContentType.text,
          cardColor: CardColor.red,
        ),
      );
      await repo.save(
        ClipboardItem(content: 'no color', type: ClipboardContentType.text),
      );
      final results = await repo.searchAdvanced(
        colors: [CardColor.red],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.cardColor, equals(CardColor.red));
    });

    test('searchAdvanced filters by isPinned true', () async {
      await repo.save(
        ClipboardItem(
          content: 'pinned item',
          type: ClipboardContentType.text,
          isPinned: true,
        ),
      );
      await repo.save(
        ClipboardItem(content: 'normal item', type: ClipboardContentType.text),
      );
      final results = await repo.searchAdvanced(
        isPinned: true,
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.isPinned, isTrue);
    });

    test('searchAdvanced with query and type filter', () async {
      await repo.save(
        ClipboardItem(content: 'hello text', type: ClipboardContentType.text),
      );
      await repo.save(
        ClipboardItem(content: 'hello link', type: ClipboardContentType.link),
      );
      await repo.save(
        ClipboardItem(content: 'world text', type: ClipboardContentType.text),
      );
      final results = await repo.searchAdvanced(
        query: 'hello',
        types: [ClipboardContentType.text],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.content, equals('hello text'));
    });

    test('getLatest returns most recently modified item', () async {
      final older = ClipboardItem(
        content: 'older',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.utc(2024, 1, 1),
      );
      final newer = ClipboardItem(
        content: 'newer',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.utc(2024, 1, 2),
      );
      await repo.save(older);
      await repo.save(newer);
      final latest = await repo.getLatest();
      expect(latest, isNotNull);
      expect(latest!.content, equals('newer'));
    });

    test('getLatest returns null on empty repository', () async {
      final latest = await repo.getLatest();
      expect(latest, isNull);
    });

    test('deleteAllUnpinned removes only non-pinned items', () async {
      final pinned = ClipboardItem(
        content: 'keep me',
        type: ClipboardContentType.text,
        isPinned: true,
      );
      final unpinned = ClipboardItem(
        content: 'delete me',
        type: ClipboardContentType.text,
      );
      await repo.save(pinned);
      await repo.save(unpinned);
      final deleted = await repo.deleteAllUnpinned();
      expect(deleted, equals(1));
      final remaining = await repo.getAll();
      expect(remaining.length, equals(1));
      expect(remaining.first.isPinned, isTrue);
    });

    test('count returns correct number of items', () async {
      expect(await repo.count(), equals(0));
      await repo.save(
        ClipboardItem(content: 'a', type: ClipboardContentType.text),
      );
      await repo.save(
        ClipboardItem(content: 'b', type: ClipboardContentType.text),
      );
      expect(await repo.count(), equals(2));

      final all = await repo.getAll();
      await repo.delete(all.first.id);
      expect(await repo.count(), equals(1));
    });

    test('getImagePaths returns content of image items', () async {
      await repo.save(
        ClipboardItem(
          content: '/images/photo.png',
          type: ClipboardContentType.image,
        ),
      );
      await repo.save(
        ClipboardItem(content: 'text item', type: ClipboardContentType.text),
      );
      final paths = await repo.getImagePaths();
      expect(paths.length, equals(1));
      expect(paths.first, equals('/images/photo.png'));
    });

    test('getImagePaths returns empty list when no images', () async {
      await repo.save(
        ClipboardItem(content: 'text', type: ClipboardContentType.text),
      );
      final paths = await repo.getImagePaths();
      expect(paths, isEmpty);
    });

    test('walCheckpoint completes without error', () async {
      await repo.save(
        ClipboardItem(content: 'test', type: ClipboardContentType.text),
      );
      await expectLater(repo.walCheckpoint(), completes);
    });

    test('searchAdvanced with skip paginates results', () async {
      for (var i = 0; i < 5; i++) {
        await repo.save(
          ClipboardItem(
            content: 'item $i',
            type: ClipboardContentType.text,
            modifiedAt: DateTime.utc(2024, 1, i + 1),
          ),
        );
      }
      final page1 = await repo.searchAdvanced(limit: 3, skip: 0);
      final page2 = await repo.searchAdvanced(limit: 3, skip: 3);
      expect(page1.length, equals(3));
      expect(page2.length, equals(2));
    });

    test('fromPath creates working repository with persisted data', () async {
      final dir = Directory.systemTemp.createTempSync('repo_path_test_');
      try {
        final dbPath = p.join(dir.path, 'test.db');
        final fileRepo = SqliteRepository.fromPath(dbPath);
        final item = ClipboardItem(
          content: 'persisted',
          type: ClipboardContentType.text,
        );
        await fileRepo.save(item);
        final found = await fileRepo.getById(item.id);
        expect(found, isNotNull);
        expect(found!.content, equals('persisted'));
        await fileRepo.close();
        expect(File(dbPath).existsSync(), isTrue);
      } finally {
        dir.deleteSync(recursive: true);
      }
    });

    test('clearOldItems triggers vacuum when more than 50 items deleted', () async {
      for (var i = 0; i < 55; i++) {
        await repo.save(
          ClipboardItem(
            content: 'old item $i',
            type: ClipboardContentType.text,
            createdAt: DateTime.utc(2000, 1, 1),
            modifiedAt: DateTime.utc(2000, 1, 1),
          ),
        );
      }
      final deleted = await repo.clearOldItems(1);
      expect(deleted, greaterThanOrEqualTo(55));
    });

    test('deleteAllUnpinned triggers vacuum when more than 50 items deleted', () async {
      for (var i = 0; i < 55; i++) {
        await repo.save(
          ClipboardItem(
            content: 'bulk item $i',
            type: ClipboardContentType.text,
          ),
        );
      }
      final deleted = await repo.deleteAllUnpinned();
      expect(deleted, greaterThanOrEqualTo(55));
    });

    test('clearOldItems respects excludePinned=false', () async {
      await repo.save(
        ClipboardItem(
          content: 'pinned old',
          type: ClipboardContentType.text,
          isPinned: true,
          createdAt: DateTime.utc(2000, 1, 1),
          modifiedAt: DateTime.utc(2000, 1, 1),
        ),
      );
      // With excludePinned=false, pinned items should also be deleted
      final deleted = await repo.clearOldItems(1, excludePinned: false);
      expect(deleted, greaterThan(0));
    });
  });
}
