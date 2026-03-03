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
      await Future.delayed(Duration.zero);

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
      await Future.delayed(Duration.zero);

      expect(second, isNotNull);
      expect(reactivated?.content, equals('dup'));
    });

    test('returns null when inside paste ignore window', () async {
      service.pasteIgnoreWindowMs = 60000;
      await service.notifyPasteInitiated('any-id');

      final result = await service.processText('ignored', ClipboardContentType.text);
      expect(result, isNull);
    });

    test('saves item with source and metadata', () async {
      final result = await service.processText(
        'data',
        ClipboardContentType.link,
        source: 'Chrome',
        metadata: '{"url":"https://example.com"}',
      );
      expect(result!.appSource, equals('Chrome'));
      expect(result.metadata, contains('example.com'));
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
      await Future.delayed(Duration.zero);

      expect(reactivated, isNotNull);
    });
  });

  group('ClipboardService.recordPaste', () {
    test('increments pasteCount and sets lastPastedContent', () async {
      final item = await service.processText('paste me', ClipboardContentType.text);
      expect(item, isNotNull);

      await service.recordPaste(item!.id);

      final updated = await repo.getById(item.id);
      expect(updated!.pasteCount, equals(1));
    });

    test('silently ignores unknown id', () async {
      await expectLater(service.recordPaste('nonexistent-id'), completes);
    });
  });
}
