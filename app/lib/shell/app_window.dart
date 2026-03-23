// coverage:ignore-file
import 'dart:ffi' hide Size;
import 'dart:io';
import 'dart:ui' show Color, Offset, Size;

import 'package:ffi/ffi.dart';
import 'package:flutter_acrylic/flutter_acrylic.dart';
import 'package:listener/listener.dart';
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
    double popupHeight = 500,
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
  bool _waylandMode = false;

  void setWaylandMode(bool enabled) => _waylandMode = enabled;

  bool get isVisible => _visible;
  bool get isReady => _ready;
  bool get isSettingsMode => _settingsMode;

  void updatePopupSize(double width, double height) {
    _popupWidth = width;
    _popupHeight = height;
  }

  Future<void> init({bool startVisible = false}) async {
    await windowManager.waitUntilReadyToShow(null, () async {
      await windowManager.setTitle('CopyPaste');
      await windowManager.setSize(Size(_popupWidth, _popupHeight));
      await windowManager.setMinimumSize(Size(_popupWidth, 400));
      await windowManager.setMaximumSize(Size(_popupWidth, 900));
      await windowManager.setTitleBarStyle(
        TitleBarStyle.hidden,
        windowButtonVisibility: !Platform.isMacOS,
      );
      await windowManager.setAlwaysOnTop(true);
      await windowManager.setResizable(false);
      await windowManager.setMaximizable(false);
      await windowManager.setPreventClose(true);
      if (!_waylandMode) {
        await windowManager.setSkipTaskbar(true);
      }
      if (Platform.isWindows || Platform.isMacOS) {
        await windowManager.setBackgroundColor(const Color(0x00000000));
        await applyEffect();
      }
      if (startVisible) {
        await windowManager.center();
        await windowManager.focus();
      } else {
        await windowManager.hide();
      }
    });
    _visible = startVisible;
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
    } else if (Platform.isMacOS || Platform.isLinux) {
      await _positionNearCursorNative();
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

  Future<void> _positionNearCursorNative() async {
    try {
      final info = await ClipboardWriter.getCursorAndScreenInfo();
      if (info == null) {
        await windowManager.center();
        return;
      }
      final cursorX = info['cursorX'] ?? 0;
      final cursorY = info['cursorY'] ?? 0;
      final workArea = (
        info['waLeft'] ?? 0,
        info['waTop'] ?? 0,
        info['waRight'] ?? 1440,
        info['waBottom'] ?? 900,
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

    if (cursorX + _popupWidth + 12 <= waRight) {
      x = cursorX + 12;
    } else if (cursorX - _popupWidth - 12 >= waLeft) {
      x = cursorX - _popupWidth - 12;
    } else {
      x = waRight - _popupWidth - 12;
    }

    y = cursorY - _popupHeight / 2;
    if (y < waTop + 8) y = waTop + 8;
    if (y + _popupHeight > waBottom - 8) y = waBottom - _popupHeight - 8;

    x = x.clamp(waLeft, waRight - _popupWidth);
    y = y.clamp(waTop, waBottom - _popupHeight);

    await windowManager.setPosition(Offset(x, y));
  }

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
    if (Platform.isWindows) {
      await applyEffect();
      await windowManager.setSkipTaskbar(false);
    } else if (Platform.isLinux) {
      await windowManager.setSkipTaskbar(false);
    }
    await windowManager.show();
    await windowManager.focus();
    _visible = true;
    onVisibilityChanged?.call(true);
  }

  Future<void> hide() async {
    if (!_visible) return;
    _visible = false;
    await windowManager.hide();
    if (!Platform.isMacOS && !_waylandMode) {
      await windowManager.setSkipTaskbar(true);
    }
    onVisibilityChanged?.call(false);
  }

  Future<void> toggle() async {
    if (_visible) {
      await hide();
    } else {
      await show();
    }
  }

  Future<void> hideIfNotPinned() async {
    if (_visible && !_settingsMode) {
      await hide();
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

  static const double _gateWidth = 480;
  static const double _gateHeight = 480;

  bool _gateMode = false;
  bool get isGateMode => _gateMode;

  Future<void> enterGateMode() async {
    _gateMode = true;
    await windowManager.setResizable(false);
    await windowManager.setMinimumSize(const Size(_gateWidth, _gateHeight));
    await windowManager.setMaximumSize(const Size(_gateWidth, _gateHeight));
    await windowManager.setSize(const Size(_gateWidth, _gateHeight));
    await windowManager.setAlwaysOnTop(false);
    await windowManager.center();
    await windowManager.show();
    await windowManager.focus();
    _visible = true;
  }

  Future<void> exitGateMode() async {
    _gateMode = false;
    await windowManager.setAlwaysOnTop(true);
    await windowManager.setMinimumSize(Size(_popupWidth, 400));
    await windowManager.setMaximumSize(Size(_popupWidth, 900));
    await windowManager.setSize(Size(_popupWidth, _popupHeight));
    await windowManager.hide();
    _visible = false;
  }
}
