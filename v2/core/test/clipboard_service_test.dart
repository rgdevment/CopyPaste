import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  late SqliteRepository repo;
  late ClipboardService service;

  setUp(() {
    repo = SqliteRepository.inMemory();
    service = ClipboardService(repo);
  });

  tearDown(() async {
    service.dispose();
    await repo.close();
  });

  group('ClipboardService.processText', () {
    test('saves new item and emits onItemAdded', () async {
      ClipboardItem? emitted;
      service.onItemAdded.listen((item) => emitted = item);

      final result = await service.processText(
        'hello',
        ClipboardContentType.text,
      );
      await Future<void>.delayed(Duration.zero);

      expect(result, isNotNull);
      expect(result!.content, equals('hello'));
      expect(emitted?.content, equals('hello'));
    });

    test('reactivates existing item and emits onItemReactivated', () async {
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      final first = await service.processText('dup', ClipboardContentType.text);
      expect(first, isNotNull);

      final second = await service.processText('dup', ClipboardContentType.text);
      await Future<void>.delayed(Duration.zero);

      expect(second, isNotNull);
      expect(reactivated?.content, equals('dup'));
    });

    test('returns null when inside paste ignore window', () async {
      service.pasteIgnoreWindowMs = 60000;
      await service.notifyPasteInitiated('any-id');

      final result = await service.processText('ignored', ClipboardContentType.text);
      expect(result, isNull);
    });

    test('saves item with source and rtf/html metadata', () async {
      final result = await service.processText(
        'data',
        ClipboardContentType.link,
        source: 'Chrome',
        rtfBytes: [72, 69, 76, 76, 79],
        htmlBytes: [60, 104, 116, 109, 108, 62],
      );
      expect(result!.appSource, equals('Chrome'));
      expect(result.metadata, isNotNull);
      expect(result.metadata, contains('rtf'));
      expect(result.metadata, contains('html'));
    });

    test('saves item without metadata when no rtf/html provided', () async {
      final result = await service.processText(
        'plain',
        ClipboardContentType.text,
      );
      expect(result!.metadata, isNull);
    });
  });

  group('ClipboardService.processImage', () {
    test('saves new image item by contentHash', () async {
      final result = await service.processImage(
        'hash-abc',
        imagePath: '/tmp/image.png',
      );
      expect(result, isNotNull);
      expect(result!.contentHash, equals('hash-abc'));
      expect(result.type, equals(ClipboardContentType.image));
      expect(result.content, equals('/tmp/image.png'));
    });

    test('reactivates duplicate image by hash', () async {
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      await service.processImage('hash-dup');
      await service.processImage('hash-dup');
      await Future<void>.delayed(Duration.zero);

      expect(reactivated, isNotNull);
    });
  });

  group('ClipboardService.recordPaste', () {
    test('increments pasteCount and returns updated item', () async {
      final item = await service.processText('paste me', ClipboardContentType.text);
      expect(item, isNotNull);

      final updated = await service.recordPaste(item!.id);

      expect(updated, isNotNull);
      expect(updated!.pasteCount, equals(1));
      final stored = await repo.getById(item.id);
      expect(stored!.pasteCount, equals(1));
    });

    test('returns null for unknown id', () async {
      final result = await service.recordPaste('nonexistent-id');
      expect(result, isNull);
    });
  });

  group('ClipboardService.processFiles', () {
    test('saves file list with metadata', () async {
      ClipboardItem? emitted;
      service.onItemAdded.listen((item) => emitted = item);

      final result = await service.processFiles(
        ['C:\\docs\\file1.txt', 'C:\\docs\\file2.txt'],
        ClipboardContentType.file,
        source: 'explorer',
      );
      await Future<void>.delayed(Duration.zero);

      expect(result, isNotNull);
      expect(result!.content, contains('file1.txt'));
      expect(result.content, contains('file2.txt'));
      expect(result.metadata, isNotNull);
      expect(result.metadata, contains('file_count'));
      expect(result.metadata, contains('"file_count":2'));
      expect(result.appSource, equals('explorer'));
      expect(emitted?.id, equals(result.id));
    });

    test('returns null for empty file list', () async {
      final result = await service.processFiles(
        [],
        ClipboardContentType.file,
      );
      expect(result, isNull);
    });

    test('reactivates duplicate file list', () async {
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      await service.processFiles(
        ['C:\\same.txt'],
        ClipboardContentType.file,
      );
      await service.processFiles(
        ['C:\\same.txt'],
        ClipboardContentType.file,
      );
      await Future<void>.delayed(Duration.zero);

      expect(reactivated, isNotNull);
    });

    test('sets is_directory true for folder type', () async {
      final result = await service.processFiles(
        ['C:\\MyFolder'],
        ClipboardContentType.folder,
      );
      expect(result, isNotNull);
      expect(result!.metadata, contains('"is_directory":true'));
    });
  });

  group('ClipboardService.removeItem', () {
    test('deletes item from repository', () async {
      final item = await service.processText('bye', ClipboardContentType.text);
      await service.removeItem(item!.id);
      final found = await repo.getById(item.id);
      expect(found, isNull);
    });
  });

  group('ClipboardService.updatePin', () {
    test('pins and unpins item', () async {
      final item = await service.processText('pin me', ClipboardContentType.text);

      await service.updatePin(item!.id, true);
      var found = await repo.getById(item.id);
      expect(found!.isPinned, isTrue);

      await service.updatePin(item.id, false);
      found = await repo.getById(item.id);
      expect(found!.isPinned, isFalse);
    });

    test('silently ignores unknown id', () async {
      await expectLater(
        service.updatePin('nonexistent', true),
        completes,
      );
    });
  });

  group('ClipboardService.updateLabelAndColor', () {
    test('updates label and color', () async {
      final item = await service.processText('label', ClipboardContentType.text);

      await service.updateLabelAndColor(item!.id, 'My Label', CardColor.blue);
      final found = await repo.getById(item.id);
      expect(found!.label, equals('My Label'));
      expect(found.cardColor, equals(CardColor.blue));
    });

    test('silently ignores unknown id', () async {
      await expectLater(
        service.updateLabelAndColor('nonexistent', null, CardColor.none),
        completes,
      );
    });
  });

  group('ClipboardService.getHistoryAdvanced', () {
    test('returns all items when no filters are applied', () async {
      await service.processText('item1', ClipboardContentType.text);
      await service.processText('item2', ClipboardContentType.text);
      final results = await service.getHistoryAdvanced(limit: 50, skip: 0);
      expect(results.length, equals(2));
    });

    test('filters by type', () async {
      await service.processText('text item', ClipboardContentType.text);
      await service.processText('link item', ClipboardContentType.link);
      final results = await service.getHistoryAdvanced(
        types: [ClipboardContentType.text],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.type, equals(ClipboardContentType.text));
    });

    test('filters by color', () async {
      final item = await service.processText('colored', ClipboardContentType.text);
      await service.updateLabelAndColor(item!.id, null, CardColor.red);
      await service.processText('no color', ClipboardContentType.text);
      final results = await service.getHistoryAdvanced(
        colors: [CardColor.red],
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.cardColor, equals(CardColor.red));
    });

    test('filters by isPinned', () async {
      final item = await service.processText('pinned', ClipboardContentType.text);
      await service.updatePin(item!.id, true);
      await service.processText('normal', ClipboardContentType.text);
      final results = await service.getHistoryAdvanced(
        isPinned: true,
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.isPinned, isTrue);
    });

    test('filters by query', () async {
      await service.processText('hello world', ClipboardContentType.text);
      await service.processText('something else', ClipboardContentType.text);
      final results = await service.getHistoryAdvanced(
        query: 'hello',
        limit: 50,
        skip: 0,
      );
      expect(results.length, equals(1));
      expect(results.first.content, equals('hello world'));
    });

    test('respects limit and skip', () async {
      for (var i = 0; i < 5; i++) {
        await service.processText('item$i unique_$i', ClipboardContentType.text);
      }
      final page1 = await service.getHistoryAdvanced(limit: 3, skip: 0);
      final page2 = await service.getHistoryAdvanced(limit: 3, skip: 3);
      expect(page1.length, equals(3));
      expect(page2.length, equals(2));
    });
  });

  group('ClipboardService.clearUnpinnedHistory', () {
    test('removes all non-pinned items', () async {
      final item = await service.processText('to be pinned', ClipboardContentType.text);
      await service.updatePin(item!.id, true);
      await service.processText('to be deleted', ClipboardContentType.text);

      final deleted = await service.clearUnpinnedHistory();
      expect(deleted, equals(1));

      final remaining = await service.getHistoryAdvanced(limit: 50, skip: 0);
      expect(remaining.length, equals(1));
      expect(remaining.first.isPinned, isTrue);
    });

    test('returns 0 when nothing to delete', () async {
      final count = await service.clearUnpinnedHistory();
      expect(count, equals(0));
    });
  });

  group('ClipboardService.getItemCount', () {
    test('returns correct count', () async {
      expect(await service.getItemCount(), equals(0));
      await service.processText('one', ClipboardContentType.text);
      expect(await service.getItemCount(), equals(1));
      await service.processText('two', ClipboardContentType.text);
      expect(await service.getItemCount(), equals(2));
    });
  });

  group('ClipboardService.walCheckpoint', () {
    test('completes without error', () async {
      await service.processText('checkpoint test', ClipboardContentType.text);
      await expectLater(service.walCheckpoint(), completes);
    });
  });

  group('ClipboardService.updateMetadata', () {
    test('updates metadata and emits onItemReactivated', () async {
      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      final item =
          await service.processText('meta', ClipboardContentType.text);
      await service.updateMetadata(item!.id, '{"key":"value"}');
      await Future<void>.delayed(Duration.zero);

      final stored = await repo.getById(item.id);
      expect(stored!.metadata, equals('{"key":"value"}'));
      expect(reactivated, isNotNull);
      expect(reactivated!.metadata, equals('{"key":"value"}'));
    });

    test('silently ignores unknown id', () async {
      await expectLater(
        service.updateMetadata('nonexistent', '{}'),
        completes,
      );
    });
  });

  group('ClipboardService.dispose', () {
    test('closes streams after dispose', () async {
      var addedDone = false;
      var reactivatedDone = false;
      service.onItemAdded.listen(null, onDone: () => addedDone = true);
      service.onItemReactivated
          .listen(null, onDone: () => reactivatedDone = true);
      service.dispose();
      await Future<void>.delayed(Duration.zero);
      expect(addedDone, isTrue);
      expect(reactivatedDone, isTrue);
    });
  });
}
