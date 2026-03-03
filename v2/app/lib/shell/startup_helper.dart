import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

typedef _RegOpenKeyExNative = Int32 Function(
    IntPtr hKey, Pointer<Utf16> lpSubKey, Uint32 ulOptions,
    Int32 samDesired, Pointer<IntPtr> phkResult);
typedef _RegOpenKeyExDart = int Function(
    int hKey, Pointer<Utf16> lpSubKey, int ulOptions,
    int samDesired, Pointer<IntPtr> phkResult);

typedef _RegSetValueExNative = Int32 Function(
    IntPtr hKey, Pointer<Utf16> lpValueName, Uint32 reserved,
    Uint32 dwType, Pointer<Utf16> lpData, Uint32 cbData);
typedef _RegSetValueExDart = int Function(
    int hKey, Pointer<Utf16> lpValueName, int reserved,
    int dwType, Pointer<Utf16> lpData, int cbData);

typedef _RegDeleteValueNative = Int32 Function(
    IntPtr hKey, Pointer<Utf16> lpValueName);
typedef _RegDeleteValueDart = int Function(
    int hKey, Pointer<Utf16> lpValueName);

typedef _RegQueryValueExNative = Int32 Function(
    IntPtr hKey, Pointer<Utf16> lpValueName, Pointer<Uint32> lpReserved,
    Pointer<Uint32> lpType, Pointer<Uint8> lpData, Pointer<Uint32> lpcbData);
typedef _RegQueryValueExDart = int Function(
    int hKey, Pointer<Utf16> lpValueName, Pointer<Uint32> lpReserved,
    Pointer<Uint32> lpType, Pointer<Uint8> lpData, Pointer<Uint32> lpcbData);

typedef _RegCloseKeyNative = Int32 Function(IntPtr hKey);
typedef _RegCloseKeyDart = int Function(int hKey);

class StartupHelper {
  static const int _hkeyCurrentUser = 0x80000001;
  static const int _keySetValue = 0x0002;
  static const int _keyQueryValue = 0x0001;
  static const int _regSz = 1;
  static const String _registryPath =
      r'Software\Microsoft\Windows\CurrentVersion\Run';
  static const String _appName = 'CopyPaste';

  static final _advapi32 = DynamicLibrary.open('advapi32.dll');

  static final _regOpenKeyEx =
      _advapi32.lookupFunction<_RegOpenKeyExNative, _RegOpenKeyExDart>(
          'RegOpenKeyExW');
  static final _regSetValueEx =
      _advapi32.lookupFunction<_RegSetValueExNative, _RegSetValueExDart>(
          'RegSetValueExW');
  static final _regDeleteValue =
      _advapi32.lookupFunction<_RegDeleteValueNative, _RegDeleteValueDart>(
          'RegDeleteValueW');
  static final _regQueryValueEx =
      _advapi32.lookupFunction<_RegQueryValueExNative, _RegQueryValueExDart>(
          'RegQueryValueExW');
  static final _regCloseKey =
      _advapi32.lookupFunction<_RegCloseKeyNative, _RegCloseKeyDart>(
          'RegCloseKey');

  static Future<void> apply(bool runOnStartup) async {
    if (!Platform.isWindows) return;

    if (runOnStartup) {
      _setRegistryValue(_getExecutablePath());
    } else {
      _removeRegistryValue();
    }
  }

  static bool isEnabled() {
    if (!Platform.isWindows) return false;
    return _queryRegistryValue() != null;
  }

  static String _getExecutablePath() {
    return Platform.resolvedExecutable;
  }

  static void _setRegistryValue(String exePath) {
    final subKey = _registryPath.toNativeUtf16();
    final hKeyPtr = calloc<IntPtr>();

    try {
      final result = _regOpenKeyEx(
        _hkeyCurrentUser, subKey, 0, _keySetValue, hKeyPtr,
      );
      if (result != 0) return;

      final hKey = hKeyPtr.value;
      final valueName = _appName.toNativeUtf16();
      final valueData = '"$exePath"'.toNativeUtf16();
      final dataSize = ('"$exePath"'.length + 1) * 2;

      try {
        _regSetValueEx(hKey, valueName, 0, _regSz, valueData, dataSize);
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
        _hkeyCurrentUser, subKey, 0, _keySetValue, hKeyPtr,
      );
      if (result != 0) return;

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

  static String? _queryRegistryValue() {
    final subKey = _registryPath.toNativeUtf16();
    final hKeyPtr = calloc<IntPtr>();

    try {
      final result = _regOpenKeyEx(
        _hkeyCurrentUser, subKey, 0, _keyQueryValue, hKeyPtr,
      );
      if (result != 0) return null;

      final hKey = hKeyPtr.value;
      final valueName = _appName.toNativeUtf16();
      final dataSize = calloc<Uint32>();
      dataSize.value = 0;

      try {
        final queryResult = _regQueryValueEx(
          hKey, valueName, nullptr, nullptr, nullptr, dataSize,
        );
        if (queryResult != 0 || dataSize.value == 0) return null;

        final data = calloc<Uint8>(dataSize.value);
        try {
          final readResult = _regQueryValueEx(
            hKey, valueName, nullptr, nullptr, data, dataSize,
          );
          if (readResult != 0) return null;
          return data.cast<Utf16>().toDartString();
        } finally {
          calloc.free(data);
        }
      } finally {
        calloc.free(valueName);
        calloc.free(dataSize);
        _regCloseKey(hKey);
      }
    } finally {
      calloc.free(subKey);
      calloc.free(hKeyPtr);
    }
  }
}
