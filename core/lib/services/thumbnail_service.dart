import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';

import 'package:image/image.dart' as img;
import 'package:path/path.dart' as p;

import '../models/clipboard_content_type.dart';
import '../models/clipboard_item.dart';
import 'app_logger.dart';
import 'native_thumbnail_provider.dart';

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
/// external media files.
///
/// Two paths:
///   1. **Native** (preferred when `nativeProvider` is set): asks the OS
///      shell for a cached thumbnail (Win `IShellItemImageFactory`,
///      macOS `QLThumbnailGenerator`, Linux `Tumbler`). Covers
///      [ClipboardContentType.image], [video] and [audio] (cover art).
///   2. **Dart fallback** (always available for images): decodes the file
///      with `package:image` in a one-shot isolate. Only handles
///      [ClipboardContentType.image].
///
/// The output file is always written under `imagesPath/<id>_thumb.png`.
/// Snippets we own (paths already inside `imagesPath`) are skipped — they
/// are small enough to render directly.
class ThumbnailService {
  ThumbnailService({
    required this.imagesPath,
    this.nativeProvider,
    this.maxSourceBytes = 25 * 1024 * 1024,
    this.maxDimension = 256,
  });

  /// Absolute, canonicalized path to the app's `images/` directory. Every
  /// generated thumb is written here and only here.
  final String imagesPath;

  /// Optional OS-backed provider tried before the Dart fallback. When set,
  /// the service also accepts video and audio items. When null, only image
  /// items are processed and the Dart fallback is used.
  final NativeThumbnailProvider? nativeProvider;

  /// Skip generation if the source file is bigger than this many bytes.
  /// Default 25 MB matches `maxImageProcessingSizeMB` in Settings (Fase 4).
  /// Only applied to the Dart fallback; native providers handle their own
  /// limits (most just read the OS cache).
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
    if (!_isAcceptedType(item.type)) return null;
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
    if (stat.size <= 0) return null;

    final outPath = p.join(imagesPath, '${item.id}_thumb.png');

    // Defense in depth: outPath must canonicalize back inside imagesPath.
    final canonicalOut = _safeCanonicalize(outPath);
    if (canonicalOut == null || !p.isWithin(canonicalImages, canonicalOut)) {
      AppLogger.error(
        'ThumbnailService: refusing thumb path outside imagesPath: $outPath',
      );
      return null;
    }

    // 1) Try native provider first (cheap cache hit when available).
    final native = nativeProvider;
    if (native != null) {
      final bytes = await _safeNativeRequest(native, sourcePath);
      if (bytes != null && bytes.isNotEmpty) {
        try {
          await File(outPath).writeAsBytes(bytes, flush: true);
          return ThumbnailResult(
            thumbPath: outPath,
            sourceModifiedAt: stat.modified.toUtc(),
          );
        } catch (e, s) {
          AppLogger.warn(
            'ThumbnailService: failed to write native thumb $outPath: $e\n$s',
          );
          // Fall through to Dart fallback (only useful for images).
        }
      }
    }

    // 2) Dart fallback: only images, only within size limit.
    if (item.type != ClipboardContentType.image) return null;
    if (stat.size > maxSourceBytes) return null;

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

  bool _isAcceptedType(ClipboardContentType type) {
    if (type == ClipboardContentType.image) return true;
    if (nativeProvider == null) return false;
    return type == ClipboardContentType.video ||
        type == ClipboardContentType.audio;
  }

  /// Whether the service will attempt to generate a thumbnail for items
  /// of [type]. Visible so callers (e.g. [ThumbnailQueue]) can short-
  /// circuit before enqueuing.
  bool acceptsType(ClipboardContentType type) => _isAcceptedType(type);

  Future<Uint8List?> _safeNativeRequest(
    NativeThumbnailProvider provider,
    String path,
  ) async {
    try {
      return await provider
          .request(path, sizePx: maxDimension)
          .timeout(const Duration(seconds: 2));
    } catch (e, s) {
      AppLogger.warn('ThumbnailService: native provider failed: $e\n$s');
      return null;
    }
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
