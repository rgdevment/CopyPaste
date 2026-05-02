// coverage:ignore-file
import 'dart:ffi' hide Size;
import 'dart:io';
import 'dart:ui' show Color, Offset, Size;

import 'package:core/core.dart';
import 'package:ffi/ffi.dart';
import 'package:flutter_acrylic/flutter_acrylic.dart';
import 'package:listener/listener.dart';
import 'package:window_manager/window_manager.dart';

import 'linux_shell.dart';

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

typedef _SetWindowPosNative =
    Int32 Function(
      IntPtr hWnd,
      IntPtr hWndInsertAfter,
      Int32 x,
      Int32 y,
      Int32 cx,
      Int32 cy,
      Uint32 uFlags,
    );
typedef _SetWindowPosDart =
    int Function(
      int hWnd,
      int hWndInsertAfter,
      int x,
      int y,
      int cx,
      int cy,
      int uFlags,
    );

typedef _FindWindowWNative =
    IntPtr Function(Pointer<Utf16> lpClassName, Pointer<Utf16> lpWindowName);
typedef _FindWindowWDart =
    int Function(Pointer<Utf16> lpClassName, Pointer<Utf16> lpWindowName);

typedef _GetWindowRectNative =
    Int32 Function(IntPtr hWnd, Pointer<Int32> lpRect);
typedef _GetWindowRectDart = int Function(int hWnd, Pointer<Int32> lpRect);

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
  late final setWindowPosFunc = _u32
      .lookupFunction<_SetWindowPosNative, _SetWindowPosDart>('SetWindowPos');
  late final findWindowFunc = _u32
      .lookupFunction<_FindWindowWNative, _FindWindowWDart>('FindWindowW');
  late final getWindowRectFunc = _u32
      .lookupFunction<_GetWindowRectNative, _GetWindowRectDart>(
        'GetWindowRect',
      );
}

class AppWindow {
  AppWindow({
    this.onVisibilityChanged,
    this.showInTaskbar = true,
    double popupWidth = 360,
    double popupHeight = 500,
    this.rememberPositionEnabled,
    this.savedPositionProvider,
    this.onPositionPersist,
  }) : _popupWidth = popupWidth,
       _popupHeight = popupHeight;

  bool showInTaskbar;

  static const double _settingsWidth = 820;
  static const double _settingsHeight = 680;

  final void Function(bool visible)? onVisibilityChanged;
  final bool Function()? rememberPositionEnabled;
  final (double, double)? Function()? savedPositionProvider;
  final void Function(double x, double y)? onPositionPersist;
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

  Future<void> init({bool startVisible = false}) async {
    AppLogger.info(
      'AppWindow.init: startVisible=$startVisible, '
      'showInTaskbar=$showInTaskbar, '
      'size=${_popupWidth}x$_popupHeight',
    );
    try {
      await windowManager
          .waitUntilReadyToShow(null, () async {
            await _configureWindow(startVisible);
          })
          .timeout(const Duration(seconds: 5));
      AppLogger.info('AppWindow.init: waitUntilReadyToShow completed');
    } catch (e) {
      AppLogger.warn(
        'AppWindow.init: waitUntilReadyToShow failed ($e), '
        'attempting direct configuration',
      );
      try {
        await _configureWindow(startVisible);
        AppLogger.info('AppWindow.init: direct configuration succeeded');
      } catch (e2) {
        AppLogger.error('Window configuration failed: $e2');
      }
    }
    _visible = startVisible;
    _ready = true;
    AppLogger.info('AppWindow.init: done, ready=$_ready, visible=$_visible');
  }

  Future<void> _configureWindow(bool startVisible) async {
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
    final inTaskbar = showInTaskbar && Platform.isWindows;
    await windowManager.setSkipTaskbar(!inTaskbar);
    if (Platform.isWindows || Platform.isMacOS) {
      await windowManager.setBackgroundColor(const Color(0x00000000));
      AppLogger.info('_configureWindow: applying initial effect');
      await applyEffect();
    }
    if (startVisible) {
      AppLogger.info('_configureWindow: centering and focusing');
      await windowManager.center();
      await windowManager.focus();
    } else if (inTaskbar) {
      AppLogger.info('_configureWindow: minimizing to taskbar');
      await windowManager.minimize();
    } else {
      AppLogger.info('_configureWindow: hiding window');
      await windowManager.hide();
    }
  }

  bool _isDark = false;

  Future<void> applyEffect({bool? dark}) async {
    if (dark != null) _isDark = dark;
    try {
      if (Platform.isWindows) {
        await Window.setEffect(
          effect: WindowEffect.mica,
          color: const Color(0x00000000),
          dark: _isDark,
        ).timeout(const Duration(seconds: 2));
      } else if (Platform.isMacOS) {
        await Window.setEffect(
          effect: WindowEffect.sidebar,
          color: const Color(0x00000000),
          dark: _isDark,
        ).timeout(const Duration(seconds: 2));
      }
    } catch (e) {
      AppLogger.warn('applyEffect: window effect unavailable (non-fatal): $e');
    }
  }

