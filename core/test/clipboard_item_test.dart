import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  group('ClipboardItem', () {
    test('generates unique id when none provided', () {
      final a = ClipboardItem(content: 'a', type: ClipboardContentType.text);
      final b = ClipboardItem(content: 'b', type: ClipboardContentType.text);
      expect(a.id, isNotEmpty);
      expect(a.id, isNot(equals(b.id)));
    });

    test('preserves provided id', () {
      final item = ClipboardItem(
        id: 'fixed-id',
        content: 'x',
        type: ClipboardContentType.text,
      );
      expect(item.id, equals('fixed-id'));
    });

    test('default field values', () {
      final item = ClipboardItem(content: 'x', type: ClipboardContentType.text);
      expect(item.isPinned, isFalse);
      expect(item.pasteCount, equals(0));
      expect(item.cardColor, equals(CardColor.none));
      expect(item.appSource, isNull);
      expect(item.label, isNull);
      expect(item.metadata, isNull);
      expect(item.contentHash, isNull);
    });

    test('isFileBasedType true for file, folder, audio, video', () {
      expect(
        ClipboardItem(
          content: '',
          type: ClipboardContentType.file,
        ).isFileBasedType,
        isTrue,
      );
      expect(
        ClipboardItem(
          content: '',
          type: ClipboardContentType.folder,
        ).isFileBasedType,
        isTrue,
      );
      expect(
        ClipboardItem(
          content: '',
          type: ClipboardContentType.audio,
        ).isFileBasedType,
        isTrue,
      );
      expect(
        ClipboardItem(
          content: '',
          type: ClipboardContentType.video,
        ).isFileBasedType,
        isTrue,
      );
    });

    test('isFileBasedType false for text, image, link, unknown', () {
      for (final type in [
        ClipboardContentType.text,
        ClipboardContentType.image,
        ClipboardContentType.link,
        ClipboardContentType.unknown,
      ]) {
        expect(
          ClipboardItem(content: '', type: type).isFileBasedType,
          isFalse,
          reason: '${type.name} should not be file-based',
        );
      }
    });

    test('copyWith only changes specified fields', () {
      final item = ClipboardItem(
        content: 'original',
        type: ClipboardContentType.text,
        pasteCount: 5,
        cardColor: CardColor.blue,
      );
      final copy = item.copyWith(content: 'updated', isPinned: true);
      expect(copy.id, equals(item.id));
      expect(copy.content, equals('updated'));
      expect(copy.isPinned, isTrue);
      expect(copy.pasteCount, equals(5));
      expect(copy.cardColor, equals(CardColor.blue));
      expect(copy.type, equals(ClipboardContentType.text));
    });

    test('copyWith with all card colors', () {
      final item = ClipboardItem(content: 'x', type: ClipboardContentType.text);
      for (final color in CardColor.values) {
        final copy = item.copyWith(cardColor: color);
        expect(copy.cardColor, equals(color));
      }
    });

    test('equality based on id', () {
      final a = ClipboardItem(
        id: 'same-id',
        content: 'a',
        type: ClipboardContentType.text,
      );
      final b = ClipboardItem(
        id: 'same-id',
        content: 'b',
        type: ClipboardContentType.link,
      );
      final c = ClipboardItem(
        id: 'different-id',
        content: 'a',
        type: ClipboardContentType.text,
      );
      expect(a, equals(b));
      expect(a.hashCode, equals(b.hashCode));
      expect(a, isNot(equals(c)));
    });

    test('isFileAvailable returns true for non-file types', () {
      final item = ClipboardItem(
        content: 'text',
        type: ClipboardContentType.text,
      );
      expect(item.isFileAvailable(), isTrue);
    });

    test('isFileAvailable returns false for empty content on file types', () {
      final item = ClipboardItem(content: '', type: ClipboardContentType.file);
      expect(item.isFileAvailable(), isFalse);
    });

    test('isFileAvailable returns false when file does not exist', () {
      final item = ClipboardItem(
        content: '/nonexistent/path/file.txt',
        type: ClipboardContentType.file,
      );
      expect(item.isFileAvailable(), isFalse);
    });

    test(
      'isFileAvailable returns false when content is only whitespace lines',
      () {
        final item = ClipboardItem(
          content: '\n\n',
          type: ClipboardContentType.file,
        );
        expect(item.isFileAvailable(), isFalse);
      },
    );

    test('isFileAvailable returns true when file exists', () {
      final dir = Directory.systemTemp.createTempSync('item_test_');
      try {
        final file = File('${dir.path}/test.txt')..writeAsStringSync('test');
        final item = ClipboardItem(
          content: file.path,
          type: ClipboardContentType.file,
        );
        expect(item.isFileAvailable(), isTrue);
      } finally {
        dir.deleteSync(recursive: true);
      }
    });

    test(
      'isFileAvailable returns true for folder type when directory exists',
      () {
        final dir = Directory.systemTemp.createTempSync('folder_item_test_');
        try {
          final item = ClipboardItem(
            content: dir.path,
            type: ClipboardContentType.folder,
          );
          expect(item.isFileAvailable(), isTrue);
        } finally {
          dir.deleteSync(recursive: true);
        }
      },
    );
  });

  group('ClipboardItem.hasRichText', () {
    ClipboardItem itemWith(String? metadata) =>
        ClipboardItem(content: 'x', type: ClipboardContentType.text)
            .copyWith(metadata: metadata);

    test('is true when metadata carries a non-empty rtf key', () {
      expect(itemWith('{"rtf":"e1xydGYx"}').hasRichText, isTrue);
    });

    test('is false when rtf is present but empty', () {
      expect(itemWith('{"rtf":""}').hasRichText, isFalse);
    });

    test('is false when only html is present', () {
      // Copying from a browser drags text/html along even for plain text, so
      // html alone must not promote an item to rich.
      expect(itemWith('{"html":"PGh0bWw+"}').hasRichText, isFalse);
    });

    test('is false when there is no metadata', () {
      expect(itemWith(null).hasRichText, isFalse);
      expect(itemWith('').hasRichText, isFalse);
    });

    test('is false on malformed or non-map metadata', () {
      expect(itemWith('not json').hasRichText, isFalse);
      expect(itemWith('[1,2,3]').hasRichText, isFalse);
    });

    test('is false when rtf holds a non-string value', () {
      expect(itemWith('{"rtf":42}').hasRichText, isFalse);
    });
  });

  group('ClipboardItem.hasFormatting', () {
    ClipboardItem itemWith(String? metadata) =>
        ClipboardItem(content: 'x', type: ClipboardContentType.text)
            .copyWith(metadata: metadata);

    test('is true for rtf', () {
      expect(itemWith('{"rtf":"e1xydGYx"}').hasFormatting, isTrue);
    });

    test('is true for html alone', () {
      // Unlike hasRichText: the writer restores html to the clipboard, so a
      // normal paste would carry formatting and stripping it is meaningful.
      final item = itemWith('{"html":"PGh0bWw+"}');
      expect(item.hasFormatting, isTrue);
      expect(item.hasRichText, isFalse);
    });

    test('is false when no format payload is attached', () {
      expect(itemWith(null).hasFormatting, isFalse);
      expect(itemWith('{"duration":42}').hasFormatting, isFalse);
      expect(itemWith('{"rtf":"","html":""}').hasFormatting, isFalse);
    });
  });
}
