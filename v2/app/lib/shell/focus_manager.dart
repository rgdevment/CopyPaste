// coverage:ignore-file
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

// ---------- Win32 FFI typedefs (only used on Windows) ----------

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

// ---------- Lazy Win32 helpers (never loaded on macOS/Linux) ----------

class _Win32 {
  _Win32._();
  static _Win32? _instance;
  static _Win32 get instance => _instance ??= _Win32._();

  static const int swRestore = 9;
  static const int gwlStyle = -16;
  static const int wsMinimize = 0x20000000;
  static const int keyeventfKeyup = 0x0002;
  static const int vkControl = 0x11;
  static const int vkV = 0x56;

  late final _user32 = DynamicLibrary.open('user32.dll');
  late final _kernel32 = DynamicLibrary.open('kernel32.dll');

  late final getForegroundWindow = _user32
      .lookupFunction<_GetForegroundWindowNative, _GetForegroundWindowDart>(
        'GetForegroundWindow',
      );
  late final isWindow = _user32.lookupFunction<_IsWindowNative, _IsWindowDart>(
    'IsWindow',
  );
  late final isWindowVisible = _user32
      .lookupFunction<_IsWindowVisibleNative, _IsWindowVisibleDart>(
        'IsWindowVisible',
      );
  late final setForegroundWindow = _user32
      .lookupFunction<_SetForegroundWindowNative, _SetForegroundWindowDart>(
        'SetForegroundWindow',
      );
  late final bringWindowToTop = _user32
      .lookupFunction<_BringWindowToTopNative, _BringWindowToTopDart>(
        'BringWindowToTop',
      );
  late final showWindow = _user32
      .lookupFunction<_ShowWindowNative, _ShowWindowDart>('ShowWindow');
  late final getWindowLongPtr = _user32
      .lookupFunction<_GetWindowLongPtrNative, _GetWindowLongPtrDart>(
        'GetWindowLongPtrW',
      );
  late final getWindowThreadProcessId = _user32
      .lookupFunction<
        _GetWindowThreadProcessIdNative,
        _GetWindowThreadProcessIdDart
      >('GetWindowThreadProcessId');
  late final getCurrentThreadId = _kernel32
      .lookupFunction<_GetCurrentThreadIdNative, _GetCurrentThreadIdDart>(
        'GetCurrentThreadId',
      );
  late final attachThreadInput = _user32
      .lookupFunction<_AttachThreadInputNative, _AttachThreadInputDart>(
        'AttachThreadInput',
      );
  late final keybdEvent = _user32
      .lookupFunction<_KeybdEventNative, _KeybdEventDart>('keybd_event');
}

// ---------- WindowFocusManager ----------

class WindowFocusManager {
  int _previousWindow = 0;
  int _previousThreadId = 0;

  // macOS: store the bundle-id of the previously focused app.
  String? _previousBundleId;

  void capturePreviousWindow() {
    if (Platform.isWindows) {
      _capturePreviousWindows();
    } else if (Platform.isMacOS) {
      _capturePreviousMacOS();
    }
  }

  bool restorePreviousWindow() {
    if (Platform.isWindows) {
      return _restorePreviousWindows();
    } else if (Platform.isMacOS) {
      return _restorePreviousMacOS();
    }
    return false;
  }

  void simulatePaste() {
    if (Platform.isWindows) {
      _simulatePasteWindows();
    } else if (Platform.isMacOS) {
      _simulatePasteMacOS();
    }
  }

  Future<void> restoreAndPaste({
    required int delayBeforeFocusMs,
    required int maxFocusVerifyAttempts,
    required int delayBeforePasteMs,
  }) async {
    if (Platform.isWindows && _previousWindow == 0) return;
    if (Platform.isMacOS && _previousBundleId == null) return;

    await Future<void>.delayed(Duration(milliseconds: delayBeforeFocusMs));

    if (Platform.isMacOS) {
      await _restoreAndPasteMacOS(delayBeforePasteMs);
      return;
    }

    if (!restorePreviousWindow()) return;

    final focused = await _waitForFocusWindows(maxFocusVerifyAttempts);
    if (!focused) {
      await Future<void>.delayed(Duration(milliseconds: delayBeforePasteMs));
    } else {
      await Future<void>.delayed(const Duration(milliseconds: 30));
    }

    simulatePaste();
  }

