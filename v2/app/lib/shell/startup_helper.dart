// coverage:ignore-file
import 'dart:ffi';
import 'dart:io';

import 'package:core/core.dart';
import 'package:ffi/ffi.dart';

typedef _RegOpenKeyExNative =
    Int32 Function(
      IntPtr hKey,
      Pointer<Utf16> lpSubKey,
      Uint32 ulOptions,
      Int32 samDesired,
      Pointer<IntPtr> phkResult,
    );
typedef _RegOpenKeyExDart =
    int Function(
      int hKey,
      Pointer<Utf16> lpSubKey,
      int ulOptions,
      int samDesired,
      Pointer<IntPtr> phkResult,
    );

typedef _RegSetValueExNative =
    Int32 Function(
      IntPtr hKey,
      Pointer<Utf16> lpValueName,
      Uint32 reserved,
      Uint32 dwType,
      Pointer<Utf16> lpData,
      Uint32 cbData,
    );
typedef _RegSetValueExDart =
    int Function(
      int hKey,
      Pointer<Utf16> lpValueName,
      int reserved,
      int dwType,
      Pointer<Utf16> lpData,
      int cbData,
    );

typedef _RegDeleteValueNative =
    Int32 Function(IntPtr hKey, Pointer<Utf16> lpValueName);
typedef _RegDeleteValueDart =
    int Function(int hKey, Pointer<Utf16> lpValueName);

typedef _RegCloseKeyNative = Int32 Function(IntPtr hKey);
typedef _RegCloseKeyDart = int Function(int hKey);

class StartupHelper {
  static const int _hkeyCurrentUser = 0x80000001;
  static const int _keySetValue = 0x0002;
  static const int _regSz = 1;
  static const String _registryPath =
      r'Software\Microsoft\Windows\CurrentVersion\Run';
  static const String _appName = 'CopyPaste';

  static final _advapi32 = DynamicLibrary.open('advapi32.dll');

  static final _regOpenKeyEx = _advapi32
      .lookupFunction<_RegOpenKeyExNative, _RegOpenKeyExDart>('RegOpenKeyExW');
  static final _regSetValueEx = _advapi32
      .lookupFunction<_RegSetValueExNative, _RegSetValueExDart>(
        'RegSetValueExW',
      );
  static final _regDeleteValue = _advapi32
      .lookupFunction<_RegDeleteValueNative, _RegDeleteValueDart>(
        'RegDeleteValueW',
      );
  static final _regCloseKey = _advapi32
      .lookupFunction<_RegCloseKeyNative, _RegCloseKeyDart>('RegCloseKey');

  static Future<void> apply(bool runOnStartup) async {
    if (!Platform.isWindows) return;

    if (runOnStartup) {
      _setRegistryValue(_getExecutablePath());
    } else {
      _removeRegistryValue();
    }
  }

  static String _getExecutablePath() {
    return Platform.resolvedExecutable;
  }

  static void _setRegistryValue(String exePath) {
    final subKey = _registryPath.toNativeUtf16();
    final hKeyPtr = calloc<IntPtr>();

    try {
      final result = _regOpenKeyEx(
        _hkeyCurrentUser,
        subKey,
        0,
        _keySetValue,
        hKeyPtr,
      );
      if (result != 0) {
        AppLogger.error('Failed to open registry key for set: $result');
        return;
      }

      final hKey = hKeyPtr.value;
      final valueName = _appName.toNativeUtf16();
      final valueData = '"$exePath"'.toNativeUtf16();
      final dataSize = ('"$exePath"'.length + 1) * 2;

      try {
        final setResult = _regSetValueEx(
          hKey,
          valueName,
          0,
          _regSz,
          valueData,
          dataSize,
        );
        if (setResult != 0) {
          AppLogger.error('Failed to set registry value: $setResult');
        }
      } finally {
        calloc.free(valueName);
        calloc.free(valueData);
        _regCloseKey(hKey);
      }
    } finally {
      calloc.free(subKey);
      calloc.free(hKeyPtr);
    }
  }

  static void _removeRegistryValue() {
    final subKey = _registryPath.toNativeUtf16();
    final hKeyPtr = calloc<IntPtr>();

    try {
      final result = _regOpenKeyEx(
        _hkeyCurrentUser,
        subKey,
        0,
        _keySetValue,
        hKeyPtr,
      );
      if (result != 0) {
        AppLogger.error('Failed to open registry key for delete: $result');
        return;
      }

      final hKey = hKeyPtr.value;
      final valueName = _appName.toNativeUtf16();

      try {
        _regDeleteValue(hKey, valueName);
      } finally {
        calloc.free(valueName);
        _regCloseKey(hKey);
      }
    } finally {
      calloc.free(subKey);
      calloc.free(hKeyPtr);
    }
  }
}
