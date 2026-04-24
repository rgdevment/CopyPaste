import 'dart:io' show Platform;

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:listener/macos_native_thumbnail_provider.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('copypaste/clipboard_writer');

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  group('MacOSNativeThumbnailProvider', () {
    test('returns null when channel returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async => null);

      final provider = MacOSNativeThumbnailProvider();
      final result = await provider.request(
        '/Users/me/missing.png',
        sizePx: 256,
      );
      expect(result, isNull);
    });

    test('returns Uint8List bytes when channel succeeds', () async {
      final fakeBytes = Uint8List.fromList(List<int>.generate(64, (i) => i));
      String? receivedPath;
      int? receivedSize;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method != 'getNativeThumbnail') return null;
            final args = call.arguments as Map<Object?, Object?>;
            receivedPath = args['path'] as String?;
            receivedSize = args['sizePx'] as int?;
            return fakeBytes;
          });

      final provider = MacOSNativeThumbnailProvider();
      final result = await provider.request('/Users/me/video.mp4', sizePx: 128);

      // On non-macOS hosts the platform guard short-circuits and the channel
      // is never reached. Assert the bytes round-trip only when it was.
      if (receivedPath != null) {
        expect(result, equals(fakeBytes));
        expect(receivedPath, equals('/Users/me/video.mp4'));
        expect(receivedSize, greaterThanOrEqualTo(128));
      } else {
        expect(result, isNull);
      }
    });

    test('treats empty list as null (no thumbnail available)', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'getNativeThumbnail') return Uint8List(0);
            return null;
          });

      final provider = MacOSNativeThumbnailProvider();
      final result = await provider.request('/Users/me/missing.bin');
      expect(result, isNull);
    });

    test('swallows PlatformException and returns null', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'getNativeThumbnail') {
              throw PlatformException(code: 'boom', message: 'native failure');
            }
            return null;
          });

      final provider = MacOSNativeThumbnailProvider();
      final result = await provider.request('/Users/me/whatever.png');
      expect(result, isNull);
    });

    test('TCC permissionDenied surfaces as null without throwing', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            if (call.method == 'getNativeThumbnail') {
              throw PlatformException(
                code: 'permissionDenied',
                message: 'TCC denied',
              );
            }
            return null;
          });

      final provider = MacOSNativeThumbnailProvider();
      final result = await provider.request('/Users/me/Documents/x.png');
      expect(result, isNull);
    });

    test(
      'rejects empty path / non-positive size before invoking channel',
      () async {
        var called = false;
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, (call) async {
              called = true;
              return null;
            });

        final provider = MacOSNativeThumbnailProvider();
        expect(await provider.request(''), isNull);
        expect(await provider.request('x', sizePx: 0), isNull);
        expect(await provider.request('x', sizePx: -1), isNull);
        expect(called, isFalse);
      },
    );

    test('returns null on non-macOS hosts (platform guard)', () async {
      var called = false;
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            called = true;
            return Uint8List.fromList([1, 2, 3]);
          });

      final provider = MacOSNativeThumbnailProvider();
      final result = await provider.request('/x', sizePx: 256);
      if (!Platform.isMacOS) {
        expect(result, isNull);
        expect(called, isFalse);
      }
    });
  });
}