  void clear() {
    _previousWindow = 0;
    _previousThreadId = 0;
    _previousBundleId = null;
  }

  // ---------- Windows implementation ----------

  void _capturePreviousWindows() {
    final w = _Win32.instance;
    final hwnd = w.getForegroundWindow();
    if (hwnd != 0 && w.isWindow(hwnd) != 0 && w.isWindowVisible(hwnd) != 0) {
      _previousWindow = hwnd;
      final pidPtr = calloc<Uint32>();
      try {
        _previousThreadId = w.getWindowThreadProcessId(hwnd, pidPtr);
      } finally {
        calloc.free(pidPtr);
      }
    } else {
      _previousWindow = 0;
      _previousThreadId = 0;
    }
  }

  bool _restorePreviousWindows() {
    if (_previousWindow == 0) return false;
    final w = _Win32.instance;
    if (w.isWindow(_previousWindow) == 0) {
      _previousWindow = 0;
      return false;
    }

    final currentThreadId = w.getCurrentThreadId();
    var attached = false;

    if (currentThreadId != _previousThreadId && _previousThreadId != 0) {
      attached =
          w.attachThreadInput(currentThreadId, _previousThreadId, 1) != 0;
    }

    try {
      final style = w.getWindowLongPtr(_previousWindow, _Win32.gwlStyle);
      if (style & _Win32.wsMinimize != 0) {
        w.showWindow(_previousWindow, _Win32.swRestore);
      }

      w.bringWindowToTop(_previousWindow);
      return w.setForegroundWindow(_previousWindow) != 0;
    } finally {
      if (attached) {
        w.attachThreadInput(currentThreadId, _previousThreadId, 0);
      }
    }
  }

  Future<bool> _waitForFocusWindows(int maxAttempts) async {
    final w = _Win32.instance;
    for (var i = 0; i < maxAttempts; i++) {
      if (w.getForegroundWindow() == _previousWindow) return true;
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    return false;
  }

  void _simulatePasteWindows() {
    final w = _Win32.instance;
    w.keybdEvent(_Win32.vkControl, 0, 0, 0);
    w.keybdEvent(_Win32.vkV, 0, 0, 0);
    w.keybdEvent(_Win32.vkV, 0, _Win32.keyeventfKeyup, 0);
    w.keybdEvent(_Win32.vkControl, 0, _Win32.keyeventfKeyup, 0);
  }

  // ---------- macOS implementation ----------

  void _capturePreviousMacOS() {
    try {
      final result = Process.runSync('osascript', [
        '-e',
        'tell application "System Events" to get bundle identifier '
            'of first application process whose frontmost is true',
      ]);
      final bid = (result.stdout as String).trim();
      _previousBundleId = bid.isNotEmpty ? bid : null;
    } catch (_) {
      _previousBundleId = null;
    }
  }

  bool _restorePreviousMacOS() {
    if (_previousBundleId == null) return false;
    try {
      Process.runSync('osascript', [
        '-e',
        'tell application id "$_previousBundleId" to activate',
      ]);
      return true;
    } catch (_) {
      return false;
    }
  }

  Future<void> _restoreAndPasteMacOS(int delayBeforePasteMs) async {
    if (_previousBundleId == null) return;
    final delaySeconds = (delayBeforePasteMs / 1000).toStringAsFixed(2);
    try {
      await Process.run('osascript', [
        '-e',
        'tell application id "$_previousBundleId" to activate',
        '-e',
        'delay $delaySeconds',
        '-e',
        'tell application "System Events" to keystroke "v" using command down',
      ]);
    } catch (_) {
      // Accessibility permission may be missing
    }
  }

  void _simulatePasteMacOS() {
    try {
      // Use osascript to simulate Cmd+V via System Events
      Process.runSync('osascript', [
        '-e',
        'tell application "System Events" to keystroke "v" '
            'using command down',
      ]);
    } catch (_) {
      // Silently fail — Accessibility permission may be missing
    }
  }
}
