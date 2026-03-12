import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:listener/listener.dart';
import 'package:core/core.dart';

void main() {
  group('ClipboardEvent.fromMap', () {
    test('parses text event with all fields', () {
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'contentHash': 'abc123',
        'text': 'Hello World',
        'source': 'Notepad',
      });
      expect(event.type, equals(ClipboardContentType.text));
      expect(event.contentHash, equals('abc123'));
      expect(event.text, equals('Hello World'));
      expect(event.source, equals('Notepad'));
      expect(event.bytes, isNull);
      expect(event.files, isNull);
      expect(event.rtfBytes, isNull);
      expect(event.htmlBytes, isNull);
    });

    test('parses image event with bytes', () {
      final bytes = Uint8List.fromList([137, 80, 78, 71]); // PNG magic
      final event = ClipboardEvent.fromMap({
        'type': 1,
        'contentHash': 'img_hash',
        'bytes': bytes,
      });
      expect(event.type, equals(ClipboardContentType.image));
      expect(event.contentHash, equals('img_hash'));
      expect(event.bytes, isNotNull);
      expect(event.bytes!.length, equals(4));
      expect(event.text, isNull);
    });

    test('parses file event with file list', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'contentHash': 'file_hash',
        'files': <Object?>['C:\\file1.txt', 'C:\\file2.txt'],
      });
      expect(event.type, equals(ClipboardContentType.file));
      expect(event.files, isNotNull);
      expect(event.files!.length, equals(2));
      expect(event.files![0], equals('C:\\file1.txt'));
      expect(event.files![1], equals('C:\\file2.txt'));
    });

    test('parses folder event', () {
      final event = ClipboardEvent.fromMap({
        'type': 3,
        'contentHash': 'folder_hash',
        'files': <Object?>['C:\\MyFolder'],
      });
      expect(event.type, equals(ClipboardContentType.folder));
    });

    test('parses link event', () {
      final event = ClipboardEvent.fromMap({
        'type': 4,
        'contentHash': 'link_hash',
        'text': 'https://example.com',
      });
      expect(event.type, equals(ClipboardContentType.link));
      expect(event.text, equals('https://example.com'));
    });

    test('parses audio event', () {
      final event = ClipboardEvent.fromMap({
        'type': 5,
        'contentHash': 'audio_hash',
        'files': <Object?>['C:\\song.mp3'],
      });
      expect(event.type, equals(ClipboardContentType.audio));
    });

    test('parses video event', () {
      final event = ClipboardEvent.fromMap({
        'type': 6,
        'contentHash': 'video_hash',
        'files': <Object?>['C:\\video.mp4'],
      });
      expect(event.type, equals(ClipboardContentType.video));
    });

    test('parses rtf and html bytes', () {
      final rtf = Uint8List.fromList([72, 84, 70]);
      final html = Uint8List.fromList([60, 104, 116]);
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'contentHash': 'rich',
        'text': 'rich text',
        'rtf': rtf,
        'html': html,
      });
      expect(event.rtfBytes, isNotNull);
      expect(event.rtfBytes!.length, equals(3));
      expect(event.htmlBytes, isNotNull);
      expect(event.htmlBytes!.length, equals(3));
    });

    test('handles missing optional fields gracefully', () {
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'contentHash': 'minimal',
      });
      expect(event.text, isNull);
      expect(event.source, isNull);
      expect(event.bytes, isNull);
      expect(event.files, isNull);
      expect(event.rtfBytes, isNull);
      expect(event.htmlBytes, isNull);
    });

    test('handles unknown type value returns unknown', () {
      final event = ClipboardEvent.fromMap({'type': 999, 'contentHash': 'u'});
      expect(event.type, equals(ClipboardContentType.unknown));
    });

    test('handles missing type defaults to unknown', () {
      final event = ClipboardEvent.fromMap({'contentHash': 'u'});
      expect(event.type, equals(ClipboardContentType.unknown));
    });

    test('handles null type defaults to unknown', () {
      final event = ClipboardEvent.fromMap({'type': null, 'contentHash': 'u'});
      expect(event.type, equals(ClipboardContentType.unknown));
    });

    test('handles missing contentHash defaults to empty string', () {
      final event = ClipboardEvent.fromMap({'type': 0});
      expect(event.contentHash, equals(''));
    });

    test('filters non-string items from files list', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'contentHash': 'h',
        'files': <Object?>['valid.txt', 42, null, 'also_valid.txt'],
      });
      expect(event.files!.length, equals(2));
      expect(event.files![0], equals('valid.txt'));
      expect(event.files![1], equals('also_valid.txt'));
    });

    test('empty files list results in empty list', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'contentHash': 'h',
        'files': <Object?>[],
      });
      expect(event.files, isNotNull);
      expect(event.files, isEmpty);
    });

    test('all content types parse correctly', () {
      final cases = {
        0: ClipboardContentType.text,
        1: ClipboardContentType.image,
        2: ClipboardContentType.file,
        3: ClipboardContentType.folder,
        4: ClipboardContentType.link,
        5: ClipboardContentType.audio,
        6: ClipboardContentType.video,
        -1: ClipboardContentType.unknown,
      };
      for (final entry in cases.entries) {
        final event = ClipboardEvent.fromMap({
          'type': entry.key,
          'contentHash': 'h',
        });
        expect(
          event.type,
          equals(entry.value),
          reason: 'type ${entry.key} should map to ${entry.value}',
        );
      }
    });
  });
}
