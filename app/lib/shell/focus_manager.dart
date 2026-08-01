// coverage:ignore-file
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:core/core.dart';
import 'package:listener/listener.dart';

import 'windows_hotkey_channel.dart';

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

class _Win32 {
  _Win32._() {
    assert(Platform.isWindows, '_Win32 requires Windows');
  }
  static _Win32? _instance;
  static _Win32 get instance => _instance ??= _Win32._();

  static const int swRestore = 9;
  static const int gwlStyle = -16;
  static const int wsMinimize = 0x20000000;

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
}

class WindowFocusManager {
  int _previousWindow = 0;
  int _previousThreadId = 0;
  String? _previousBundleId;

  bool get hasDestination =>
      Platform.isWindows ? _previousWindow != 0 : _previousBundleId != null;

  Future<bool> capturePreviousWindow() async {
    if (Platform.isWindows) {
      return _capturePreviousWindows();
    } else if (Platform.isMacOS || Platform.isLinux) {
      _previousBundleId = await ClipboardWriter.captureFrontmostApp();
      AppLogger.info(
        'Focus session capture: platform=${Platform.operatingSystem}, '
        'destination=${_previousBundleId ?? '-'}, '
        'success=${_previousBundleId != null}',
      );
      return _previousBundleId != null;
    }
    return false;
  }

  Future<PasteResponse> restoreAndPaste({
    required int delayBeforeFocusMs,
    required int maxFocusVerifyAttempts,
    required int delayBeforePasteMs,
  }) async {
    if (Platform.isWindows && _previousWindow == 0) {
      AppLogger.warn('Paste cancelled: no previous Windows destination');
      return const PasteResponse(success: false, errorCode: 'noPreviousWindow');
    }
    if ((Platform.isMacOS || Platform.isLinux) && _previousBundleId == null) {
      AppLogger.warn('Paste cancelled: no previous application destination');
      return const PasteResponse(success: false, errorCode: 'noPreviousWindow');
    }

    try {
      await Future<void>.delayed(Duration(milliseconds: delayBeforeFocusMs));

      if (Platform.isMacOS || Platform.isLinux) {
        final response = await ClipboardWriter.activateAndPaste(
          bundleId: _previousBundleId!,
          delayMs: delayBeforePasteMs,
        );
        AppLogger.info(
          'Paste destination result: platform=${Platform.operatingSystem}, '
          'success=${response.success}, error=${response.errorCode ?? '-'}',
        );
        return response;
      }

      if (!_restorePreviousWindows()) {
        AppLogger.warn(
          'Paste cancelled: Windows rejected destination restore '
          '(hwnd=$_previousWindow)',
        );
        return const PasteResponse(success: false, errorCode: 'restoreFailed');
      }

      final focused = await _waitForFocusWindows(maxFocusVerifyAttempts);
      if (!focused) {
        AppLogger.warn(
          'Paste cancelled: Windows destination focus verification timed out '
          '(hwnd=$_previousWindow)',
        );
        return const PasteResponse(success: false, errorCode: 'focusTimeout');
      }

      await Future<void>.delayed(Duration(milliseconds: delayBeforePasteMs));
      final inputResponse = await _simulatePasteWindows();
      if (!inputResponse.success) return inputResponse;
      AppLogger.info(
        'Paste destination result: platform=windows, success=true',
      );
      return const PasteResponse(success: true);
    } finally {
      clear();
    }
  }

  void clear() {
    _previousWindow = 0;
    _previousThreadId = 0;
    _previousBundleId = null;
  }

  bool _capturePreviousWindows() {
    final w = _Win32.instance;
    final hwnd = w.getForegroundWindow();
    if (hwnd != 0 && w.isWindow(hwnd) != 0 && w.isWindowVisible(hwnd) != 0) {
      final pidPtr = calloc<Uint32>();
      try {
        final threadId = w.getWindowThreadProcessId(hwnd, pidPtr);
        if (pidPtr.value == pid) {
          clear();
          AppLogger.warn(
            'Focus session capture rejected CopyPaste itself (hwnd=$hwnd)',
          );
          return false;
        }
        _previousWindow = hwnd;
        _previousThreadId = threadId;
        AppLogger.info(
          'Focus session capture: platform=windows, hwnd=$hwnd, '
          'pid=${pidPtr.value}, success=true',
        );
        return true;
      } finally {
        calloc.free(pidPtr);
      }
    } else {
      clear();
      AppLogger.warn('Focus session capture failed: no foreground window');
      return false;
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
      final accepted = w.setForegroundWindow(_previousWindow) != 0;
      return accepted || w.getForegroundWindow() == _previousWindow;
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

  Future<PasteResponse> _simulatePasteWindows() async {
    try {
      final response = await WindowsHotkeyChannel.sendPaste();
      if (response.success) return const PasteResponse(success: true);
      AppLogger.error(
        'Windows SendInput failed: sent=${response.sentInputs ?? 0}/'
        '${response.expectedInputs ?? 0}, '
        'error=${response.errorCode}, win32=${response.win32Error}',
      );
      return PasteResponse(
        success: false,
        errorCode: response.errorCode ?? 'sendInputFailed',
      );
    } catch (e) {
      AppLogger.error('Windows SendInput platform call failed: $e');
      return const PasteResponse(success: false, errorCode: 'sendInputFailed');
    }
  }
}
