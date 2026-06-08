// coverage:ignore-file
import 'dart:io' show Platform;

import 'package:core/core.dart';
import 'package:flutter/services.dart';

import 'base_native_thumbnail_provider.dart';

/// macOS-backed [BaseNativeThumbnailProvider]. The native handler uses
/// `QLThumbnailGenerator.generateBestRepresentation(for:)` and re-encodes the
/// representation as PNG.
///
/// TCC: when macOS denies access to the source file (`~/Documents`,
/// `~/Downloads`, `~/Desktop`, iCloud Drive, etc.), the native handler surfaces
/// a `permissionDenied` PlatformException; it is logged distinctly so the UI
/// can render a TCC-specific message instead of a generic "file not found".
class MacOSNativeThumbnailProvider extends BaseNativeThumbnailProvider {
  MacOSNativeThumbnailProvider({super.channel});

  @override
  bool get isCurrentPlatform => Platform.isMacOS;

  @override
  String get debugLabel => 'MacOSNativeThumbnailProvider';

  @override
  bool handlePlatformException(PlatformException e, String path) {
    if (e.code == 'permissionDenied') {
      AppLogger.warn('[NativeThumb] TCC denied for $path: ${e.message}');
      return true;
    }
    return false;
  }
}
