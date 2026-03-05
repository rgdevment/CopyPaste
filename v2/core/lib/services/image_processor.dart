import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

class ImageProcessResult {
  const ImageProcessResult({
    required this.imagePath,
    required this.thumbPath,
    required this.width,
    required this.height,
    required this.thumbWidth,
    required this.thumbHeight,
    required this.fileSize,
  });

  final String imagePath;
  final String thumbPath;
  final int width;
  final int height;
  final int thumbWidth;
  final int thumbHeight;
  final int fileSize;
}

class ImageProcessor {
  static Future<ImageProcessResult?> processAndSave({
    required Uint8List imageBytes,
    required String id,
    required String imagesDir,
    required String thumbsDir,
    int thumbnailWidth = 200,
    int thumbnailQuality = 80,
  }) async {
    return Isolate.run(() => _processSync(
          imageBytes: imageBytes,
          id: id,
          imagesDir: imagesDir,
          thumbsDir: thumbsDir,
          thumbnailWidth: thumbnailWidth,
          thumbnailQuality: thumbnailQuality,
        ));
  }

  static ImageProcessResult? _processSync({
    required Uint8List imageBytes,
    required String id,
    required String imagesDir,
    required String thumbsDir,
    required int thumbnailWidth,
    required int thumbnailQuality,
  }) {
    try {
      final decoded = img.decodeImage(imageBytes);
      if (decoded == null) return null;

      // Save original as PNG
      final pngBytes = img.encodePng(decoded);
      final imagePath = p.join(imagesDir, '$id.png');
      File(imagePath).writeAsBytesSync(pngBytes);

      // Generate thumbnail
      final thumb = img.copyResize(decoded, width: thumbnailWidth);
      final thumbBytes = img.encodePng(thumb);
      final thumbPath = p.join(thumbsDir, '${id}_t.png');
      File(thumbPath).writeAsBytesSync(thumbBytes);

      return ImageProcessResult(
        imagePath: imagePath,
        thumbPath: thumbPath,
        width: decoded.width,
        height: decoded.height,
        thumbWidth: thumb.width,
        thumbHeight: thumb.height,
        fileSize: pngBytes.length,
      );
    } catch (_) {
      return null;
    }
  }
}
