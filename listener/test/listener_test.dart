import 'dart:typed_data';

import 'package:core/core.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:listener/listener.dart';

void main() {
  group('ClipboardEvent.fromMap', () {
    test('parses text event', () {
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'hello world',
        'source': 'notepad',
        'contentHash': 'abc123',
      });
      expect(event.type, ClipboardContentType.text);
      expect(event.text, 'hello world');
      expect(event.source, 'notepad');
      expect(event.contentHash, 'abc123');
      expect(event.bytes, isNull);
      expect(event.files, isNull);
    });

    test('parses link event', () {
      final event = ClipboardEvent.fromMap({
        'type': 4,
        'text': 'https://example.com',
        'contentHash': 'def456',
      });
      expect(event.type, ClipboardContentType.link);
      expect(event.text, 'https://example.com');
    });

    test('parses files event', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'files': ['C:\\file1.txt', 'C:\\file2.txt'],
        'contentHash': 'xyz',
      });
      expect(event.type, ClipboardContentType.file);
      expect(event.files, hasLength(2));
      expect(event.files!.first, 'C:\\file1.txt');
    });

    test('uses defaults for missing fields', () {
      final event = ClipboardEvent.fromMap({});
      expect(event.type, ClipboardContentType.unknown);
      expect(event.contentHash, '');
      expect(event.source, isNull);
      expect(event.rtfBytes, isNull);
      expect(event.htmlBytes, isNull);
    });

    test('parses rtf and html bytes', () {
      final rtf = Uint8List.fromList([72, 69, 76, 76, 79]);
      final html = Uint8List.fromList([60, 104, 62]);
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'test',
        'contentHash': 'h1',
        'rtf': rtf,
        'html': html,
      });
      expect(event.rtfBytes, equals(rtf));
      expect(event.htmlBytes, equals(html));
    });
  });
}
