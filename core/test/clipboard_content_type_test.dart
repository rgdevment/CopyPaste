import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  group('ClipboardContentType.value', () {
    test('returns correct int for each variant', () {
      expect(ClipboardContentType.unknown.value, equals(-1));
      expect(ClipboardContentType.text.value, equals(0));
      expect(ClipboardContentType.image.value, equals(1));
      expect(ClipboardContentType.file.value, equals(2));
      expect(ClipboardContentType.folder.value, equals(3));
      expect(ClipboardContentType.link.value, equals(4));
      expect(ClipboardContentType.audio.value, equals(5));
      expect(ClipboardContentType.video.value, equals(6));
      expect(ClipboardContentType.email.value, equals(7));
      expect(ClipboardContentType.phone.value, equals(8));
      expect(ClipboardContentType.color.value, equals(9));
      expect(ClipboardContentType.ip.value, equals(10));
      expect(ClipboardContentType.uuid.value, equals(11));
      expect(ClipboardContentType.json.value, equals(12));
    });
  });

  group('ClipboardContentType.fromValue', () {
    test('converts known int values', () {
      expect(ClipboardContentType.fromValue(0), ClipboardContentType.text);
      expect(ClipboardContentType.fromValue(1), ClipboardContentType.image);
      expect(ClipboardContentType.fromValue(2), ClipboardContentType.file);
      expect(ClipboardContentType.fromValue(3), ClipboardContentType.folder);
      expect(ClipboardContentType.fromValue(4), ClipboardContentType.link);
      expect(ClipboardContentType.fromValue(5), ClipboardContentType.audio);
      expect(ClipboardContentType.fromValue(6), ClipboardContentType.video);
      expect(ClipboardContentType.fromValue(7), ClipboardContentType.email);
      expect(ClipboardContentType.fromValue(8), ClipboardContentType.phone);
      expect(ClipboardContentType.fromValue(9), ClipboardContentType.color);
      expect(ClipboardContentType.fromValue(10), ClipboardContentType.ip);
      expect(ClipboardContentType.fromValue(11), ClipboardContentType.uuid);
      expect(ClipboardContentType.fromValue(12), ClipboardContentType.json);
    });

    test('returns unknown for out-of-range values', () {
      expect(ClipboardContentType.fromValue(-1), ClipboardContentType.unknown);
      expect(ClipboardContentType.fromValue(99), ClipboardContentType.unknown);
      expect(ClipboardContentType.fromValue(-99), ClipboardContentType.unknown);
    });

    test('value and fromValue are inverse for all non-unknown variants', () {
      for (final type in ClipboardContentType.values) {
        if (type == ClipboardContentType.unknown) continue;
        expect(ClipboardContentType.fromValue(type.value), equals(type));
      }
    });
  });
}
