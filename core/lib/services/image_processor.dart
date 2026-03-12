import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

class ImageProcessResult {
  const ImageProcessResult({
    required this.imagePath,
    required this.width,
    required this.height,
    required this.fileSize,
  });

  final String imagePath;
  final int width;
  final int height;
  final int fileSize;
}

class ImageProcessor {
  static Future<ImageProcessResult?> processAndSave({
    required Uint8List imageBytes,
    required String id,
    required String imagesDir,
  }) async {
    return Isolate.run(
      () => _processSync(imageBytes: imageBytes, id: id, imagesDir: imagesDir),
    );
  }

  static ImageProcessResult? _processSync({
    required Uint8List imageBytes,
    required String id,
    required String imagesDir,
  }) {
    try {
      final decoded = img.decodeImage(imageBytes);
      if (decoded == null) return null;

      final pngBytes = img.encodePng(decoded);
      final imagePath = p.join(imagesDir, '$id.png');
      File(imagePath).writeAsBytesSync(pngBytes);

      return ImageProcessResult(
        imagePath: imagePath,
        width: decoded.width,
        height: decoded.height,
        fileSize: pngBytes.length,
      );
    } catch (_) {
      return null;
    }
  }
}
