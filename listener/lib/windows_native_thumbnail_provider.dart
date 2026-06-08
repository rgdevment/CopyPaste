// coverage:ignore-file
import 'dart:io' show Platform;

import 'base_native_thumbnail_provider.dart';

/// Windows-backed [BaseNativeThumbnailProvider]. The native handler uses
/// `IShellItemImageFactory::GetImage(SIIGBF_THUMBNAILONLY | SIIGBF_INCACHEONLY)`
/// and re-encodes the resulting bitmap as PNG before returning the bytes.
/// The C++ side enforces a 64-px minimum heuristic to reject generic
/// file-type icons.
class WindowsNativeThumbnailProvider extends BaseNativeThumbnailProvider {
  WindowsNativeThumbnailProvider({super.channel});

  @override
  bool get isCurrentPlatform => Platform.isWindows;

  @override
  String get debugLabel => 'WindowsNativeThumbnailProvider';
}
