/// Platform-agnostic tests that verify ClipboardEvent parsing is robust
/// across all content types and unusual native payloads (Windows, macOS, Linux
/// all send `Map<dynamic, dynamic>` via BasicMessageChannel).
library;

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:listener/clipboard_event.dart';
import 'package:core/core.dart';

void main() {
  group('ClipboardEvent.fromMap – all content types', () {
    test('parses type 0 as text', () {
      final event = ClipboardEvent.fromMap({'type': 0, 'text': 'hello'});
      expect(event.type, equals(ClipboardContentType.text));
      expect(event.text, equals('hello'));
    });

    test('parses type 1 as image', () {
      final bytes = Uint8List.fromList([137, 80, 78, 71]);
      final event = ClipboardEvent.fromMap({'type': 1, 'bytes': bytes});
      expect(event.type, equals(ClipboardContentType.image));
      expect(event.bytes, isNotNull);
    });

    test('parses type 2 as file', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'files': ['/home/user/doc.pdf'],
      });
      expect(event.type, equals(ClipboardContentType.file));
      expect(event.files, contains('/home/user/doc.pdf'));
    });

    test('parses type 3 as folder', () {
      final event = ClipboardEvent.fromMap({
        'type': 3,
        'files': ['/home/user/folder'],
      });
      expect(event.type, equals(ClipboardContentType.folder));
    });

    test('parses type 4 as link', () {
      final event = ClipboardEvent.fromMap({
        'type': 4,
        'text': 'https://example.com',
      });
      expect(event.type, equals(ClipboardContentType.link));
    });

    test('parses type 5 as audio', () {
      final event = ClipboardEvent.fromMap({
        'type': 5,
        'files': ['/music/song.mp3'],
      });
      expect(event.type, equals(ClipboardContentType.audio));
    });

    test('parses type 6 as video', () {
      final event = ClipboardEvent.fromMap({
        'type': 6,
        'files': ['/video/clip.mp4'],
      });
      expect(event.type, equals(ClipboardContentType.video));
    });

    test('parses type 7 as email', () {
      final event = ClipboardEvent.fromMap({
        'type': 7,
        'text': 'user@example.com',
      });
      expect(event.type, equals(ClipboardContentType.email));
    });

    test('parses type 8 as phone', () {
      final event = ClipboardEvent.fromMap({
        'type': 8,
        'text': '+1 800 555 0100',
      });
      expect(event.type, equals(ClipboardContentType.phone));
    });

    test('parses type 9 as color', () {
      final event = ClipboardEvent.fromMap({'type': 9, 'text': '#FF5733'});
      expect(event.type, equals(ClipboardContentType.color));
    });

    test('parses type 10 as ip', () {
      final event = ClipboardEvent.fromMap({'type': 10, 'text': '192.168.1.1'});
      expect(event.type, equals(ClipboardContentType.ip));
    });

    test('parses type 11 as uuid', () {
      final event = ClipboardEvent.fromMap({
        'type': 11,
        'text': '550e8400-e29b-41d4-a716-446655440000',
      });
      expect(event.type, equals(ClipboardContentType.uuid));
    });

    test('parses type 12 as json', () {
      final event = ClipboardEvent.fromMap({
        'type': 12,
        'text': '{"key":"value"}',
      });
      expect(event.type, equals(ClipboardContentType.json));
    });

    test('parses unknown type as unknown', () {
      final event = ClipboardEvent.fromMap({'type': 999, 'text': 'anything'});
      expect(event.type, equals(ClipboardContentType.unknown));
    });
  });

  group('ClipboardEvent.fromMap – source / contentHash', () {
    test('parses source field', () {
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'hello',
        'source': 'com.apple.finder',
      });
      expect(event.source, equals('com.apple.finder'));
    });

    test('source is null when not provided', () {
      final event = ClipboardEvent.fromMap({'type': 0, 'text': 'hi'});
      expect(event.source, isNull);
    });

    test('parses contentHash field', () {
      final event = ClipboardEvent.fromMap({
        'type': 1,
        'contentHash': 'sha256-abc',
      });
      expect(event.contentHash, equals('sha256-abc'));
    });

    test('contentHash defaults to empty string when not provided', () {
      final event = ClipboardEvent.fromMap({'type': 0, 'text': 'hello'});
      expect(event.contentHash, equals(''));
    });
  });

  group('ClipboardEvent.fromMap – RTF and HTML bytes', () {
    // Native layer sends these under the keys 'rtf' and 'html'
    test('parses rtfBytes from Uint8List (key: rtf)', () {
      final rtf = Uint8List.fromList([0x7B, 0x5C, 0x72, 0x74, 0x66]);
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'rich',
        'rtf': rtf,
      });
      expect(event.rtfBytes, equals(rtf));
    });

    test('parses htmlBytes from Uint8List (key: html)', () {
      final html = Uint8List.fromList([0x3C, 0x62, 0x3E]);
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'rich',
        'html': html,
      });
      expect(event.htmlBytes, equals(html));
    });

    test('rtfBytes is null when rtf key not provided', () {
      final event = ClipboardEvent.fromMap({'type': 0, 'text': 'plain'});
      expect(event.rtfBytes, isNull);
    });

    test('htmlBytes is null when html key not provided', () {
      final event = ClipboardEvent.fromMap({'type': 0, 'text': 'plain'});
      expect(event.htmlBytes, isNull);
    });
  });

  group('ClipboardEvent.fromMap – files list edge cases', () {
    test('empty files list results in empty list', () {
      final event = ClipboardEvent.fromMap({'type': 2, 'files': <dynamic>[]});
      expect(event.files, isEmpty);
    });

    test('files list with multiple paths is preserved', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'files': ['/a/file1.txt', '/a/file2.txt', '/a/file3.txt'],
      });
      expect(event.files, hasLength(3));
    });

    test('non-string entries in files list are filtered out', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'files': ['/valid/path.txt', 42, null, '/other/path.txt'],
      });
      // Only string entries must be kept — non-strings filtered by whereType
      expect(event.files, hasLength(2));
      expect(event.files, containsAll(['/valid/path.txt', '/other/path.txt']));
    });

    test('missing files field results in null', () {
      final event = ClipboardEvent.fromMap({'type': 2});
      expect(event.files, isNull);
    });
  });

  group('ClipboardEvent.fromMap – defaults and missing fields', () {
    test('missing type defaults to unknown', () {
      final event = ClipboardEvent.fromMap({});
      expect(event.type, equals(ClipboardContentType.unknown));
    });

    test('missing text defaults to null', () {
      final event = ClipboardEvent.fromMap({'type': 0});
      expect(event.text, isNull);
    });

    test('missing bytes defaults to null', () {
      final event = ClipboardEvent.fromMap({'type': 1});
      expect(event.bytes, isNull);
    });
  });

  group('ClipboardEvent.fromMap – Windows-style paths', () {
    test('Windows file path is preserved verbatim', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'files': [r'C:\Users\user\Desktop\file.txt'],
      });
      expect(event.files, contains(r'C:\Users\user\Desktop\file.txt'));
    });

    test('Windows UNC path is preserved verbatim', () {
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'files': [r'\\server\share\file.pdf'],
      });
      expect(event.files, contains(r'\\server\share\file.pdf'));
    });
  });

  group('ClipboardEvent.fromMap – large payloads', () {
    test('handles very long text without truncation', () {
      final longText = 'A' * 100000;
      final event = ClipboardEvent.fromMap({'type': 0, 'text': longText});
      expect(event.text!.length, equals(100000));
    });

    test('handles large image bytes without truncation', () {
      final bigImage = Uint8List(50000);
      final event = ClipboardEvent.fromMap({'type': 1, 'bytes': bigImage});
      expect(event.bytes!.length, equals(50000));
    });

    test('handles many file paths', () {
      final manyPaths = List.generate(200, (i) => '/path/file_$i.txt');
      final event = ClipboardEvent.fromMap({'type': 2, 'files': manyPaths});
      expect(event.files, hasLength(200));
    });
  });
}