  Future<void> _positionNearCursor() async {
    if (Platform.isWindows) {
      await _positionNearCursorWindows();
    } else if (Platform.isLinux) {
      await _positionNearCursorLinux();
    } else if (Platform.isMacOS) {
      await _positionNearCursorNative();
    } else {
      await windowManager.center();
    }
  }

  Future<void> _positionNearCursorLinux() async {
    try {
      final info = await LinuxShell.getCursorMonitor();
      if (info == null) {
        await _positionNearCursorNative();
        return;
      }
      final workArea = (
        info.x,
        info.y,
        info.x + info.width,
        info.y + info.height,
      );
      await _applyPosition(info.cursorX, info.cursorY, workArea);
    } catch (e) {
      AppLogger.warn('_positionNearCursorLinux: fallback to native: $e');
      await _positionNearCursorNative();
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
    } catch (e) {
      AppLogger.warn('_positionNearCursorWindows: fallback to center: $e');
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
    } catch (e) {
      AppLogger.warn('_positionNearCursorNative: fallback to center: $e');
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

    if (Platform.isWindows) {
      final ok = _setPositionWin32(x, y);
      if (!ok) {
        AppLogger.warn(
          '_setPositionWin32 returned false, falling back to windowManager.setPosition',
        );
        await windowManager.setPosition(Offset(x, y));
      }
    } else {
      await windowManager.setPosition(Offset(x, y));
    }
  }

  static bool _setPositionWin32(double x, double y) {
    try {
      const swpNoSize = 0x0001;
      const swpNoZOrder = 0x0004;
      const swpNoActivate = 0x0010;
      final w = _Win32Pos.instance;
      final className = 'FLUTTER_RUNNER_WIN32_WINDOW'.toNativeUtf16();
      final windowName = 'CopyPaste'.toNativeUtf16();
      try {
        final hwnd = w.findWindowFunc(className, windowName);
        if (hwnd == 0) return false;
        final result = w.setWindowPosFunc(
          hwnd,
          0,
          x.toInt(),
          y.toInt(),
          0,
          0,
          swpNoSize | swpNoZOrder | swpNoActivate,
        );
        return result != 0;
      } finally {
        calloc.free(className);
        calloc.free(windowName);
      }
    } catch (e) {
      AppLogger.warn('_setPositionWin32 failed: $e');
      return false;
    }
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

  static (double, double)? _getPositionWin32() {
    try {
      final w = _Win32Pos.instance;
      final className = 'FLUTTER_RUNNER_WIN32_WINDOW'.toNativeUtf16();
      final windowName = 'CopyPaste'.toNativeUtf16();
      final rect = calloc<Int32>(4);
      try {
        final hwnd = w.findWindowFunc(className, windowName);
        if (hwnd == 0) return null;
        final result = w.getWindowRectFunc(hwnd, rect);
        if (result == 0) return null;
        return (rect[0].toDouble(), rect[1].toDouble());
      } finally {
        calloc.free(className);
        calloc.free(windowName);
        calloc.free(rect);
      }
    } catch (e) {
      AppLogger.warn('_getPositionWin32 failed: $e');
      return null;
    }
  }

  static bool isPositionInSaneRange(double x, double y) {
    if (!x.isFinite || !y.isFinite) return false;
    if (x < -10000 || x > 50000) return false;
    if (y < -10000 || y > 30000) return false;
    return true;
  }

  bool _isPositionVisible(double x, double y) {
    if (!isPositionInSaneRange(x, y)) return false;
    if (Platform.isWindows) {
      try {
        const monitorDefaultToNull = 0x00000000;
        final w = _Win32Pos.instance;
        final centerX = (x + _popupWidth / 2).toInt();
        final centerY = (y + _popupHeight / 2).toInt();
        final hMonitor = w.monitorFromPointFunc(
          centerX,
          centerY,
          monitorDefaultToNull,
        );
        return hMonitor != 0;
      } catch (e) {
        AppLogger.warn('_isPositionVisible failed: $e');
        return false;
      }
    }
    return true;
  }

  Future<bool> _tryRestoreSavedPosition() async {
    if (rememberPositionEnabled?.call() != true) return false;
    final saved = savedPositionProvider?.call();
    if (saved == null) return false;
    final (x, y) = saved;
    if (!_isPositionVisible(x, y)) return false;
    if (Platform.isWindows) {
      final ok = _setPositionWin32(x, y);
      if (!ok) {
        await windowManager.setPosition(Offset(x, y));
      }
    } else {
      await windowManager.setPosition(Offset(x, y));
      await Future<void>.delayed(const Duration(milliseconds: 50));
      try {
        final actual = await windowManager.getPosition();
        if ((actual.dx - x).abs() > 100 || (actual.dy - y).abs() > 100) {
          return false;
        }
      } catch (_) {}
    }
    return true;
  }

  Future<void> show() async {
    AppLogger.info('AppWindow.show: starting');
    if (Platform.isLinux) {
      await windowManager.setSkipTaskbar(false);
      await windowManager.show();
      final restored = await _tryRestoreSavedPosition();
      if (!restored) {
        await _positionNearCursor();
      }
      await LinuxShell.focusWindow();
    } else {
      final restored = await _tryRestoreSavedPosition();
      if (!restored) {
        await _positionNearCursor();
      }
      if (Platform.isWindows) {
        await windowManager.setSkipTaskbar(false);
      }
      await windowManager.show();
      await windowManager.focus();
      AppLogger.info('AppWindow.show: window shown and focused');
      if (Platform.isWindows) {
        await applyEffect();
      }
    }
    _visible = true;
    onVisibilityChanged?.call(true);
  }

  Future<void> _captureCurrentPosition() async {
    if (rememberPositionEnabled?.call() != true) return;
    try {
      double? x;
      double? y;
      if (Platform.isWindows) {
        final pos = _getPositionWin32();
        if (pos != null) {
          x = pos.$1;
          y = pos.$2;
        }
      } else {
        final pos = await windowManager.getPosition();
        x = pos.dx;
        y = pos.dy;
      }
      if (x != null && y != null) {
        onPositionPersist?.call(x, y);
      }
    } catch (e) {
      AppLogger.warn('hide: failed to read window position: $e');
    }
  }

  Future<void> hide() async {
    if (!_visible) return;
    _visible = false;
    await _captureCurrentPosition();
    if (showInTaskbar && Platform.isWindows) {
      await windowManager.minimize();
    } else {
      Future<bool>? unmappedFuture;
      if (Platform.isLinux) {
        unmappedFuture = LinuxShell.awaitEvent(
          'unmapped',
          timeout: const Duration(milliseconds: 300),
        );
      }
      await windowManager.hide();
      if (!Platform.isMacOS) {
        await windowManager.setSkipTaskbar(true);
      }
      if (unmappedFuture != null) {
        await unmappedFuture;
      }
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
    await _captureCurrentPosition();
    _settingsMode = true;
    await windowManager.setResizable(true);
    Future<bool>? configureFuture;
    if (Platform.isLinux) {
      configureFuture = LinuxShell.awaitEvent(
        'configureNotify',
        timeout: const Duration(milliseconds: 250),
      );
    }
    await windowManager.setMinimumSize(
      const Size(_settingsWidth, _settingsHeight),
    );
    await windowManager.setMaximumSize(const Size(1200, 900));
    await windowManager.setSize(const Size(_settingsWidth, _settingsHeight));
    if (configureFuture != null) {
      await configureFuture;
    }
    await windowManager.center();
    if (!await windowManager.isVisible()) {
      await windowManager.show();
    }
    await windowManager.focus();
    _visible = true;
  }

  Future<void> exitSettingsMode() async {
    _settingsMode = false;
    Future<bool>? configureFuture;
    if (Platform.isLinux) {
      await windowManager.setResizable(true);
      configureFuture = LinuxShell.awaitEvent(
        'configureNotify',
        timeout: const Duration(milliseconds: 250),
      );
    }
    await windowManager.setMinimumSize(Size(_popupWidth, 400));
    await windowManager.setMaximumSize(Size(_popupWidth, 900));
    await windowManager.setSize(Size(_popupWidth, _popupHeight));
    if (configureFuture != null) {
      await configureFuture;
    }
    await windowManager.setResizable(false);
    await _positionNearCursor();
  }

  static const double _gateWidth = 480;
  static const double _gateHeight = 540;

  bool _gateMode = false;
  bool get isGateMode => _gateMode;

  Future<void> enterGateMode() async {
    AppLogger.info('AppWindow.enterGateMode: starting');
    await _captureCurrentPosition();
    _gateMode = true;
    await windowManager.setResizable(false);
    await windowManager.setMinimumSize(const Size(_gateWidth, _gateHeight));
    await windowManager.setMaximumSize(const Size(_gateWidth, _gateHeight));
    await windowManager.setSize(const Size(_gateWidth, _gateHeight));
    await windowManager.setAlwaysOnTop(false);
    await windowManager.setSkipTaskbar(false);
    await windowManager.center();
    await windowManager.show();
    await windowManager.focus();
    _visible = true;
    AppLogger.info('AppWindow.enterGateMode: done');
  }

  Future<void> exitGateMode() async {
    _gateMode = false;
    await windowManager.setAlwaysOnTop(true);
    await windowManager.setSkipTaskbar(!(showInTaskbar && Platform.isWindows));
    await windowManager.setMinimumSize(Size(_popupWidth, 400));
    await windowManager.setMaximumSize(Size(_popupWidth, 900));
    await windowManager.setSize(Size(_popupWidth, _popupHeight));
    await windowManager.hide();
    _visible = false;
  }
}
