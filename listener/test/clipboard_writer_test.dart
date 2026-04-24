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

  group('ClipboardWriter.captureFrontmostApp', () {
    test('returns bundle id on success', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'captureFrontmostApp') {
              return 'com.apple.finder';
            }
            return null;
          });
      final result = await ClipboardWriter.captureFrontmostApp();
      expect(result, equals('com.apple.finder'));
    });

    test('returns null when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async => null);
      final result = await ClipboardWriter.captureFrontmostApp();
      expect(result, isNull);
    });

    test('returns null when channel throws', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            throw PlatformException(code: 'UNAVAILABLE');
          });
      final result = await ClipboardWriter.captureFrontmostApp();
      expect(result, isNull);
    });
  });

  group('ClipboardWriter.activateAndPaste', () {
    test('returns success on bool true', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'activateAndPaste') return true;
            return null;
          });
      final result = await ClipboardWriter.activateAndPaste(
        bundleId: 'com.apple.safari',
        delayMs: 150,
      );
      expect(result.success, isTrue);
      expect(result.errorCode, isNull);
    });

    test('parses Map response with success and errorCode', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            return <String, Object?>{
              'success': false,
              'errorCode': 'focusTimeout',
            };
          });
      final result = await ClipboardWriter.activateAndPaste(
        bundleId: 'x11:0xabc',
        delayMs: 0,
      );
      expect(result.success, isFalse);
      expect(result.errorCode, equals('focusTimeout'));
      expect(result.isFocusTimeout, isTrue);
    });

    test('sends bundleId, delayMs and focusTimeoutMs as arguments', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return true;
          });
      await ClipboardWriter.activateAndPaste(
        bundleId: 'com.example.app',
        delayMs: 200,
        focusTimeoutMs: 350,
      );
      expect(captured!.method, equals('activateAndPaste'));
      expect(captured!.arguments['bundleId'], equals('com.example.app'));
      expect(captured!.arguments['delayMs'], equals(200));
      expect(captured!.arguments['focusTimeoutMs'], equals(350));
    });

    test('returns failure when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async => null);
      final result = await ClipboardWriter.activateAndPaste(
        bundleId: 'com.test',
        delayMs: 0,
      );
      expect(result.success, isFalse);
    });

    test('rethrows when channel throws ACCESSIBILITY_DENIED', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            throw PlatformException(code: 'ACCESSIBILITY_DENIED');
          });
      expect(
        () =>
            ClipboardWriter.activateAndPaste(bundleId: 'com.test', delayMs: 0),
        throwsA(
          isA<PlatformException>().having(
            (e) => e.code,
            'code',
            'ACCESSIBILITY_DENIED',
          ),
        ),
      );
    });

    test('returns platformError on other PlatformException', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            throw PlatformException(code: 'UNKNOWN_ERROR');
          });
      final result = await ClipboardWriter.activateAndPaste(
        bundleId: 'com.test',
        delayMs: 0,
      );
      expect(result.success, isFalse);
      expect(result.errorCode, equals('platformError'));
    });
  });

  group('ClipboardWriter.getCursorAndScreenInfo', () {
    test('returns typed map on success', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'getCursorAndScreenInfo') {
              return <String, Object?>{
                'cursorX': 100.0,
                'cursorY': 200.0,
                'waLeft': 0.0,
                'waTop': 25.0,
                'waRight': 1440.0,
                'waBottom': 900.0,
              };
            }
            return null;
          });
      final result = await ClipboardWriter.getCursorAndScreenInfo();
      expect(result, isNotNull);
      expect(result!['cursorX'], equals(100.0));
      expect(result['cursorY'], equals(200.0));
      expect(result['waLeft'], equals(0.0));
      expect(result['waTop'], equals(25.0));
      expect(result['waRight'], equals(1440.0));
      expect(result['waBottom'], equals(900.0));
    });

    test('converts integer values to double', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'getCursorAndScreenInfo') {
              return <String, Object?>{
                'cursorX': 50,
                'cursorY': 75,
                'waLeft': 0,
                'waTop': 0,
                'waRight': 1280,
                'waBottom': 800,
              };
            }
            return null;
          });
      final result = await ClipboardWriter.getCursorAndScreenInfo();
      expect(result, isNotNull);
      expect(result!['cursorX'], isA<double>());
      expect(result['cursorX'], equals(50.0));
    });

    test('returns null when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async => null);
      final result = await ClipboardWriter.getCursorAndScreenInfo();
      expect(result, isNull);
    });

    test('returns null when channel throws', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            throw PlatformException(code: 'ERROR');
          });
      final result = await ClipboardWriter.getCursorAndScreenInfo();
      expect(result, isNull);
    });
  });

  group('ClipboardWriter.checkAccessibility', () {
    test('returns true when accessibility is granted', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'checkAccessibility') return true;
            return null;
          });
      final result = await ClipboardWriter.checkAccessibility();
      expect(result, isTrue);
    });

    test('returns false when accessibility is denied', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'checkAccessibility') return false;
            return null;
          });
      final result = await ClipboardWriter.checkAccessibility();
      expect(result, isFalse);
    });

    test('returns false when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async => null);
      final result = await ClipboardWriter.checkAccessibility();
      expect(result, isFalse);
    });

    test('returns false when channel throws', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            throw PlatformException(code: 'ERROR');
          });
      final result = await ClipboardWriter.checkAccessibility();
      expect(result, isFalse);
    });
  });

  group('ClipboardWriter.requestAccessibility', () {
    test('returns true when user grants permission', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'requestAccessibility') return true;
            return null;
          });
      final result = await ClipboardWriter.requestAccessibility();
      expect(result, isTrue);
    });

    test('returns false when user denies permission', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'requestAccessibility') return false;
            return null;
          });
      final result = await ClipboardWriter.requestAccessibility();
      expect(result, isFalse);
    });

    test('returns false when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async => null);
      final result = await ClipboardWriter.requestAccessibility();
      expect(result, isFalse);
    });

    test('returns false when channel throws', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            throw PlatformException(code: 'ERROR');
          });
      final result = await ClipboardWriter.requestAccessibility();
      expect(result, isFalse);
    });
  });

  group('ClipboardWriter.openAccessibilitySettings', () {
    test('completes without error on success', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'openAccessibilitySettings') return true;
            return null;
          });
      await expectLater(ClipboardWriter.openAccessibilitySettings(), completes);
    });

    test('completes without error even when channel throws', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (_) async {
            throw PlatformException(code: 'ERROR');
          });
      await expectLater(ClipboardWriter.openAccessibilitySettings(), completes);
    });

    test('invokes correct method name', () async {
      MethodCall? captured;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            captured = call;
            return null;
          });
      await ClipboardWriter.openAccessibilitySettings();
      expect(captured!.method, equals('openAccessibilitySettings'));
    });
  });
}
