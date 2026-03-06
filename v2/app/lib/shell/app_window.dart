import 'dart:ffi' hide Size;
import 'dart:io';
import 'dart:ui' show Color, Offset, Size;

import 'package:ffi/ffi.dart';
import 'package:flutter_acrylic/flutter_acrylic.dart';
import 'package:window_manager/window_manager.dart';

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
        await applyMica();
      }
      await windowManager.hide();
    });
    _visible = false;
    _ready = true;
  }

  bool _isDark = false;

  Future<void> applyMica({bool? dark}) async {
    if (dark != null) _isDark = dark;
    await Window.setEffect(
      effect: WindowEffect.mica,
      color: const Color(0x00000000),
      dark: _isDark,
    );
  }

  Future<void> _positionNearCursor() async {
    if (!Platform.isWindows) {
      await windowManager.center();
      return;
    }
    try {
      final cursor = _getCursorPos();
      if (cursor == null) {
        await windowManager.center();
        return;
      }
      final workArea = _getWorkAreaForPoint(cursor.$1, cursor.$2);
      if (workArea == null) {
        await windowManager.center();
        return;
      }
      final waLeft = workArea.$1;
      final waTop = workArea.$2;
      final waRight = workArea.$3;
      final waBottom = workArea.$4;

      double x;
      double y;
      final cursorX = cursor.$1;
      final cursorY = cursor.$2;

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
    } catch (_) {
      await windowManager.center();
    }
  }

  static final _u32 = DynamicLibrary.open('user32.dll');
  static final _spiFunc = _u32
      .lookupFunction<_SystemParametersInfoWNative, _SystemParametersInfoWDart>(
        'SystemParametersInfoW',
      );
  static final _getCursorPosFunc = _u32
      .lookupFunction<_GetCursorPosNative, _GetCursorPosDart>('GetCursorPos');
  static final _monitorFromPointFunc = _u32
      .lookupFunction<_MonitorFromPointNative, _MonitorFromPointDart>(
        'MonitorFromPoint',
      );
  static final _getMonitorInfoFunc = _u32
      .lookupFunction<_GetMonitorInfoWNative, _GetMonitorInfoWDart>(
        'GetMonitorInfoW',
      );

  static (double, double)? _getCursorPos() {
    final pt = calloc<Int32>(2);
    try {
      final result = _getCursorPosFunc(pt);
      if (result == 0) return null;
      return (pt[0].toDouble(), pt[1].toDouble());
    } finally {
      calloc.free(pt);
    }
  }

  static (double, double, double, double)? _getWorkAreaForPoint(
    double x,
    double y,
  ) {
    const monitorDefaultToNearest = 0x00000002;
    final hMonitor = _monitorFromPointFunc(
      x.toInt(),
      y.toInt(),
      monitorDefaultToNearest,
    );
    if (hMonitor == 0) return _getWorkArea();

    // MONITORINFO: cbSize(4) + rcMonitor(16) + rcWork(16) + dwFlags(4) = 40 bytes
    final mi = calloc<Int32>(10);
    try {
      mi[0] = 40; // cbSize
      final result = _getMonitorInfoFunc(hMonitor, mi);
      if (result == 0) return _getWorkArea();
      // rcWork starts at offset 20 bytes = index 5
      return (
        mi[5].toDouble(), // left
        mi[6].toDouble(), // top
        mi[7].toDouble(), // right
        mi[8].toDouble(), // bottom
      );
    } finally {
      calloc.free(mi);
    }
  }

  static (double, double, double, double)? _getWorkArea() {
    const spiGetWorkArea = 0x0030;
    final rect = calloc<Int32>(4);
    try {
      final result = _spiFunc(spiGetWorkArea, 0, rect, 0);
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
    if (Platform.isWindows) {
      await applyMica();
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
