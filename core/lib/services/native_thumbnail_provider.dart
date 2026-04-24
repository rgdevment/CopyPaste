import 'dart:typed_data';

/// Contract for OS-backed thumbnail providers. Implementations request a
/// thumbnail bitmap from the native shell (Windows `IShellItemImageFactory`,
/// macOS `QLThumbnailGenerator`, Linux `Tumbler`) and return the encoded
/// PNG bytes ready to be written to disk.
///
/// Implementations are expected to:
///   - Return `null` when the OS has no usable thumbnail (no error).
///   - Treat icon-only fallbacks as `null` (e.g. discard if the bitmap is
///     ≤ 64 px on either side when 256 px were requested).
///   - Time out fast (≤ 2 s) so the queue stays responsive.
///   - Never throw for "missing thumb" cases. Throw only for genuine
///     programming errors (invalid arguments, channel not registered).
///
/// Returned bytes must be a valid PNG. The caller writes them verbatim
/// inside the app's `images/` directory and is responsible for the path
/// safety checks.
abstract class NativeThumbnailProvider {
  /// Requests a thumbnail of [path] sized at [sizePx] on the longest side.
  /// HiDPI scaling is the implementation's responsibility.
  Future<Uint8List?> request(String path, {int sizePx = 256});
}

/// Default no-op implementation. Used on platforms with no native backend
/// wired yet, and in tests that want to exercise only the Dart fallback.
class NoopNativeThumbnailProvider implements NativeThumbnailProvider {
  const NoopNativeThumbnailProvider();

  @override
  Future<Uint8List?> request(String path, {int sizePx = 256}) async => null;
}
