import 'dart:io';
import 'dart:typed_data';

import 'package:core/models/clipboard_content_type.dart';
import 'package:core/models/clipboard_item.dart';
import 'package:core/services/native_thumbnail_provider.dart';
import 'package:core/services/thumbnail_service.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

void main() {
  late Directory tempDir;
  late Directory imagesDir;
  late Directory externalDir;
  late ThumbnailService service;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('thumb_svc_test_');
    imagesDir = Directory(p.join(tempDir.path, 'images'))
      ..createSync(recursive: true);
    externalDir = Directory(p.join(tempDir.path, 'external'))
      ..createSync(recursive: true);
    service = ThumbnailService(imagesPath: imagesDir.path);
  });

  tearDown(() {
    if (tempDir.existsSync()) tempDir.deleteSync(recursive: true);
  });

  Uint8List makePng({int width = 64, int height = 32}) {
    final image = img.Image(width: width, height: height);
    for (var x = 0; x < width; x++) {
      for (var y = 0; y < height; y++) {
        image.setPixelRgb(x, y, x * 4, y * 8, 128);
      }
    }
    return Uint8List.fromList(img.encodePng(image));
  }

  ClipboardItem imageItem(String externalPath, {String id = 'item-1'}) =>
      ClipboardItem(
        id: id,
        content: externalPath,
        type: ClipboardContentType.image,
      );

  group('ThumbnailService.generateForItem', () {
    test('produces 256-px PNG for large external image', () async {
      final src = File(p.join(externalDir.path, 'big.png'))
        ..writeAsBytesSync(makePng(width: 1024, height: 512));

      final result = await service.generateForItem(imageItem(src.path));

      expect(result, isNotNull);
      expect(
        result!.thumbPath,
        equals(p.join(imagesDir.path, 'item-1_thumb.png')),
      );
      expect(File(result.thumbPath).existsSync(), isTrue);

      final decoded = img.decodePng(File(result.thumbPath).readAsBytesSync());
      expect(decoded, isNotNull);
      expect(decoded!.width, equals(256));
      expect(decoded.height, equals(128));
    });

    test('does not upscale small images', () async {
      final src = File(p.join(externalDir.path, 'small.png'))
        ..writeAsBytesSync(makePng(width: 64, height: 32));

      final result = await service.generateForItem(imageItem(src.path));

      expect(result, isNotNull);
      final decoded = img.decodePng(File(result!.thumbPath).readAsBytesSync());
      expect(decoded!.width, equals(64));
      expect(decoded.height, equals(32));
    });

    test('records source mtime in result', () async {
      final src = File(p.join(externalDir.path, 'mtime.png'))
        ..writeAsBytesSync(makePng());
      final mtime = src.statSync().modified.toUtc();

      final result = await service.generateForItem(imageItem(src.path));

      expect(result, isNotNull);
      expect(result!.sourceModifiedAt, equals(mtime));
    });

    test('returns null for non-image items', () async {
      final src = File(p.join(externalDir.path, 'unused.png'))
        ..writeAsBytesSync(makePng());
      final item = ClipboardItem(
        id: 'text-1',
        content: src.path,
        type: ClipboardContentType.text,
      );

      expect(await service.generateForItem(item), isNull);
    });

    test('returns null when source file does not exist', () async {
      final item = imageItem(p.join(externalDir.path, 'missing.png'));
      expect(await service.generateForItem(item), isNull);
    });

    test('returns null when content is empty', () async {
      final item = ClipboardItem(
        id: 'empty',
        content: '',
        type: ClipboardContentType.image,
      );
      expect(await service.generateForItem(item), isNull);
    });

    test(
      'returns null for multi-path content (drag of multiple files)',
      () async {
        final a = File(p.join(externalDir.path, 'a.png'))
          ..writeAsBytesSync(makePng());
        final b = File(p.join(externalDir.path, 'b.png'))
          ..writeAsBytesSync(makePng());
        final item = ClipboardItem(
          id: 'multi',
          content: '${a.path}\n${b.path}',
          type: ClipboardContentType.image,
        );

        expect(await service.generateForItem(item), isNull);
      },
    );

    test('skips snippets owned by imagesPath', () async {
      // A snippet captured by the image processing queue would already
      // live inside imagesPath. We do not create thumbs for those.
      final snippet = File(p.join(imagesDir.path, 'snippet.png'))
        ..writeAsBytesSync(makePng(width: 1024, height: 1024));
      final item = imageItem(snippet.path, id: 'snip');

      expect(await service.generateForItem(item), isNull);
      expect(
        File(p.join(imagesDir.path, 'snip_thumb.png')).existsSync(),
        isFalse,
      );
    });

    test('returns null for unreadable / non-image bytes', () async {
      final src = File(p.join(externalDir.path, 'garbage.png'))
        ..writeAsBytesSync(Uint8List.fromList(List.filled(64, 0xAB)));

      expect(await service.generateForItem(imageItem(src.path)), isNull);
    });

    test('returns null when source exceeds maxSourceBytes', () async {
      final smallService = ThumbnailService(
        imagesPath: imagesDir.path,
        maxSourceBytes: 16,
      );
      final src = File(p.join(externalDir.path, 'too_big.png'))
        ..writeAsBytesSync(makePng(width: 32, height: 32));

      expect(await smallService.generateForItem(imageItem(src.path)), isNull);
    });

    test('writes thumb only inside imagesPath', () async {
      final src = File(p.join(externalDir.path, 'safe.png'))
        ..writeAsBytesSync(makePng());

      final result = await service.generateForItem(imageItem(src.path));
      expect(result, isNotNull);
      expect(
        p.isWithin(imagesDir.path, result!.thumbPath),
        isTrue,
        reason: 'thumb must live inside imagesPath',
      );
    });
  });

  group('ThumbnailService isTypeEnabled gate (PR #10)', () {
    test('skips generation when callback returns false for the type', () async {
      service.isTypeEnabled = (_) => false;
      final src = File(p.join(externalDir.path, 'gated.png'))
        ..writeAsBytesSync(makePng());

      final result = await service.generateForItem(imageItem(src.path));

      expect(result, isNull);
      expect(service.acceptsType(ClipboardContentType.image), isFalse);
    });

    test('proceeds when callback returns true', () async {
      service.isTypeEnabled = (_) => true;
      final src = File(p.join(externalDir.path, 'allowed.png'))
        ..writeAsBytesSync(makePng());

      final result = await service.generateForItem(imageItem(src.path));

      expect(result, isNotNull);
      expect(service.acceptsType(ClipboardContentType.image), isTrue);
    });

    test('mutating the gate is honored on the next call', () async {
      final src = File(p.join(externalDir.path, 'mutating.png'))
        ..writeAsBytesSync(makePng());

      service.isTypeEnabled = (_) => false;
      expect(await service.generateForItem(imageItem(src.path)), isNull);

      service.isTypeEnabled = (_) => true;
      expect(await service.generateForItem(imageItem(src.path)), isNotNull);
    });
  });

  group('ThumbnailService.acceptsType with nativeProvider', () {
    test('returns true for audio when nativeProvider is set', () {
      final nativeService = ThumbnailService(
        imagesPath: imagesDir.path,
        nativeProvider: const NoopNativeThumbnailProvider(),
      );
      expect(nativeService.acceptsType(ClipboardContentType.audio), isTrue);
    });
  });

  group('ThumbnailService._downscale portrait image', () {
    test('constrains height when image is taller than maxDimension', () async {
      // Portrait: width=64, height=512 — height > maxDimension(256) so
      // _downscale uses copyResize(src, height: maxDim) branch (line 215).
      final portraitImage = img.Image(width: 64, height: 512);
      for (var x = 0; x < 64; x++) {
        for (var y = 0; y < 512; y++) {
          portraitImage.setPixelRgb(x, y, x * 4, y ~/ 2, 128);
        }
      }
      final pngBytes = Uint8List.fromList(img.encodePng(portraitImage));
      final src = File(p.join(externalDir.path, 'portrait.png'))
        ..writeAsBytesSync(pngBytes);

      final result = await service.generateForItem(imageItem(src.path));

      expect(result, isNotNull);
      final thumb = img.decodePng(await File(result!.thumbPath).readAsBytes())!;
      expect(thumb.height, lessThanOrEqualTo(256));
      expect(thumb.height, greaterThan(thumb.width));
    });
  });
}
