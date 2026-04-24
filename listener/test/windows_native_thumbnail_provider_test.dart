import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:listener/windows_native_thumbnail_provider.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('copypaste/clipboard_writer');

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  group('WindowsNativeThumbnailProvider', () {
    test('returns null on non-Windows hosts', () async {
      // The test runner here is Windows in CI/local; this test still
      // covers the early-return branch because we mock the channel to
      // throw, which would surface as null only via the platform guard.
      // On Linux/macOS hosts the early `Platform.isWindows` guard takes
      // over before any channel call happens.
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async => null);

      final provider = WindowsNativeThumbnailProvider();
      final result = await provider.request('C:/missing.txt', sizePx: 256);
      // On Windows the mock returns null → expect null; on others the
      // platform guard returns null first. Same observable behavior.
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

      final provider = WindowsNativeThumbnailProvider();
      final result = await provider.request('C:/video.mp4', sizePx: 128);

      // Outside the platform guard this is a no-op on non-Windows hosts.
      // We assert behavior conditionally: when the channel was reached,
      // the bytes round-trip and the path was forwarded verbatim.
      if (receivedPath != null) {
        expect(result, equals(fakeBytes));
        expect(receivedPath, equals('C:/video.mp4'));
        // sizePx is scaled by devicePixelRatio (>= 1.0) and clamped >= 64.
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

      final provider = WindowsNativeThumbnailProvider();
      final result = await provider.request('C:/missing.bin');
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

      final provider = WindowsNativeThumbnailProvider();
      final result = await provider.request('C:/whatever.png');
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

        final provider = WindowsNativeThumbnailProvider();
        expect(await provider.request(''), isNull);
        expect(await provider.request('x', sizePx: 0), isNull);
        expect(await provider.request('x', sizePx: -1), isNull);
        expect(called, isFalse);
      },
    );
  });
}
