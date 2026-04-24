import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';
import 'app_logger.dart';

/// Result of a thumbnail generation attempt.
class ThumbnailResult {
  const ThumbnailResult({
    required this.thumbPath,
    required this.sourceModifiedAt,
  });

  /// Path inside `imagesPath`, of the form `<id>_thumb.png`.
  final String thumbPath;

  /// `mtime` (UTC) of the source file at the time the thumb was generated.
  /// Used to detect staleness when the external file is modified.
  final DateTime sourceModifiedAt;
}

/// Generates 256-px PNG thumbnails for clipboard items that reference
/// external image files.
///
/// Scope of this implementation (PR #4):
///   - Only [ClipboardContentType.image] with a single existing local path.
///   - Writes `<imagesPath>/<id>_thumb.png` (always inside the app data dir).
///
/// Out of scope (later PRs):
///   - Video / audio / generic file thumbs (require OS shell APIs).
///   - Native cache lookup (Win `IShellItemImageFactory`,
///     Mac `QLThumbnailGenerator`, Linux `Tumbler`).
///   - Snippets stored as `<id>.png` already inside `imagesPath` — those
///     are small enough to be rendered directly with `Image.file(content)`.
class ThumbnailService {
  ThumbnailService({
    required this.imagesPath,
    this.maxSourceBytes = 25 * 1024 * 1024,
    this.maxDimension = 256,
  });

  /// Absolute, canonicalized path to the app's `images/` directory. Every
  /// generated thumb is written here and only here.
  final String imagesPath;

  /// Skip generation if the source file is bigger than this many bytes.
  /// Default 25 MB matches `maxImageProcessingSizeMB` in Settings (Fase 4).
  final int maxSourceBytes;

  /// Longest side of the generated thumbnail, in pixels.
  final int maxDimension;

  /// Generates a thumbnail for [item] if applicable. Returns the result
  /// metadata so the caller can persist `thumbPath` + `sourceModifiedAt`
  /// in the repository, or `null` if no thumb was produced.
  ///
  /// This method is safe to call from the UI thread: heavy work runs in
  /// a one-shot isolate via `Isolate.run`.
  Future<ThumbnailResult?> generateForItem(ClipboardItem item) async {
    if (item.type != ClipboardContentType.image) return null;
    if (item.content.isEmpty) return null;

    final paths = item.content.split('\n').where((s) => s.isNotEmpty).toList();
    if (paths.length != 1) return null;

    final sourcePath = paths.single;

    // Skip snippets we own: they already live inside imagesPath and are
    // typically small enough to render directly. They are also the
    // output of the image processing queue, so generating a thumb of a
    // thumb is wasteful.
    final canonicalSource = _safeCanonicalize(sourcePath);
    final canonicalImages = _safeCanonicalize(imagesPath);
    if (canonicalSource == null || canonicalImages == null) return null;
    if (p.isWithin(canonicalImages, canonicalSource)) return null;

    final sourceFile = File(sourcePath);
    if (!sourceFile.existsSync()) return null;

    final FileStat stat;
    try {
      stat = sourceFile.statSync();
    } catch (e) {
      AppLogger.warn('ThumbnailService: stat failed for $sourcePath: $e');
      return null;
    }
    if (stat.size <= 0 || stat.size > maxSourceBytes) return null;

    final outPath = p.join(imagesPath, '${item.id}_thumb.png');

    // Defense in depth: outPath must canonicalize back inside imagesPath.
    final canonicalOut = _safeCanonicalize(outPath);
    if (canonicalOut == null || !p.isWithin(canonicalImages, canonicalOut)) {
      AppLogger.error(
        'ThumbnailService: refusing thumb path outside imagesPath: $outPath',
      );
      return null;
    }

    final bytes = await sourceFile.readAsBytes();

    final ok = await Isolate.run(
      () => _encodeThumbSync(
        bytes: bytes,
        outPath: outPath,
        maxDimension: maxDimension,
      ),
    );
    if (!ok) return null;

    return ThumbnailResult(
      thumbPath: outPath,
      sourceModifiedAt: stat.modified.toUtc(),
    );
  }

  /// Decode + downscale + encode PNG, all synchronous. Designed to run
  /// inside an isolate. Returns `true` on success, `false` if decoding
  /// failed; rethrows on I/O failures so the caller can log them.
  static bool _encodeThumbSync({
    required Uint8List bytes,
    required String outPath,
    required int maxDimension,
  }) {
    final img.Image? decoded;
    try {
      decoded = img.decodeImage(bytes);
    } catch (_) {
      return false;
    }
    if (decoded == null) return false;

    final scaled = _downscale(decoded, maxDimension);
    final pngBytes = img.encodePng(scaled);
    File(outPath).writeAsBytesSync(pngBytes);
    return true;
  }

  static img.Image _downscale(img.Image src, int maxDim) {
    final w = src.width;
    final h = src.height;
    if (w <= maxDim && h <= maxDim) return src;
    if (w >= h) {
      return img.copyResize(src, width: maxDim);
    }
    return img.copyResize(src, height: maxDim);
  }

  static String? _safeCanonicalize(String path) {
    try {
      return p.canonicalize(path);
    } catch (_) {
      return null;
    }
  }
}
