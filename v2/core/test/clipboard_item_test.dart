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
  });
}
