import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

import 'webp_encoder.dart';

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

    // Try WebP first (smaller files); fall back to PNG if libwebp is missing
    // or the encode failed for any reason.
    //
    // initialize() is idempotent and cheap; it must run inside the isolate
    // because static state is not shared across isolates.
    if (WebpEncoder.initialize()) {
      final webpResult = _encodeWebp(decoded, id, imagesDir);
      if (webpResult != null) return webpResult;
      // Encode failed silently → fall through to PNG below.
    }

    return _encodePng(decoded, id, imagesDir);
  }

  static ImageProcessResult? _encodeWebp(
    img.Image decoded,
    String id,
    String imagesDir,
  ) {
    try {
      // package:image stores RGBA in `getBytes(order: ChannelOrder.rgba)`.
      final rgba = decoded.getBytes(order: img.ChannelOrder.rgba);
      final webpBytes = WebpEncoder.encodeLosslessRgba(
        rgba,
        decoded.width,
        decoded.height,
      );
      if (webpBytes == null) return null;

      final imagePath = p.join(imagesDir, '$id.webp');
      File(imagePath).writeAsBytesSync(webpBytes);

      return ImageProcessResult(
        imagePath: imagePath,
        width: decoded.width,
        height: decoded.height,
        fileSize: webpBytes.length,
      );
    } catch (_) {
      return null;
    }
  }

  static ImageProcessResult _encodePng(
    img.Image decoded,
    String id,
    String imagesDir,
  ) {
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
