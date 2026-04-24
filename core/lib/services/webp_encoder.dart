import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

import 'app_logger.dart';

// ---------------------------------------------------------------------------
// FFI bindings — only the symbols we need from libwebp's encode API.
// Reference: https://developers.google.com/speed/webp/docs/api
// ---------------------------------------------------------------------------

/// `size_t WebPEncodeLosslessRGBA(const uint8_t* rgba, int width, int height,
///                                int stride, uint8_t** output);`
typedef _WebPEncodeLosslessRGBANative =
    Size Function(
      Pointer<Uint8> rgba,
      Int32 width,
      Int32 height,
      Int32 stride,
      Pointer<Pointer<Uint8>> output,
    );

typedef _WebPEncodeLosslessRGBADart =
    int Function(
      Pointer<Uint8> rgba,
      int width,
      int height,
      int stride,
      Pointer<Pointer<Uint8>> output,
    );

/// `size_t WebPEncodeRGBA(const uint8_t* rgba, int width, int height,
///                        int stride, float quality_factor, uint8_t** output);`
typedef _WebPEncodeRGBANative =
    Size Function(
      Pointer<Uint8> rgba,
      Int32 width,
      Int32 height,
      Int32 stride,
      Float quality,
      Pointer<Pointer<Uint8>> output,
    );

typedef _WebPEncodeRGBADart =
    int Function(
      Pointer<Uint8> rgba,
      int width,
      int height,
      int stride,
      double quality,
      Pointer<Pointer<Uint8>> output,
    );

/// `void WebPFree(void* ptr);`
typedef _WebPFreeNative = Void Function(Pointer<Uint8> ptr);
typedef _WebPFreeDart = void Function(Pointer<Uint8> ptr);

/// `int WebPGetEncoderVersion(void);` — returns version as 0xMMmmpp.
typedef _WebPGetEncoderVersionNative = Int32 Function();
typedef _WebPGetEncoderVersionDart = int Function();

// ---------------------------------------------------------------------------
// Library loading
// ---------------------------------------------------------------------------

/// Wrapper around libwebp's encoder API. Detects availability at startup and
/// falls back gracefully when the native library cannot be loaded.
///
/// Usage:
/// ```dart
/// WebpEncoder.initialize(); // call once at app startup
/// if (WebpEncoder.available) {
///   final bytes = WebpEncoder.encodeLosslessRgba(rgba, w, h);
/// }
/// ```
class WebpEncoder {
  WebpEncoder._(this._lib)
    : _encodeLossless = _lib
          .lookup<NativeFunction<_WebPEncodeLosslessRGBANative>>(
            'WebPEncodeLosslessRGBA',
          )
          .asFunction<_WebPEncodeLosslessRGBADart>(),
      _encodeLossy = _lib
          .lookup<NativeFunction<_WebPEncodeRGBANative>>('WebPEncodeRGBA')
          .asFunction<_WebPEncodeRGBADart>(),
      _free = _lib
          .lookup<NativeFunction<_WebPFreeNative>>('WebPFree')
          .asFunction<_WebPFreeDart>(),
      _version = _lib
          .lookup<NativeFunction<_WebPGetEncoderVersionNative>>(
            'WebPGetEncoderVersion',
          )
          .asFunction<_WebPGetEncoderVersionDart>();

  // ignore: unused_field — kept alive so the DynamicLibrary is not GC'd.
  final DynamicLibrary _lib;
  final _WebPEncodeLosslessRGBADart _encodeLossless;
  final _WebPEncodeRGBADart _encodeLossy;
  final _WebPFreeDart _free;
  final _WebPGetEncoderVersionDart _version;

  static WebpEncoder? _instance;
  static bool _initialized = false;

  /// True if libwebp loaded successfully and is ready to encode.
  static bool get available => _instance != null;

