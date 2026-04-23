// coverage:ignore-file
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

typedef _GetCurrentPackageFullNameNative =
    Int32 Function(Pointer<Uint32> packageFullNameLength, Pointer<Utf16> packageFullName);
typedef _GetCurrentPackageFullNameDart =
    int Function(Pointer<Uint32> packageFullNameLength, Pointer<Utf16> packageFullName);

class WinPackageContext {
  WinPackageContext._();

  static const int _appmodelErrorNoPackage = 15700;
  static const int _errorInsufficientBuffer = 122;

  static bool? _cachedIsMsix;
  static String? _cachedPackageFullName;

  static bool get isMsix {
    if (!Platform.isWindows) return false;
    return _cachedIsMsix ??= _detect().$1;
  }

  static String? get packageFullName {
    if (!Platform.isWindows) return null;
    if (_cachedIsMsix == null) _detect();
    return _cachedPackageFullName;
  }

  static (bool, String?) _detect() {
    try {
      final kernel32 = DynamicLibrary.open('kernel32.dll');
      final getCurrentPackageFullName = kernel32.lookupFunction<
          _GetCurrentPackageFullNameNative,
          _GetCurrentPackageFullNameDart>('GetCurrentPackageFullName');

      final lenPtr = calloc<Uint32>();
      try {
        lenPtr.value = 0;
        final probe = getCurrentPackageFullName(lenPtr, nullptr);
        if (probe == _appmodelErrorNoPackage) {
          _cachedIsMsix = false;
          _cachedPackageFullName = null;
          return (false, null);
        }
        if (probe != _errorInsufficientBuffer && probe != 0) {
          _cachedIsMsix = false;
          _cachedPackageFullName = null;
          return (false, null);
        }

        final bufLen = lenPtr.value;
        if (bufLen == 0) {
          _cachedIsMsix = true;
          _cachedPackageFullName = null;
          return (true, null);
        }

        final namePtr = calloc<Uint16>(bufLen).cast<Utf16>();
        try {
          final result = getCurrentPackageFullName(lenPtr, namePtr);
          if (result != 0) {
            _cachedIsMsix = false;
            _cachedPackageFullName = null;
            return (false, null);
          }
          final name = namePtr.toDartString();
          _cachedIsMsix = true;
          _cachedPackageFullName = name;
          return (true, name);
        } finally {
          calloc.free(namePtr);
        }
      } finally {
        calloc.free(lenPtr);
      }
    } catch (_) {
      _cachedIsMsix = false;
      _cachedPackageFullName = null;
      return (false, null);
    }
  }
}
