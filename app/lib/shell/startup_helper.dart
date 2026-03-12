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

class _Win32Registry {
  _Win32Registry._();
  static _Win32Registry? _instance;
  static _Win32Registry get instance => _instance ??= _Win32Registry._();

  late final _advapi32 = DynamicLibrary.open('advapi32.dll');

  late final regOpenKeyEx = _advapi32
      .lookupFunction<_RegOpenKeyExNative, _RegOpenKeyExDart>('RegOpenKeyExW');
  late final regSetValueEx = _advapi32
      .lookupFunction<_RegSetValueExNative, _RegSetValueExDart>(
        'RegSetValueExW',
      );
  late final regDeleteValue = _advapi32
      .lookupFunction<_RegDeleteValueNative, _RegDeleteValueDart>(
        'RegDeleteValueW',
      );
  late final regCloseKey = _advapi32
      .lookupFunction<_RegCloseKeyNative, _RegCloseKeyDart>('RegCloseKey');
}

class StartupHelper {
  static const int _hkeyCurrentUser = 0x80000001;
  static const int _keySetValue = 0x0002;
  static const int _regSz = 1;
  static const String _registryPath =
      r'Software\Microsoft\Windows\CurrentVersion\Run';
  static const String _appName = 'CopyPaste';
  static const String _macOsPlistLabel = 'com.rgdevment.copypaste';

  static Future<void> apply(bool runOnStartup) async {
    if (Platform.isWindows) {
      if (runOnStartup) {
        _setRegistryValue(Platform.resolvedExecutable);
      } else {
        _removeRegistryValue();
      }
    } else if (Platform.isMacOS) {
      if (runOnStartup) {
        _installLaunchAgent();
      } else {
        _removeLaunchAgent();
      }
    } else if (Platform.isLinux) {
      if (runOnStartup) {
        _installDesktopAutostart();
      } else {
        _removeDesktopAutostart();
      }
    }
  }

  static void _setRegistryValue(String exePath) {
    final r = _Win32Registry.instance;
    final subKey = _registryPath.toNativeUtf16();
    final hKeyPtr = calloc<IntPtr>();

    try {
      final result = r.regOpenKeyEx(
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
        final setResult = r.regSetValueEx(
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
        r.regCloseKey(hKey);
      }
    } finally {
      calloc.free(subKey);
      calloc.free(hKeyPtr);
    }
  }

  static void _removeRegistryValue() {
    final r = _Win32Registry.instance;
    final subKey = _registryPath.toNativeUtf16();
    final hKeyPtr = calloc<IntPtr>();

    try {
      final result = r.regOpenKeyEx(
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
        r.regDeleteValue(hKey, valueName);
      } finally {
        calloc.free(valueName);
        r.regCloseKey(hKey);
      }
    } finally {
      calloc.free(subKey);
      calloc.free(hKeyPtr);
    }
  }

  static String get _launchAgentPath {
    final home = Platform.environment['HOME'] ?? '/tmp';
    return '$home/Library/LaunchAgents/$_macOsPlistLabel.plist';
  }

  static void _installLaunchAgent() {
    try {
      final exePath = Platform.resolvedExecutable;
      final plist =
          '''<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$_macOsPlistLabel</string>
  <key>ProgramArguments</key>
  <array>
    <string>$exePath</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <false/>
</dict>
</plist>
''';
      final agentDir = Directory(
        '${Platform.environment['HOME']}/Library/LaunchAgents',
      );
      if (!agentDir.existsSync()) agentDir.createSync(recursive: true);
      File(_launchAgentPath).writeAsStringSync(plist);
    } catch (e) {
      AppLogger.error('Failed to install LaunchAgent: $e');
    }
  }

  static void _removeLaunchAgent() {
    try {
      final file = File(_launchAgentPath);
      if (file.existsSync()) file.deleteSync();
    } catch (e) {
      AppLogger.error('Failed to remove LaunchAgent: $e');
    }
  }

  static String get _desktopAutostartPath {
    final home = Platform.environment['HOME'] ?? '/tmp';
    return '$home/.config/autostart/$_appName.desktop';
  }

  static void _installDesktopAutostart() {
    try {
      final exePath = Platform.resolvedExecutable;
      final desktop =
          '[Desktop Entry]\n'
          'Type=Application\n'
          'Name=$_appName\n'
          'Exec=$exePath\n'
          'X-GNOME-Autostart-enabled=true\n'
          'StartupNotify=false\n'
          'Terminal=false\n';
      final autostartDir = Directory(
        '${Platform.environment['HOME']}/.config/autostart',
      );
      if (!autostartDir.existsSync()) autostartDir.createSync(recursive: true);
      File(_desktopAutostartPath).writeAsStringSync(desktop);
    } catch (e) {
      AppLogger.error('Failed to install autostart desktop entry: $e');
    }
  }

  static void _removeDesktopAutostart() {
    try {
      final file = File(_desktopAutostartPath);
      if (file.existsSync()) file.deleteSync();
    } catch (e) {
      AppLogger.error('Failed to remove autostart desktop entry: $e');
    }
  }
}
