// coverage:ignore-file
import 'dart:ffi' hide Size;
import 'dart:io';
import 'dart:ui' show Color, Offset, Size;

import 'package:ffi/ffi.dart';
import 'package:flutter_acrylic/flutter_acrylic.dart';
import 'package:window_manager/window_manager.dart';

// ---------- Win32 FFI typedefs ----------

typedef _SystemParametersInfoWNative =
    Int32 Function(
      Uint32 uiAction,
      Uint32 uiParam,
      Pointer lpvParam,
      Uint32 fWinIni,
    );
typedef _SystemParametersInfoWDart =
    int Function(int uiAction, int uiParam, Pointer lpvParam, int fWinIni);

typedef _GetCursorPosNative = Int32 Function(Pointer<Int32> lpPoint);
typedef _GetCursorPosDart = int Function(Pointer<Int32> lpPoint);

typedef _MonitorFromPointNative =
    IntPtr Function(Int32 x, Int32 y, Uint32 dwFlags);
typedef _MonitorFromPointDart = int Function(int x, int y, int dwFlags);

typedef _GetMonitorInfoWNative = Int32 Function(IntPtr hMonitor, Pointer lpmi);
typedef _GetMonitorInfoWDart = int Function(int hMonitor, Pointer lpmi);

// ---------- Lazy Win32 positioning helpers ----------

class _Win32Pos {
  _Win32Pos._();
  static _Win32Pos? _instance;
  static _Win32Pos get instance => _instance ??= _Win32Pos._();

  late final _u32 = DynamicLibrary.open('user32.dll');
  late final spiFunc = _u32
      .lookupFunction<_SystemParametersInfoWNative, _SystemParametersInfoWDart>(
        'SystemParametersInfoW',
      );
  late final getCursorPosFunc = _u32
      .lookupFunction<_GetCursorPosNative, _GetCursorPosDart>('GetCursorPos');
  late final monitorFromPointFunc = _u32
      .lookupFunction<_MonitorFromPointNative, _MonitorFromPointDart>(
        'MonitorFromPoint',
      );
  late final getMonitorInfoFunc = _u32
      .lookupFunction<_GetMonitorInfoWNative, _GetMonitorInfoWDart>(
        'GetMonitorInfoW',
      );
}

class AppWindow {
  AppWindow({
    this.onVisibilityChanged,
    double popupWidth = 360,
    double popupHeight = 520,
  }) : _popupWidth = popupWidth,
       _popupHeight = popupHeight;

  static const double _settingsWidth = 820;
  static const double _settingsHeight = 680;

  final void Function(bool visible)? onVisibilityChanged;
  double _popupWidth;
  double _popupHeight;
  bool _visible = false;
  bool _ready = false;
  bool _settingsMode = false;

  bool get isVisible => _visible;
  bool get isReady => _ready;
  bool get isSettingsMode => _settingsMode;

  void updatePopupSize(double width, double height) {
    _popupWidth = width;
    _popupHeight = height;
  }

  Future<void> init() async {
    await windowManager.waitUntilReadyToShow(null, () async {
      await windowManager.setTitle('CopyPaste');
      await windowManager.setSize(Size(_popupWidth, _popupHeight));
      await windowManager.setMinimumSize(Size(_popupWidth, 400));
      await windowManager.setMaximumSize(Size(_popupWidth, 900));
      await windowManager.setTitleBarStyle(TitleBarStyle.hidden);
      await windowManager.setAlwaysOnTop(true);
      await windowManager.setResizable(false);
      await windowManager.setMaximizable(false);
      await windowManager.setPreventClose(true);
      await windowManager.setSkipTaskbar(true);
      if (Platform.isWindows) {
        await windowManager.setBackgroundColor(const Color(0x00000000));
        await applyEffect();
      } else if (Platform.isMacOS) {
        await windowManager.setBackgroundColor(const Color(0x00000000));
        await applyEffect();
      }
      await windowManager.hide();
    });
    _visible = false;
    _ready = true;
  }

  bool _isDark = false;

