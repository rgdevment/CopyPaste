// coverage:ignore-file
import 'dart:async';
import 'dart:io' show Platform;
import 'dart:ui' as ui show PlatformDispatcher;

import 'package:core/core.dart';
import 'package:flutter/services.dart';

/// macOS-backed [NativeThumbnailProvider]. Bridges to the native handler
/// `getNativeThumbnail` exposed by the listener plugin, which uses
/// `QLThumbnailGenerator.generateBestRepresentation(for:)` and re-encodes
/// the resulting representation as PNG before returning the bytes.
///
/// This provider is a no-op on non-macOS platforms — the call returns
/// `null` immediately so the queue can fall back to the Dart pipeline.
///
/// HiDPI: the requested [sizePx] is multiplied by the platform device
/// pixel ratio so the OS produces a bitmap large enough for the largest
/// connected display. The Swift side passes that as the pixel size with
/// `scale = 1.0` to mirror Windows behavior.
///
/// TCC: when macOS denies access to the source file (`~/Documents`,
/// `~/Downloads`, `~/Desktop`, iCloud Drive, etc.), the native handler
/// surfaces a `permissionDenied` PlatformException. We log a distinct
/// warning so the upstream UI can render a TCC-specific message instead
/// of confusing the user with a generic "file not found".
class MacOSNativeThumbnailProvider implements NativeThumbnailProvider {
  MacOSNativeThumbnailProvider({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel('copypaste/clipboard_writer');

  final MethodChannel _channel;

  @override
  Future<Uint8List?> request(String path, {int sizePx = 256}) async {
    if (!Platform.isMacOS) return null;
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
      if (e.code == 'permissionDenied') {
        AppLogger.warn('[NativeThumb] TCC denied for $path: ${e.message}');
      } else {
        AppLogger.warn(
          'MacOSNativeThumbnailProvider: platform error: ${e.code} ${e.message}\n$s',
        );
      }
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
