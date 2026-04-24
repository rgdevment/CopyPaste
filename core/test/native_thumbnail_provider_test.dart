import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'package:core/core.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

/// Test stub: returns whatever bytes the test sets, records each call.
class _StubProvider implements NativeThumbnailProvider {
  Uint8List? bytes;
  bool throwIt = false;
  Duration? delay;
  final List<({String path, int sizePx})> calls = [];

  @override
  Future<Uint8List?> request(String path, {int sizePx = 256}) async {
    calls.add((path: path, sizePx: sizePx));
    if (delay != null) await Future<void>.delayed(delay!);
    if (throwIt) throw StateError('boom');
    return bytes;
  }
}

Uint8List _validPng() {
  final image = img.Image(width: 4, height: 4);
  return Uint8List.fromList(img.encodePng(image));
}

Future<ClipboardItem> _waitForThumb(
  SqliteRepository repo,
  String id, {
  Duration timeout = const Duration(seconds: 3),
}) async {
  final deadline = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(deadline)) {
    final item = await repo.getById(id);
    if (item?.thumbPath != null) return item!;
    await Future<void>.delayed(const Duration(milliseconds: 25));
  }
  throw TimeoutException('thumbPath was never set for $id');
}

void main() {
  group('NoopNativeThumbnailProvider', () {
    test('always returns null', () async {
      const provider = NoopNativeThumbnailProvider();
      final result = await provider.request('whatever', sizePx: 256);
      expect(result, isNull);
    });
  });

  group('ClipboardService with NativeThumbnailProvider', () {
    late SqliteRepository repo;
    late Directory tmp;
    late String imagesPath;

    setUp(() async {
      repo = SqliteRepository.inMemory();
      tmp = await Directory.systemTemp.createTemp('cp_native_thumb_');
      imagesPath = p.join(tmp.path, 'images');
      await Directory(imagesPath).create(recursive: true);
    });

    tearDown(() async {
      await repo.close();
      if (tmp.existsSync()) await tmp.delete(recursive: true);
    });

    test('writes native bytes as <id>_thumb.png for video items', () async {
      final stub = _StubProvider()..bytes = _validPng();
      final service = ClipboardService(
        repo,
        imagesPath: imagesPath,
        nativeThumbnailProvider: stub,
      );
      addTearDown(service.dispose);

      // External video file (path must exist + be outside imagesPath).
      final videoPath = p.join(tmp.path, 'sample.mp4');
      await File(videoPath).writeAsBytes([0, 1, 2, 3, 4, 5]);

      final item = await service.processFiles([
        videoPath,
      ], ClipboardContentType.video);
      expect(item, isNotNull);

      final updated = await _waitForThumb(repo, item!.id);
      expect(updated.thumbPath, isNotNull);
      expect(File(updated.thumbPath!).existsSync(), isTrue);
      expect(p.dirname(updated.thumbPath!), equals(imagesPath));
      expect(stub.calls, hasLength(1));
      expect(stub.calls.single.path, equals(videoPath));
    });

    test('does not enqueue when nativeProvider is absent (audio)', () async {
      final service = ClipboardService(repo, imagesPath: imagesPath);
      addTearDown(service.dispose);

      final audioPath = p.join(tmp.path, 'jingle.mp3');
      await File(audioPath).writeAsBytes([0, 1, 2]);

      final item = await service.processFiles([
        audioPath,
      ], ClipboardContentType.audio);
      expect(item, isNotNull);

      // Give the queue a moment in case it would have run.
      await Future<void>.delayed(const Duration(milliseconds: 200));
      final fetched = await repo.getById(item!.id);
      expect(fetched?.thumbPath, isNull);
    });

    test(
      'falls back to Dart pipeline when native returns null on image',
      () async {
        final stub = _StubProvider()..bytes = null;
        final service = ClipboardService(
          repo,
          imagesPath: imagesPath,
          nativeThumbnailProvider: stub,
        );
        addTearDown(service.dispose);

        final imagePath = p.join(tmp.path, 'pic.png');
        await File(imagePath).writeAsBytes(_validPng());

        final item = await service.processFiles([
          imagePath,
        ], ClipboardContentType.image);
        expect(item, isNotNull);

        final updated = await _waitForThumb(repo, item!.id);
        expect(updated.thumbPath, isNotNull);
        expect(File(updated.thumbPath!).existsSync(), isTrue);
        expect(stub.calls, hasLength(1));
      },
    );

    test('swallows native provider errors and falls back', () async {
      final stub = _StubProvider()..throwIt = true;
      final service = ClipboardService(
        repo,
        imagesPath: imagesPath,
        nativeThumbnailProvider: stub,
      );
      addTearDown(service.dispose);

      final imagePath = p.join(tmp.path, 'pic2.png');
      await File(imagePath).writeAsBytes(_validPng());

      final item = await service.processFiles([
        imagePath,
      ], ClipboardContentType.image);
      expect(item, isNotNull);

      final updated = await _waitForThumb(repo, item!.id);
      expect(updated.thumbPath, isNotNull);
      expect(stub.calls, hasLength(1));
    });
  });
}
