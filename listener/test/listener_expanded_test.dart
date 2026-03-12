import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:listener/listener.dart';
import 'package:core/core.dart';

void main() {
  group('Listener Plugin Interface', () {
    test('creates ClipboardEvent correctly from native event', () {
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'test content',
        'contentHash': 'hash123',
        'source': 'TestApp',
      });

      expect(event.type, ClipboardContentType.text);
      expect(event.text, 'test content');
      expect(event.contentHash, 'hash123');
      expect(event.source, 'TestApp');
    });

    test('ClipboardEvent handles empty text', () {
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': '',
        'contentHash': 'empty_hash',
      });

      expect(event.text, isEmpty);
      expect(event.contentHash, 'empty_hash');
    });

    test('ClipboardEvent with very long text', () {
      final longText = 'x' * 100000;
      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': longText,
        'contentHash': 'big_hash',
      });

      expect(event.text, equals(longText));
      expect(event.text!.length, equals(100000));
    });

    test('ClipboardEvent image with large URI', () {
      final event = ClipboardEvent.fromMap({
        'type': 1,
        'contentHash': 'image_hash',
        'bytes': Uint8List.fromList(List.filled(1000, 255)),
      });

      expect(event.type, ClipboardContentType.image);
      expect(event.bytes!.length, equals(1000));
    });

    test('ClipboardEvent files with multiple paths', () {
      const paths = [
        'C:\\Documents\\file1.pdf',
        'C:\\Documents\\file2.docx',
        'D:\\Photos\\image.jpg',
        'E:\\Videos\\movie.mp4',
      ];

      final event = ClipboardEvent.fromMap({
        'type': 2,
        'contentHash': 'files_hash',
        'files': paths,
      });

      expect(event.files, hasLength(4));
      expect(event.files, containsAll(paths));
    });

    test('ClipboardEvent with RTF and HTML formatting', () {
      final rtf = Uint8List.fromList([
        0x7B,
        0x5C,
        0x72,
        0x74,
        0x66,
        0x31,
        0x20,
        0x74,
        0x65,
        0x73,
        0x74,
        0x7D,
      ]); // {\\rtf1 test}
      final html = Uint8List.fromList([
        0x3C,
        0x62,
        0x3E,
        0x74,
        0x65,
        0x73,
        0x74,
        0x3C,
        0x2F,
        0x62,
        0x3E,
      ]); // <b>test</b>

      final event = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'formatted text',
        'contentHash': 'formatted',
        'rtf': rtf,
        'html': html,
      });

      expect(event.rtfBytes, isNotNull);
      expect(event.htmlBytes, isNotNull);
      expect(event.rtfBytes!.length, equals(12));
      expect(event.htmlBytes!.length, equals(11));
    });

    test('ClipboardEvent link type specialized event', () {
      const url = 'https://github.com/user/project/issues/123';
      final event = ClipboardEvent.fromMap({
        'type': 4,
        'text': url,
        'contentHash': 'url_hash',
        'source': 'Chrome',
      });

      expect(event.type, ClipboardContentType.link);
      expect(event.text, url);
      expect(event.source, 'Chrome');
    });

    test('ClipboardEvent audio type with metadata', () {
      final event = ClipboardEvent.fromMap({
        'type': 5,
        'files': ['C:\\Music\\song.mp3'],
        'contentHash': 'audio_hash',
        'source': 'Windows Explorer',
      });

      expect(event.type, ClipboardContentType.audio);
      expect(event.files!.first, contains('song.mp3'));
    });

    test('ClipboardEvent video type with multiple files', () {
      const files = ['C:\\Video1.mp4', 'C:\\Video2.mp4', 'C:\\Subtitle.srt'];

      final event = ClipboardEvent.fromMap({
        'type': 6,
        'files': files,
        'contentHash': 'video_hash',
      });

      expect(event.type, ClipboardContentType.video);
      expect(event.files, hasLength(3));
      expect(event.files, equals(files));
    });

    test('ClipboardEvent handles mixed file types in files list', () {
      // Some implementations might have non-string items that need filtering
      final event = ClipboardEvent.fromMap({
        'type': 2,
        'files': ['path1.txt', 'path2.txt', 'path3.txt'],
        'contentHash': 'mixed_hash',
      });

      expect(event.files, isNotNull);
      expect(event.files!.isNotEmpty, true);
    });

    test('ClipboardEvent maintains content hash uniqueness', () {
      final event1 = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'content A',
        'contentHash': 'hash_A',
      });

      final event2 = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'content A',
        'contentHash': 'hash_A',
      });

      final event3 = ClipboardEvent.fromMap({
        'type': 0,
        'text': 'content B',
        'contentHash': 'hash_B',
      });

      expect(event1.contentHash, event2.contentHash);
      expect(event1.contentHash, isNot(event3.contentHash));
    });

    test('ClipboardEvent folder type', () {
      final event = ClipboardEvent.fromMap({
        'type': 3,
        'files': ['C:\\Users\\User\\AppData'],
        'contentHash': 'folder_hash',
      });

      expect(event.type, ClipboardContentType.folder);
      expect(event.files, isNotEmpty);
    });
  });
}
