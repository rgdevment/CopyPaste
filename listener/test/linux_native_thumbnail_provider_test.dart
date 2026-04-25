import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:listener/linux_native_thumbnail_provider.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('copypaste/clipboard_writer');

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  group('LinuxNativeThumbnailProvider', () {
    test('returns null on non-Linux hosts (or empty channel)', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async => null);

      final provider = LinuxNativeThumbnailProvider();
      final result = await provider.request('/tmp/missing.png', sizePx: 256);
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

      final provider = LinuxNativeThumbnailProvider();
      final result = await provider.request('/tmp/photo.jpg', sizePx: 128);

      // Outside the platform guard this is a no-op on non-Linux hosts.
      if (receivedPath != null) {
        expect(result, equals(fakeBytes));
        expect(receivedPath, equals('/tmp/photo.jpg'));
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

      final provider = LinuxNativeThumbnailProvider();
      final result = await provider.request('/tmp/missing.bin');
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

      final provider = LinuxNativeThumbnailProvider();
      final result = await provider.request('/tmp/whatever.png');
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

        final provider = LinuxNativeThumbnailProvider();
        expect(await provider.request(''), isNull);
        expect(await provider.request('x', sizePx: 0), isNull);
        expect(await provider.request('x', sizePx: -1), isNull);
        expect(called, isFalse);
      },
    );

    test('survives MissingPluginException (no listener registered)', () async {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, null);

      final provider = LinuxNativeThumbnailProvider();
      final result = await provider.request('/tmp/anything.png');
      expect(result, isNull);
    });
  });
}