  /// Encoder version as a tuple (major, minor, revision), or `null` if not
  /// available.
  static (int, int, int)? get version {
    final inst = _instance;
    if (inst == null) return null;
    final v = inst._version();
    return ((v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF);
  }

  /// Attempts to load libwebp once. Safe to call multiple times — subsequent
  /// calls are no-ops.
  ///
  /// Searches the platform's standard library locations:
  /// - Windows: `libwebp.dll` next to the executable, then PATH
  /// - macOS:   `libwebp.dylib` in `Frameworks/`, then `@rpath`
  /// - Linux:   `libwebp.so` (system or `LD_LIBRARY_PATH`); also tries
  ///            `libwebp.so.7` (Debian/Ubuntu naming)
  ///
  /// Returns `true` on success.
  static bool initialize() {
    if (_initialized) return _instance != null;
    _initialized = true;

    for (final name in _candidateNames()) {
      try {
        final lib = DynamicLibrary.open(name);
        _instance = WebpEncoder._(lib);
        final v = version;
        AppLogger.info(
          '[WebpEncoder] loaded $name (v${v?.$1}.${v?.$2}.${v?.$3})',
        );
        return true;
      } on ArgumentError catch (_) {
        // Symbol not found in the library — wrong lib, try next candidate.
      } catch (_) {
        // Library not found in this location — try next candidate.
      }
    }

    AppLogger.warn(
      '[WebpEncoder] libwebp not found — falling back to PNG encoding',
    );
    return false;
  }

  static List<String> _candidateNames() {
    if (Platform.isWindows) return const ['libwebp.dll', 'webp.dll'];
    if (Platform.isMacOS) {
      return const [
        'libwebp.dylib',
        '@rpath/libwebp.dylib',
        '/usr/local/lib/libwebp.dylib',
        '/opt/homebrew/lib/libwebp.dylib',
      ];
    }
    if (Platform.isLinux) {
      return const ['libwebp.so.7', 'libwebp.so'];
    }
    return const [];
  }

  // -------------------------------------------------------------------------
  // Public API
  // -------------------------------------------------------------------------

  /// Encodes RGBA pixel data losslessly. Returns the WebP byte stream, or
  /// `null` if encoding failed (e.g. out-of-memory).
  ///
  /// [rgba] must contain `width * height * 4` bytes in RGBA order (no
  /// premultiplied alpha).
  static Uint8List? encodeLosslessRgba(Uint8List rgba, int width, int height) {
    final inst = _instance;
    if (inst == null) return null;
    if (rgba.length < width * height * 4) {
      AppLogger.error('[WebpEncoder] buffer too small: ${rgba.length}');
      return null;
    }
    return inst._encode(rgba, width, height, lossless: true, quality: 100);
  }

  /// Encodes RGBA pixel data with lossy compression. [quality] in 0–100.
  /// Returns the WebP byte stream, or `null` if encoding failed.
  static Uint8List? encodeLossyRgba(
    Uint8List rgba,
    int width,
    int height, {
    double quality = 85,
  }) {
    final inst = _instance;
    if (inst == null) return null;
    if (rgba.length < width * height * 4) {
      AppLogger.error('[WebpEncoder] buffer too small: ${rgba.length}');
      return null;
    }
    return inst._encode(rgba, width, height, lossless: false, quality: quality);
  }

  Uint8List? _encode(
    Uint8List rgba,
    int width,
    int height, {
    required bool lossless,
    required double quality,
  }) {
    final stride = width * 4;
    final input = malloc<Uint8>(rgba.length);
    final output = malloc<Pointer<Uint8>>();
    output.value = nullptr;

    try {
      input.asTypedList(rgba.length).setAll(0, rgba);

      final size = lossless
          ? _encodeLossless(input, width, height, stride, output)
          : _encodeLossy(input, width, height, stride, quality, output);

      if (size == 0 || output.value == nullptr) {
        AppLogger.error('[WebpEncoder] encoding returned 0 bytes');
        return null;
      }

      // Copy to Dart-owned buffer before freeing the libwebp allocation.
      return Uint8List.fromList(output.value.asTypedList(size));
    } catch (e, s) {
      AppLogger.error('[WebpEncoder] encode failed: $e\n$s');
      return null;
    } finally {
      if (output.value != nullptr) _free(output.value);
      malloc.free(input);
      malloc.free(output);
    }
  }
}
