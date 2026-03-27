// coverage:ignore-file
import 'dart:async';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

// Shell_NotifyIconW message codes
const _nimAdd = 0;
const _nimDelete = 2;

// NIF flags
const _nifIcon = 0x02;
const _nifInfo = 0x10;

// NIIF flags  (balloon icon + silence)
const _niifInfo = 0x01;
const _niifNosound = 0x10;

// System icon resource id (IDI_APPLICATION)
const _idiApplication = 32512;

// NOTIFYICONDATAW struct size (Vista+ without GUID/hBalloonIcon)
const _nidSize = 952;

// NOTIFYICONDATAW field offsets on x64
const _offCbsize = 0; // DWORD  (+0)
const _offHwnd = 8; // HWND   (+8, pointer-aligned)
const _offUid = 16; // UINT   (+16)
const _offUflags = 20; // UINT   (+20)
const _offHicon = 32; // HICON  (+32, pointer-aligned)
const _offSzinfo = 304; // WCHAR[256] (+304)
const _offSzinfotitle = 820; // WCHAR[64] (+820)
const _offDwinfoflags = 948; // DWORD (+948)

typedef _ShellNotifyNative =
    Int32 Function(Uint32 dwMessage, Pointer<Uint8> lpData);
typedef _ShellNotifyDart = int Function(int dwMessage, Pointer<Uint8> lpData);

typedef _FindWindowNative =
    IntPtr Function(Pointer<Utf16> lpClassName, Pointer<Utf16> lpWindowName);
typedef _FindWindowDart =
    int Function(Pointer<Utf16> lpClassName, Pointer<Utf16> lpWindowName);

typedef _LoadIconNative = IntPtr Function(IntPtr hInstance, IntPtr lpIconName);
typedef _LoadIconDart = int Function(int hInstance, int lpIconName);

/// Shows a Windows balloon notification near the system tray.
///
/// Design rules:
/// - Static: no sound, standard Windows fade animation only.
/// - Non-intrusive: info icon, auto-dismisses, never blocks input.
/// - Informative: shows app name + current hotkey so user knows how to open it.
///
/// Safe to call on any platform — no-op on non-Windows.
/// Use [unawaited] at the call site; this method awaits the cleanup timer.
class WindowsBalloon {
  WindowsBalloon._();

  static const _balloonUid = 0x4350; // 'CP' — avoids conflict with tray_manager
  static const _cleanupDelayMs =
      7000; // balloon auto-dismisses around 5-15s on Win11

  static _ShellNotifyDart? _shellNotify;
  static _FindWindowDart? _findWindow;
  static _LoadIconDart? _loadIcon;

  static void _ensureLoaded() {
    if (_shellNotify != null) return;
    final shell32 = DynamicLibrary.open('shell32.dll');
    final user32 = DynamicLibrary.open('user32.dll');
    _shellNotify = shell32.lookupFunction<_ShellNotifyNative, _ShellNotifyDart>(
      'Shell_NotifyIconW',
    );
    _findWindow = user32.lookupFunction<_FindWindowNative, _FindWindowDart>(
      'FindWindowW',
    );
    _loadIcon = user32.lookupFunction<_LoadIconNative, _LoadIconDart>(
      'LoadIconW',
    );
  }

  static void _writeUint32(Pointer<Uint8> p, int offset, int value) =>
      p.elementAt(offset).cast<Uint32>().value = value;

  static void _writeUint64(Pointer<Uint8> p, int offset, int value) =>
      p.elementAt(offset).cast<Uint64>().value = value;

  static void _writeWString(
    Pointer<Uint8> p,
    int offset,
    String text,
    int maxChars,
  ) {
    final units = text.codeUnits;
    final len = units.length < maxChars - 1 ? units.length : maxChars - 1;
    for (var i = 0; i < len; i++) {
      p.elementAt(offset + i * 2).cast<Uint16>().value = units[i];
    }
    // null-terminate
    p.elementAt(offset + len * 2).cast<Uint16>().value = 0;
  }

  /// Shows a balloon notification with [title] and [body].
  ///
  /// The body should include the current hotkey so users know how to open
  /// the app without needing to find the tray icon.
  static Future<void> show({
    required String title,
    required String body,
  }) async {
    if (!Platform.isWindows) return;
    try {
      _ensureLoaded();

      final winTitle = 'CopyPaste'.toNativeUtf16();
      final hwnd = _findWindow!(nullptr, winTitle);
      calloc.free(winTitle);
      if (hwnd == 0) return;

      // Load the default system app icon (IDI_APPLICATION).
      // hInstance = 0 → load from system resources.
      final hIcon = _loadIcon!(0, _idiApplication);

      // calloc zero-initialises all bytes.
      final nid = calloc<Uint8>(_nidSize);
      try {
        _writeUint32(nid, _offCbsize, _nidSize);
        _writeUint64(nid, _offHwnd, hwnd);
        _writeUint32(nid, _offUid, _balloonUid);
        _writeUint32(nid, _offUflags, _nifInfo | (hIcon != 0 ? _nifIcon : 0));
        if (hIcon != 0) _writeUint64(nid, _offHicon, hIcon);
        _writeWString(nid, _offSzinfo, body, 256);
        _writeWString(nid, _offSzinfotitle, title, 64);
        _writeUint32(nid, _offDwinfoflags, _niifInfo | _niifNosound);

        _shellNotify!(_nimAdd, nid);
        await Future<void>.delayed(
          const Duration(milliseconds: _cleanupDelayMs),
        );
      } finally {
        _shellNotify!(_nimDelete, nid);
        calloc.free(nid);
      }
    } catch (_) {
      // Balloon is best-effort — a failure must never affect app startup.
    }
  }
}