  Future<void> applyEffect({bool? dark}) async {
    if (dark != null) _isDark = dark;
    if (Platform.isWindows) {
      await Window.setEffect(
        effect: WindowEffect.mica,
        color: const Color(0x00000000),
        dark: _isDark,
      );
    } else if (Platform.isMacOS) {
      await Window.setEffect(
        effect: WindowEffect.sidebar,
        color: const Color(0x00000000),
        dark: _isDark,
      );
    }
  }

  Future<void> _positionNearCursor() async {
    if (Platform.isWindows) {
      await _positionNearCursorWindows();
    } else if (Platform.isMacOS) {
      await _positionNearCursorMacOS();
    } else {
      await windowManager.center();
    }
  }

  Future<void> _positionNearCursorWindows() async {
    try {
      final cursor = _getCursorPosWin32();
      if (cursor == null) {
        await windowManager.center();
        return;
      }
      final workArea = _getWorkAreaForPointWin32(cursor.$1, cursor.$2);
      if (workArea == null) {
        await windowManager.center();
        return;
      }
      await _applyPosition(cursor.$1, cursor.$2, workArea);
    } catch (_) {
      await windowManager.center();
    }
  }

  Future<void> _positionNearCursorMacOS() async {
    try {
      // Get mouse location via AppleScript (returns {x, y} in screen coords)
      final result = Process.runSync('osascript', [
        '-e',
        'use framework "AppKit"\n'
            'set mousePos to current application\'s NSEvent\'s mouseLocation()\n'
            'set screenH to (current application\'s NSScreen\'s mainScreen()\'s frame()\'s |size|\'s height) as integer\n'
            'set mx to (mousePos\'s x) as integer\n'
            'set my to (screenH - (mousePos\'s y as integer))\n'
            'return (mx as text) & "," & (my as text)',
      ]);
      final parts = (result.stdout as String).trim().split(',');
      if (parts.length != 2) {
        await windowManager.center();
        return;
      }
      final cursorX = double.tryParse(parts[0]) ?? 0;
      final cursorY = double.tryParse(parts[1]) ?? 0;

      // Get visible frame of the screen containing the cursor
      final screenResult = Process.runSync('osascript', [
        '-e',
        'use framework "AppKit"\n'
            'set screens to current application\'s NSScreen\'s screens()\n'
            'set mousePos to current application\'s NSEvent\'s mouseLocation()\n'
            'set mainH to (current application\'s NSScreen\'s mainScreen()\'s frame()\'s |size|\'s height) as integer\n'
            'repeat with s in screens\n'
            '  set f to s\'s frame()\n'
            '  set fx to f\'s origin\'s x\n'
            '  set fy to f\'s origin\'s y\n'
            '  set fw to f\'s |size|\'s width\n'
            '  set fh to f\'s |size|\'s height\n'
            '  if (mousePos\'s x) >= fx and (mousePos\'s x) < (fx + fw) and (mousePos\'s y) >= fy and (mousePos\'s y) < (fy + fh) then\n'
            '    set vf to s\'s visibleFrame()\n'
            '    set vx to (vf\'s origin\'s x) as integer\n'
            '    set vy to (mainH - ((vf\'s origin\'s y) as integer) - ((vf\'s |size|\'s height) as integer))\n'
            '    set vw to (vf\'s |size|\'s width) as integer\n'
            '    set vh to (vf\'s |size|\'s height) as integer\n'
            '    return (vx as text) & "," & (vy as text) & "," & ((vx + vw) as text) & "," & ((vy + vh) as text)\n'
            '  end if\n'
            'end repeat\n'
            'return ""',
      ]);
      final waParts = (screenResult.stdout as String).trim().split(',');
      if (waParts.length != 4) {
        await windowManager.center();
        return;
      }
      final workArea = (
        double.tryParse(waParts[0]) ?? 0,
        double.tryParse(waParts[1]) ?? 0,
        double.tryParse(waParts[2]) ?? 1440,
        double.tryParse(waParts[3]) ?? 900,
      );
      await _applyPosition(cursorX, cursorY, workArea);
    } catch (_) {
      await windowManager.center();
    }
  }

