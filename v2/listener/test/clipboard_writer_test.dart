import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:listener/clipboard_writer.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('copypaste/clipboard_writer');

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, (call) async {
          switch (call.method) {
            case 'setClipboardContent':
              return true;
            case 'getMediaInfo':
              return <String, Object?>{'width': 1920, 'height': 1080};
            default:
              return null;
          }
        });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  group('ClipboardWriter.setText', () {
    test('returns true on success', () async {
      final result = await ClipboardWriter.setText('hello');
      expect(result, isTrue);
    });

    test('sends plain text flag', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setText('hi', plainText: true);
      expect(captured!.arguments['plainText'], isTrue);
    });

    test('sends rtf decoded from base64 in metadata', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      final rtfBytes = utf8.encode('{\\rtf1 hello}');
      final meta = jsonEncode({'rtf': base64Encode(rtfBytes)});
      await ClipboardWriter.setText('hello', metadata: meta);
      expect(captured!.arguments['rtf'], isNotNull);
    });

    test('sends html decoded from base64 in metadata', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      final htmlBytes = utf8.encode('<b>hello</b>');
      final meta = jsonEncode({'html': base64Encode(htmlBytes)});
      await ClipboardWriter.setText('hello', metadata: meta);
      expect(captured!.arguments['html'], isNotNull);
    });

    test('handles invalid metadata JSON gracefully', () async {
      final result = await ClipboardWriter.setText(
        'test',
        metadata: 'not valid json {{{',
      );
      expect(result, isTrue);
    });

    test('skips rtf/html when plainText is true', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      final rtfBytes = utf8.encode('{\\rtf1 hello}');
      final meta = jsonEncode({'rtf': base64Encode(rtfBytes)});
      await ClipboardWriter.setText('hi', metadata: meta, plainText: true);
      expect(captured!.arguments.containsKey('rtf'), isFalse);
    });

    test('returns false when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async => null);
      final result = await ClipboardWriter.setText('test');
      expect(result, isFalse);
    });
  });

  group('ClipboardWriter.setImage', () {
    test('returns true on success', () async {
      final result = await ClipboardWriter.setImage('/path/to/image.png');
      expect(result, isTrue);
    });

    test('sends type 1 and correct path', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setImage('/img/photo.png');
      expect(captured!.arguments['type'], equals(1));
      expect(captured!.arguments['content'], equals('/img/photo.png'));
    });
  });

  group('ClipboardWriter.setFiles', () {
    test('returns true on success', () async {
      final result = await ClipboardWriter.setFiles('/path/to/file.txt', 2);
      expect(result, isTrue);
    });

    test('sends provided typeValue', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setFiles('/file.mp3', 5);
      expect(captured!.arguments['type'], equals(5));
    });
  });

  group('ClipboardWriter.setFromItem', () {
    test('type 0 (text) calls setText', () async {
      final result = await ClipboardWriter.setFromItem(
        typeValue: 0,
        content: 'text content',
      );
      expect(result, isTrue);
    });

    test('type 4 (link) calls setText', () async {
      final result = await ClipboardWriter.setFromItem(
        typeValue: 4,
        content: 'https://example.com',
      );
      expect(result, isTrue);
    });

    test('type 1 (image) calls setImage', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setFromItem(typeValue: 1, content: '/path/img.png');
      expect(captured!.arguments['type'], equals(1));
    });

    test('type 2 (file) calls setFiles', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setFromItem(typeValue: 2, content: '/file.txt');
      expect(captured!.arguments['type'], equals(2));
    });

    test('type 3 (folder) calls setFiles', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setFromItem(typeValue: 3, content: '/folder/');
      expect(captured!.arguments['type'], equals(3));
    });

    test('type 5 (audio) calls setFiles', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setFromItem(typeValue: 5, content: '/audio.mp3');
      expect(captured!.arguments['type'], equals(5));
    });

    test('type 6 (video) calls setFiles', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setFromItem(typeValue: 6, content: '/video.mp4');
      expect(captured!.arguments['type'], equals(6));
    });

    test('unknown type defaults to plainText setText', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.setFromItem(typeValue: 99, content: 'fallback');
      expect(captured!.arguments['plainText'], isTrue);
    });
  });

  group('ClipboardWriter.getMediaInfo', () {
    test('returns map on success', () async {
      final result = await ClipboardWriter.getMediaInfo('/path/video.mp4');
      expect(result, isNotNull);
      expect(result!['width'], equals(1920));
    });

    test('returns null when channel throws', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            throw PlatformException(code: 'ERROR', message: 'fail');
          });
      final result = await ClipboardWriter.getMediaInfo('/bad/path');
      expect(result, isNull);
    });
  });
}
