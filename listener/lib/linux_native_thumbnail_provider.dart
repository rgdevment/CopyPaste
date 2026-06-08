// coverage:ignore-file
import 'dart:io' show Platform;

import 'base_native_thumbnail_provider.dart';

/// Linux-backed [BaseNativeThumbnailProvider]. The native handler uses
/// `gdk_pixbuf_new_from_file_at_size()` to decode the source and
/// `gdk_pixbuf_save_to_buffer(... "png")` to encode PNG bytes.
///
/// GdkPixbuf natively decodes PNG/JPEG/BMP/GIF/TIFF/ICO, plus SVG (via
/// librsvg-loader). Video/audio frames are not handled here (would require
/// libavformat); the Dart fallback covers those (returns null → type icon).
class LinuxNativeThumbnailProvider extends BaseNativeThumbnailProvider {
  LinuxNativeThumbnailProvider({super.channel});

  @override
  bool get isCurrentPlatform => Platform.isLinux;

  @override
  String get debugLabel => 'LinuxNativeThumbnailProvider';
}
