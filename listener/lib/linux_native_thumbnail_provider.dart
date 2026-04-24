// coverage:ignore-file
import 'dart:async';
import 'dart:io' show Platform;
import 'dart:ui' as ui show PlatformDispatcher;

import 'package:core/core.dart';
import 'package:flutter/services.dart';

/// Linux-backed [NativeThumbnailProvider]. Bridges to the native handler
/// `getNativeThumbnail` exposed by the listener plugin, which uses
/// `gdk_pixbuf_new_from_file_at_size()` to decode the source image and
/// `gdk_pixbuf_save_to_buffer(... "png")` to encode PNG bytes.
///
/// Backed by GdkPixbuf, which natively decodes PNG/JPEG/BMP/GIF/TIFF/ICO,
/// plus SVG (via librsvg-loader) and any other format with an installed
/// gdk-pixbuf-loader. Video/audio frames are not handled here (would
/// require libavformat); the Dart fallback covers those (returns null →
/// generic type icon).
///
/// This provider is a no-op on non-Linux platforms.
///
/// HiDPI: the requested [sizePx] is multiplied by the platform device
/// pixel ratio so the OS produces a bitmap large enough for the largest
/// connected display. The C side enforces a 64-px minimum heuristic to
/// reject icon-only fallbacks.
class LinuxNativeThumbnailProvider implements NativeThumbnailProvider {
  LinuxNativeThumbnailProvider({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel('copypaste/clipboard_writer');

  final MethodChannel _channel;

  @override
  Future<Uint8List?> request(String path, {int sizePx = 256}) async {
    if (!Platform.isLinux) return null;
    if (path.isEmpty || sizePx <= 0) return null;

    final scaled = (sizePx * _devicePixelRatio()).round().clamp(64, 1024);

    try {
      final result = await _channel.invokeMethod<Object?>(
        'getNativeThumbnail',
        <String, Object?>{'path': path, 'sizePx': scaled},
      );
      if (result is Uint8List && result.isNotEmpty) {
        AppLogger.info(
          '[NativeThumb] OK ${result.length}B for $path (size=$scaled)',
        );
        return result;
      }
      if (result is List<int> && result.isNotEmpty) {
        AppLogger.info(
          '[NativeThumb] OK ${result.length}B for $path (size=$scaled)',
        );
        return Uint8List.fromList(result);
      }
      AppLogger.info('[NativeThumb] empty for $path (size=$scaled)');
      return null;
    } on PlatformException catch (e, s) {
      AppLogger.warn(
        'LinuxNativeThumbnailProvider: platform error: ${e.code} ${e.message}\n$s',
      );
      return null;
    } on MissingPluginException {
      // Plugin not registered (e.g. running in a unit test host without the
      // listener plugin loaded). Quiet fallback.
      return null;
    }
  }

  double _devicePixelRatio() {
    final views = ui.PlatformDispatcher.instance.views;
    if (views.isEmpty) return 1.0;
    var maxRatio = 1.0;
    for (final view in views) {
      if (view.devicePixelRatio > maxRatio) maxRatio = view.devicePixelRatio;
    }
    return maxRatio;
  }
}
