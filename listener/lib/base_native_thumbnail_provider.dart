// coverage:ignore-file
import 'dart:async';
import 'dart:ui' as ui show PlatformDispatcher;

import 'package:core/core.dart';
import 'package:flutter/services.dart';

/// Shared bridge to the native `getNativeThumbnail` handler. Subclasses only
/// declare which platform they serve, a debug label, and (optionally) how to
/// map a platform-specific error code to a dedicated log line.
abstract class BaseNativeThumbnailProvider implements NativeThumbnailProvider {
  BaseNativeThumbnailProvider({MethodChannel? channel})
    : channel = channel ?? const MethodChannel('copypaste/clipboard_writer');

  final MethodChannel channel;

  bool get isCurrentPlatform;

  String get debugLabel;

  /// Returns true when the error was already logged with a dedicated message,
  /// so the generic warning is skipped.
  bool handlePlatformException(PlatformException e, String path) => false;

  @override
  Future<Uint8List?> request(String path, {int sizePx = 256}) async {
    if (!isCurrentPlatform) return null;
    if (path.isEmpty || sizePx <= 0) return null;

    final scaled = (sizePx * _maxDevicePixelRatio()).round().clamp(64, 1024);

    try {
      final result = await channel.invokeMethod<Object?>(
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
      if (!handlePlatformException(e, path)) {
        AppLogger.warn(
          '$debugLabel: platform error: ${e.code} ${e.message}\n$s',
        );
      }
      return null;
    } on MissingPluginException {
      return null;
    }
  }

  /// Largest device pixel ratio across connected displays, so the OS produces a
  /// bitmap big enough for the sharpest screen.
  double _maxDevicePixelRatio() {
    final views = ui.PlatformDispatcher.instance.views;
    if (views.isEmpty) return 1.0;
    var maxRatio = 1.0;
    for (final view in views) {
      if (view.devicePixelRatio > maxRatio) maxRatio = view.devicePixelRatio;
    }
    return maxRatio;
  }
}
