import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

typedef _GetForegroundWindowNative = IntPtr Function();
typedef _GetForegroundWindowDart = int Function();

typedef _IsWindowNative = Int32 Function(IntPtr hWnd);
typedef _IsWindowDart = int Function(int hWnd);

typedef _IsWindowVisibleNative = Int32 Function(IntPtr hWnd);
typedef _IsWindowVisibleDart = int Function(int hWnd);

typedef _SetForegroundWindowNative = Int32 Function(IntPtr hWnd);
typedef _SetForegroundWindowDart = int Function(int hWnd);

typedef _BringWindowToTopNative = Int32 Function(IntPtr hWnd);
typedef _BringWindowToTopDart = int Function(int hWnd);

typedef _ShowWindowNative = Int32 Function(IntPtr hWnd, Int32 nCmdShow);
typedef _ShowWindowDart = int Function(int hWnd, int nCmdShow);

typedef _GetWindowLongPtrNative = IntPtr Function(IntPtr hWnd, Int32 nIndex);
typedef _GetWindowLongPtrDart = int Function(int hWnd, int nIndex);

typedef _GetWindowThreadProcessIdNative =
    Uint32 Function(IntPtr hWnd, Pointer<Uint32> lpdwProcessId);
typedef _GetWindowThreadProcessIdDart =
    int Function(int hWnd, Pointer<Uint32> lpdwProcessId);

typedef _GetCurrentThreadIdNative = Uint32 Function();
typedef _GetCurrentThreadIdDart = int Function();

typedef _AttachThreadInputNative =
    Int32 Function(Uint32 idAttach, Uint32 idAttachTo, Int32 fAttach);
typedef _AttachThreadInputDart =
    int Function(int idAttach, int idAttachTo, int fAttach);

typedef _KeybdEventNative =
    Void Function(Uint8 bVk, Uint8 bScan, Uint32 dwFlags, IntPtr dwExtraInfo);
typedef _KeybdEventDart =
    void Function(int bVk, int bScan, int dwFlags, int dwExtraInfo);

class WindowFocusManager {
  static const int _swRestore = 9;
  static const int _gwlStyle = -16;
  static const int _wsMinimize = 0x20000000;
  static const int _keyeventfKeyup = 0x0002;
  static const int _vkControl = 0x11;
  static const int _vkV = 0x56;

  static final _user32 = DynamicLibrary.open('user32.dll');
  static final _kernel32 = DynamicLibrary.open('kernel32.dll');

  static final _getForegroundWindow = _user32
      .lookupFunction<_GetForegroundWindowNative, _GetForegroundWindowDart>(
        'GetForegroundWindow',
      );
  static final _isWindow = _user32
      .lookupFunction<_IsWindowNative, _IsWindowDart>('IsWindow');
  static final _isWindowVisible = _user32
      .lookupFunction<_IsWindowVisibleNative, _IsWindowVisibleDart>(
        'IsWindowVisible',
      );
  static final _setForegroundWindow = _user32
      .lookupFunction<_SetForegroundWindowNative, _SetForegroundWindowDart>(
        'SetForegroundWindow',
      );
  static final _bringWindowToTop = _user32
      .lookupFunction<_BringWindowToTopNative, _BringWindowToTopDart>(
        'BringWindowToTop',
      );
  static final _showWindow = _user32
      .lookupFunction<_ShowWindowNative, _ShowWindowDart>('ShowWindow');
  static final _getWindowLongPtr = _user32
      .lookupFunction<_GetWindowLongPtrNative, _GetWindowLongPtrDart>(
        'GetWindowLongPtrW',
      );
  static final _getWindowThreadProcessId = _user32
      .lookupFunction<
        _GetWindowThreadProcessIdNative,
        _GetWindowThreadProcessIdDart
      >('GetWindowThreadProcessId');
  static final _getCurrentThreadId = _kernel32
      .lookupFunction<_GetCurrentThreadIdNative, _GetCurrentThreadIdDart>(
        'GetCurrentThreadId',
      );
  static final _attachThreadInput = _user32
      .lookupFunction<_AttachThreadInputNative, _AttachThreadInputDart>(
        'AttachThreadInput',
      );
  static final _keybdEvent = _user32
      .lookupFunction<_KeybdEventNative, _KeybdEventDart>('keybd_event');

  int _previousWindow = 0;
  int _previousThreadId = 0;

  void capturePreviousWindow() {
    if (!Platform.isWindows) return;

    final hwnd = _getForegroundWindow();
    if (hwnd != 0 && _isWindow(hwnd) != 0 && _isWindowVisible(hwnd) != 0) {
      _previousWindow = hwnd;
      final pidPtr = calloc<Uint32>();
      try {
        _previousThreadId = _getWindowThreadProcessId(hwnd, pidPtr);
      } finally {
        calloc.free(pidPtr);
      }
    } else {
      _previousWindow = 0;
      _previousThreadId = 0;
    }
  }

  bool restorePreviousWindow() {
    if (!Platform.isWindows || _previousWindow == 0) return false;
    if (_isWindow(_previousWindow) == 0) {
      _previousWindow = 0;
      return false;
    }

    final currentThreadId = _getCurrentThreadId();
    var attached = false;

    if (currentThreadId != _previousThreadId && _previousThreadId != 0) {
      attached = _attachThreadInput(currentThreadId, _previousThreadId, 1) != 0;
    }

    try {
      final style = _getWindowLongPtr(_previousWindow, _gwlStyle);
      if (style & _wsMinimize != 0) {
        _showWindow(_previousWindow, _swRestore);
      }

      _bringWindowToTop(_previousWindow);
      return _setForegroundWindow(_previousWindow) != 0;
    } finally {
      if (attached) {
        _attachThreadInput(currentThreadId, _previousThreadId, 0);
      }
    }
  }

  Future<bool> _waitForFocus(int maxAttempts) async {
    for (var i = 0; i < maxAttempts; i++) {
      if (_getForegroundWindow() == _previousWindow) return true;
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    return false;
  }

  void simulatePaste() {
    if (!Platform.isWindows) return;
    _keybdEvent(_vkControl, 0, 0, 0);
    _keybdEvent(_vkV, 0, 0, 0);
    _keybdEvent(_vkV, 0, _keyeventfKeyup, 0);
    _keybdEvent(_vkControl, 0, _keyeventfKeyup, 0);
  }

  Future<void> restoreAndPaste({
    required int delayBeforeFocusMs,
    required int maxFocusVerifyAttempts,
    required int delayBeforePasteMs,
  }) async {
    if (_previousWindow == 0) return;

    await Future<void>.delayed(Duration(milliseconds: delayBeforeFocusMs));

    if (!restorePreviousWindow()) return;

    final focused = await _waitForFocus(maxFocusVerifyAttempts);
    if (!focused) {
      await Future<void>.delayed(Duration(milliseconds: delayBeforePasteMs));
    } else {
      // Brief stabilization delay for clipboard propagation
      await Future<void>.delayed(const Duration(milliseconds: 30));
    }

    simulatePaste();
  }

  void clear() {
    _previousWindow = 0;
    _previousThreadId = 0;
  }
}
