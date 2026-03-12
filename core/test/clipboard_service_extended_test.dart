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

  group('ClipboardService.processFiles', () {
    test('creates file item from multiple file paths', () async {
      final files = ['C:\\file1.txt', 'C:\\file2.txt', 'C:\\file3.txt'];
      final result = await service.processFiles(
        files,
        ClipboardContentType.file,
      );

      expect(result, isNotNull);
      expect(result!.type, equals(ClipboardContentType.file));
      expect(result.content, contains('file1.txt'));
      expect(result.content, contains('file2.txt'));
      expect(result.metadata, isNotNull);
    });

    test('creates folder item from path', () async {
      final result = await service.processFiles([
        'C:\\MyFolder',
      ], ClipboardContentType.folder);

      expect(result, isNotNull);
      expect(result!.type, equals(ClipboardContentType.folder));
    });

    test('returns null for empty file list', () async {
      final result = await service.processFiles([], ClipboardContentType.file);

      expect(result, isNull);
    });

    test('reactivates existing file item', () async {
      final files = ['C:\\existing.txt'];
      final first = await service.processFiles(
        files,
        ClipboardContentType.file,
      );

      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      final second = await service.processFiles(
        files,
        ClipboardContentType.file,
      );
      await Future<void>.delayed(Duration.zero);

      expect(second, isNotNull);
      expect(reactivated?.id, equals(first!.id));
    });

    test('includes file metadata in item', () async {
      final files = ['C:\\test.pdf'];
      final result = await service.processFiles(
        files,
        ClipboardContentType.file,
      );

      expect(result!.metadata, isNotNull);
      expect(result.metadata, contains('file_count'));
      expect(result.metadata, contains('file_name'));
      expect(result.metadata, contains('first_ext'));
    });
  });

  group('ClipboardService.processImage', () {
    test('creates image item by contentHash', () async {
      final result = await service.processImage(
        'hash-abc-123',
        imagePath: '/tmp/image.png',
      );

      expect(result, isNotNull);
      expect(result!.contentHash, equals('hash-abc-123'));
      expect(result.type, equals(ClipboardContentType.image));
    });

    test('reactivates existing image by contentHash', () async {
      const hash = 'dup-hash';
      final first = await service.processImage(hash, imagePath: '/img1.png');

      ClipboardItem? reactivated;
      service.onItemReactivated.listen((item) => reactivated = item);

      final second = await service.processImage(hash, imagePath: '/img2.png');
      await Future<void>.delayed(Duration.zero);

      expect(second, isNotNull);
      expect(reactivated?.id, equals(first!.id));
    });

    test('stores image path in content field', () async {
      const imagePath = '/home/user/screenshot.png';
      final result = await service.processImage(
        'path-hash',
        imagePath: imagePath,
      );

      expect(result!.content, equals(imagePath));
    });
  });

  group('ClipboardService.notifyPasteInitiated', () {
    test('ignores items within paste window', () async {
      service.pasteIgnoreWindowMs = 100;

      final item = await service.processText(
        'first content',
        ClipboardContentType.text,
      );
      expect(item, isNotNull);

      await service.notifyPasteInitiated(item!.id);

      final ignored = await service.processText(
        'second content',
        ClipboardContentType.text,
      );

      expect(ignored, isNull);
    });

    test('window expires allowing new items', () async {
      service.pasteIgnoreWindowMs = 50;

      final first = await service.processText(
        'content1',
        ClipboardContentType.text,
      );
      expect(first, isNotNull);

      await service.notifyPasteInitiated(first!.id);
      await Future<void>.delayed(const Duration(milliseconds: 60));

      final second = await service.processText(
        'content2',
        ClipboardContentType.text,
      );

      expect(second, isNotNull);
      expect(second!.content, equals('content2'));
    });

    test('same content ignored within double window', () async {
      service.pasteIgnoreWindowMs = 50;

      const text = 'duplicate content';
      final first = await service.processText(text, ClipboardContentType.text);
      expect(first, isNotNull);

      await service.notifyPasteInitiated(first!.id);
      await Future<void>.delayed(const Duration(milliseconds: 30));

      final duplicate = await service.processText(
        text,
        ClipboardContentType.text,
      );

      expect(duplicate, isNull);
    });
  });

  group('ClipboardService.dispose', () {
    test('disposes resources without errors', () async {
      final testService = ClipboardService(repo);

      // Just verify dispose completes without errors
      testService.dispose();
      expect(true, isTrue);
    });
  });

  group('ClipboardService integration scenarios', () {
    test('text and file items coexist', () async {
      final text = await service.processText(
        'text content',
        ClipboardContentType.text,
      );
      final files = await service.processFiles([
        'C:\\document.docx',
      ], ClipboardContentType.file);

      expect(text, isNotNull);
      expect(files, isNotNull);
      expect(text!.type, equals(ClipboardContentType.text));
      expect(files!.type, equals(ClipboardContentType.file));
      expect(text.id, isNot(files.id));
    });

    test('can process all content types', () async {
      final text = await service.processText('text', ClipboardContentType.text);
      final link = await service.processText(
        'https://example.com',
        ClipboardContentType.link,
      );
      final image = await service.processImage('img-hash');
      final files = await service.processFiles([
        'file.txt',
      ], ClipboardContentType.file);

      expect(text, isNotNull);
      expect(link, isNotNull);
      expect(image, isNotNull);
      expect(files, isNotNull);
    });

    test('metadata is preserved for complex content', () async {
      final result = await service.processText(
        'rich content',
        ClipboardContentType.text,
        source: 'Word',
        rtfBytes: [0x7B, 0x5C, 0x72, 0x74, 0x66],
        htmlBytes: [0x3C, 0x68, 0x74, 0x6D, 0x6C],
      );

      expect(result, isNotNull);
      expect(result!.appSource, equals('Word'));
      expect(result.metadata, isNotNull);
      expect(result.metadata, contains('rtf'));
      expect(result.metadata, contains('html'));
    });
  });
}