  Future<void> _applyPosition(
    double cursorX,
    double cursorY,
    (double, double, double, double) workArea,
  ) async {
    final waLeft = workArea.$1;
    final waTop = workArea.$2;
    final waRight = workArea.$3;
    final waBottom = workArea.$4;

    double x;
    double y;

    // Horizontal: try right of cursor, else left
    if (cursorX + _popupWidth + 12 <= waRight) {
      x = cursorX + 12;
    } else if (cursorX - _popupWidth - 12 >= waLeft) {
      x = cursorX - _popupWidth - 12;
    } else {
      x = waRight - _popupWidth - 12;
    }

    // Vertical: center on cursor, clamp to work area
    y = cursorY - _popupHeight / 2;
    if (y < waTop + 8) y = waTop + 8;
    if (y + _popupHeight > waBottom - 8) y = waBottom - _popupHeight - 8;

    // Final clamp
    x = x.clamp(waLeft, waRight - _popupWidth);
    y = y.clamp(waTop, waBottom - _popupHeight);

    await windowManager.setPosition(Offset(x, y));
  }

  // ---------- Win32 cursor/monitor helpers ----------

  static (double, double)? _getCursorPosWin32() {
    final w = _Win32Pos.instance;
    final pt = calloc<Int32>(2);
    try {
      final result = w.getCursorPosFunc(pt);
      if (result == 0) return null;
      return (pt[0].toDouble(), pt[1].toDouble());
    } finally {
      calloc.free(pt);
    }
  }

  static (double, double, double, double)? _getWorkAreaForPointWin32(
    double x,
    double y,
  ) {
    const monitorDefaultToNearest = 0x00000002;
    final w = _Win32Pos.instance;
    final hMonitor = w.monitorFromPointFunc(
      x.toInt(),
      y.toInt(),
      monitorDefaultToNearest,
    );
    if (hMonitor == 0) return _getWorkAreaWin32();

    final mi = calloc<Int32>(10);
    try {
      mi[0] = 40;
      final result = w.getMonitorInfoFunc(hMonitor, mi);
      if (result == 0) return _getWorkAreaWin32();
      return (
        mi[5].toDouble(),
        mi[6].toDouble(),
        mi[7].toDouble(),
        mi[8].toDouble(),
      );
    } finally {
      calloc.free(mi);
    }
  }

  static (double, double, double, double)? _getWorkAreaWin32() {
    const spiGetWorkArea = 0x0030;
    final w = _Win32Pos.instance;
    final rect = calloc<Int32>(4);
    try {
      final result = w.spiFunc(spiGetWorkArea, 0, rect, 0);
      if (result == 0) return null;
      return (
        rect[0].toDouble(),
        rect[1].toDouble(),
        rect[2].toDouble(),
        rect[3].toDouble(),
      );
    } finally {
      calloc.free(rect);
    }
  }

  Future<void> show() async {
    await _positionNearCursor();
    if (Platform.isWindows || Platform.isMacOS) {
      await applyEffect();
    }
    await windowManager.setSkipTaskbar(false);
    await windowManager.show();
    await windowManager.focus();
    _visible = true;
    onVisibilityChanged?.call(true);
  }

  Future<void> hide() async {
    await windowManager.hide();
    await windowManager.setSkipTaskbar(true);
    _visible = false;
    onVisibilityChanged?.call(false);
  }

  Future<void> toggle() async {
    if (_visible) {
      await hide();
    } else {
      await show();
    }
  }

  void hideIfNotPinned() {
    if (_visible && !_settingsMode) {
      hide();
    }
  }

  Future<void> enterSettingsMode() async {
    _settingsMode = true;
    await windowManager.setResizable(true);
    await windowManager.setMinimumSize(
      const Size(_settingsWidth, _settingsHeight),
    );
    await windowManager.setMaximumSize(const Size(1200, 900));
    await windowManager.setSize(const Size(_settingsWidth, _settingsHeight));
    await windowManager.center();
  }

  Future<void> exitSettingsMode() async {
    _settingsMode = false;
    await windowManager.setMinimumSize(Size(_popupWidth, 400));
    await windowManager.setMaximumSize(Size(_popupWidth, 900));
    await windowManager.setSize(Size(_popupWidth, _popupHeight));
    await windowManager.setResizable(false);
    await _positionNearCursor();
  }
}
