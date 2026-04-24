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
      () => processSync(imageBytes: imageBytes, id: id, imagesDir: imagesDir),
    );
  }

  static ImageProcessResult? processSync({
    required Uint8List imageBytes,
    required String id,
    required String imagesDir,
  }) {
    // Decode failures (unsupported format, truncated data) → return null so the
    // caller can distinguish "format not supported" from I/O errors below.
    final img.Image? decoded;
    try {
      decoded = img.decodeImage(imageBytes);
    } catch (_) {
      return null;
    }
    if (decoded == null) return null;

    // Encoding and file-write errors (disk full, permissions) are NOT silenced —
    // they propagate out of the Isolate and are caught + logged by the caller.
    final pngBytes = img.encodePng(decoded);
    final imagePath = p.join(imagesDir, '$id.png');
    File(imagePath).writeAsBytesSync(pngBytes);

    return ImageProcessResult(
      imagePath: imagePath,
      width: decoded.width,
      height: decoded.height,
      fileSize: pngBytes.length,
    );
  }
}
