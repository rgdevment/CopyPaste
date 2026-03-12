import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:image/image.dart' as img;

import 'package:core/services/image_processor.dart';

void main() {
  late Directory tempDir;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('img_proc_test_');
  });

  tearDown(() => tempDir.deleteSync(recursive: true));

  Uint8List makeTestPng({int width = 4, int height = 4}) {
    final image = img.Image(width: width, height: height);
    for (var x = 0; x < width; x++) {
      for (var y = 0; y < height; y++) {
        image.setPixelRgb(x, y, 255, 0, 0);
      }
    }
    return Uint8List.fromList(img.encodePng(image));
  }

  group('ImageProcessResult', () {
    test('stores all properties correctly', () {
      const result = ImageProcessResult(
        imagePath: '/tmp/test.png',
        width: 1920,
        height: 1080,
        fileSize: 98765,
      );
      expect(result.imagePath, equals('/tmp/test.png'));
      expect(result.width, equals(1920));
      expect(result.height, equals(1080));
      expect(result.fileSize, equals(98765));
    });
  });

  group('ImageProcessor.processAndSave', () {
    test('returns result with correct dimensions for valid PNG', () async {
      final pngBytes = makeTestPng(width: 4, height: 4);

      final result = await ImageProcessor.processAndSave(
        imageBytes: pngBytes,
        id: 'test-id',
        imagesDir: tempDir.path,
      );

      expect(result, isNotNull);
      expect(result!.width, equals(4));
      expect(result.height, equals(4));
      expect(result.fileSize, greaterThan(0));
    });

    test('saves PNG file to imagesDir', () async {
      final pngBytes = makeTestPng(width: 2, height: 2);

      final result = await ImageProcessor.processAndSave(
        imageBytes: pngBytes,
        id: 'saved-image',
        imagesDir: tempDir.path,
      );

      expect(result, isNotNull);
      expect(File(result!.imagePath).existsSync(), isTrue);
      expect(result.imagePath, endsWith('saved-image.png'));
    });

    test('saved file is valid PNG', () async {
      final pngBytes = makeTestPng(width: 3, height: 3);

      final result = await ImageProcessor.processAndSave(
        imageBytes: pngBytes,
        id: 'valid-png',
        imagesDir: tempDir.path,
      );

      expect(result, isNotNull);
      final savedBytes = File(result!.imagePath).readAsBytesSync();
      expect(savedBytes.length, greaterThan(0));
      // PNG files start with PNG magic bytes
      expect(savedBytes[0], equals(0x89));
      expect(savedBytes[1], equals(0x50)); // P
      expect(savedBytes[2], equals(0x4E)); // N
      expect(savedBytes[3], equals(0x47)); // G
    });

    test('returns null for invalid image bytes', () async {
      final result = await ImageProcessor.processAndSave(
        imageBytes: Uint8List.fromList([1, 2, 3, 4, 5]),
        id: 'bad-image',
        imagesDir: tempDir.path,
      );

      expect(result, isNull);
    });

    test('returns null for empty bytes', () async {
      final result = await ImageProcessor.processAndSave(
        imageBytes: Uint8List(0),
        id: 'empty-image',
        imagesDir: tempDir.path,
      );

      expect(result, isNull);
    });

    test('imagePath contains the provided id', () async {
      final pngBytes = makeTestPng();

      final result = await ImageProcessor.processAndSave(
        imageBytes: pngBytes,
        id: 'my-custom-id',
        imagesDir: tempDir.path,
      );

      expect(result, isNotNull);
      expect(result!.imagePath, contains('my-custom-id'));
    });

    test('fileSize matches actual saved file size', () async {
      final pngBytes = makeTestPng(width: 8, height: 8);

      final result = await ImageProcessor.processAndSave(
        imageBytes: pngBytes,
        id: 'size-check',
        imagesDir: tempDir.path,
      );

      expect(result, isNotNull);
      final savedSize = File(result!.imagePath).lengthSync();
      expect(result.fileSize, equals(savedSize));
    });
  });
}
