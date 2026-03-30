// coverage:ignore-file
import 'dart:async';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:ffi/ffi.dart';

typedef _CreateMutexWNative =
    IntPtr Function(
      Pointer<Void> lpMutexAttributes,
      Int32 bInitialOwner,
      Pointer<Utf16> lpName,
    );
typedef _CreateMutexWDart =
    int Function(
      Pointer<Void> lpMutexAttributes,
      int bInitialOwner,
      Pointer<Utf16> lpName,
    );

typedef _GetLastErrorNative = Uint32 Function();
typedef _GetLastErrorDart = int Function();

typedef _CloseHandleNative = Int32 Function(IntPtr hObject);
typedef _CloseHandleDart = int Function(int hObject);

typedef _ReleaseMutexNative = Int32 Function(IntPtr hMutex);
typedef _ReleaseMutexDart = int Function(int hMutex);

typedef _AllowSetForegroundWindowNative = Int32 Function(Uint32 dwProcessId);
typedef _AllowSetForegroundWindowDart = int Function(int dwProcessId);

typedef _WaitForSingleObjectNative = Uint32 Function(
  IntPtr hHandle,
  Uint32 dwMilliseconds,
);
typedef _WaitForSingleObjectDart = int Function(int hHandle, int dwMilliseconds);

// Named pipe FFI types
typedef _CreateNamedPipeWNative =
    IntPtr Function(
      Pointer<Utf16> lpName,
      Uint32 dwOpenMode,
      Uint32 dwPipeMode,
      Uint32 nMaxInstances,
      Uint32 nOutBufferSize,
      Uint32 nInBufferSize,
      Uint32 nDefaultTimeOut,
      Pointer<Void> lpSecurityAttributes,
    );
typedef _CreateNamedPipeWDart =
    int Function(
      Pointer<Utf16> lpName,
      int dwOpenMode,
      int dwPipeMode,
      int nMaxInstances,
      int nOutBufferSize,
      int nInBufferSize,
      int nDefaultTimeOut,
      Pointer<Void> lpSecurityAttributes,
    );

typedef _ConnectNamedPipeNative =
    Int32 Function(IntPtr hNamedPipe, Pointer<Void> lpOverlapped);
typedef _ConnectNamedPipeDart =
    int Function(int hNamedPipe, Pointer<Void> lpOverlapped);

typedef _DisconnectNamedPipeNative = Int32 Function(IntPtr hNamedPipe);
typedef _DisconnectNamedPipeDart = int Function(int hNamedPipe);

typedef _CreateFileWNative =
    IntPtr Function(
      Pointer<Utf16> lpFileName,
      Uint32 dwDesiredAccess,
      Uint32 dwShareMode,
      Pointer<Void> lpSecurityAttributes,
      Uint32 dwCreationDisposition,
      Uint32 dwFlagsAndAttributes,
      IntPtr hTemplateFile,
    );
typedef _CreateFileWDart =
    int Function(
      Pointer<Utf16> lpFileName,
      int dwDesiredAccess,
      int dwShareMode,
      Pointer<Void> lpSecurityAttributes,
      int dwCreationDisposition,
      int dwFlagsAndAttributes,
      int hTemplateFile,
    );

typedef _WriteFileNative =
    Int32 Function(
      IntPtr hFile,
      Pointer<Uint8> lpBuffer,
      Uint32 nNumberOfBytesToWrite,
      Pointer<Uint32> lpNumberOfBytesWritten,
      Pointer<Void> lpOverlapped,
    );
typedef _WriteFileDart =
    int Function(
      int hFile,
      Pointer<Uint8> lpBuffer,
      int nNumberOfBytesToWrite,
      Pointer<Uint32> lpNumberOfBytesWritten,
      Pointer<Void> lpOverlapped,
    );

typedef _ReadFileNative =
    Int32 Function(
      IntPtr hFile,
      Pointer<Uint8> lpBuffer,
      Uint32 nNumberOfBytesToRead,
      Pointer<Uint32> lpNumberOfBytesRead,
      Pointer<Void> lpOverlapped,
    );
typedef _ReadFileDart =
    int Function(
      int hFile,
      Pointer<Uint8> lpBuffer,
      int nNumberOfBytesToRead,
      Pointer<Uint32> lpNumberOfBytesRead,
      Pointer<Void> lpOverlapped,
    );

class _Win32 {
  _Win32._() {
    assert(Platform.isWindows, '_Win32 requires Windows');
  }
  static _Win32? _instance;
  static _Win32 get instance => _instance ??= _Win32._();

  late final _kernel32 = DynamicLibrary.open('kernel32.dll');
  late final createMutex = _kernel32
      .lookupFunction<_CreateMutexWNative, _CreateMutexWDart>('CreateMutexW');
  late final getLastError = _kernel32
      .lookupFunction<_GetLastErrorNative, _GetLastErrorDart>('GetLastError');
  late final closeHandle = _kernel32
      .lookupFunction<_CloseHandleNative, _CloseHandleDart>('CloseHandle');
  late final releaseMutex = _kernel32
      .lookupFunction<_ReleaseMutexNative, _ReleaseMutexDart>('ReleaseMutex');
  late final createNamedPipe = _kernel32
      .lookupFunction<_CreateNamedPipeWNative, _CreateNamedPipeWDart>(
        'CreateNamedPipeW',
      );
  late final connectNamedPipe = _kernel32
      .lookupFunction<_ConnectNamedPipeNative, _ConnectNamedPipeDart>(
        'ConnectNamedPipe',
      );
  late final disconnectNamedPipe = _kernel32
      .lookupFunction<_DisconnectNamedPipeNative, _DisconnectNamedPipeDart>(
        'DisconnectNamedPipe',
      );
  late final createFile = _kernel32
      .lookupFunction<_CreateFileWNative, _CreateFileWDart>('CreateFileW');
  late final writeFile = _kernel32
      .lookupFunction<_WriteFileNative, _WriteFileDart>('WriteFile');
  late final readFile = _kernel32
      .lookupFunction<_ReadFileNative, _ReadFileDart>('ReadFile');
  late final waitForSingleObject = _kernel32
      .lookupFunction<_WaitForSingleObjectNative, _WaitForSingleObjectDart>(
        'WaitForSingleObject',
      );

  late final _user32 = DynamicLibrary.open('user32.dll');
  late final allowSetForegroundWindow = _user32
      .lookupFunction<
        _AllowSetForegroundWindowNative,
        _AllowSetForegroundWindowDart
      >('AllowSetForegroundWindow');
}

// Named pipe constants
const int _pipeAccessInbound = 1;
const int _pipeTypeByte = 0;
const int _pipeWait = 0;
const int _pipeUnlimitedInstances = 255;
const int _genericWrite = 0x40000000;
const int _openExisting = 3;
const int _invalidHandleValue = -1;

const String _pipeName = r'\\.\pipe\CopyPasteSingleInstance';

class SingleInstance {
  static const String _mutexName = r'Local\CopyPaste_SingleInstance_Mutex';
  static const String _wakeupFileName = 'copypaste.wakeup';

  static int _mutexHandle = 0;
  static RandomAccessFile? _lockFile;
  static StreamSubscription<void>? _wakeupSubscription;
  static Isolate? _pipeIsolate;
  static ReceivePort? _pipeReceivePort;
  static DateTime? _lastWakeup;

  static void _callWakeup(void Function() onWakeup) {
    final now = DateTime.now();
    if (_lastWakeup != null &&
        now.difference(_lastWakeup!).inMilliseconds < 2000) {
      return;
    }
    _lastWakeup = now;
    onWakeup();
  }

  static bool acquire() {
    bool acquired;
    if (Platform.isWindows) {
      acquired = _acquireWindows();
    } else if (Platform.isMacOS || Platform.isLinux) {
      acquired = _acquireUnix();
    } else {
      return true;
    }
    if (!acquired) signalWakeup();
    return acquired;
  }

  static void release() {
    if (Platform.isWindows) {
      _releaseWindows();
    } else {
      _releaseUnix();
    }
  }

  /// Writes a wakeup signal so the running instance can show its window.
  /// On Windows uses a named pipe; falls back to file on other platforms.
  /// Also grants foreground permission so SetForegroundWindow works.
  static void signalWakeup() {
    if (Platform.isWindows) {
      _signalWakeupPipe();
    } else {
      _signalWakeupFile();
    }
  }

  static void _signalWakeupPipe() {
    try {
      final w = _Win32.instance;
      // Grant foreground permission before connecting
      w.allowSetForegroundWindow(0xFFFFFFFF);

      final name = _pipeName.toNativeUtf16();
      try {
        final hPipe = w.createFile(
          name,
          _genericWrite,
          0,
          nullptr,
          _openExisting,
          0,
          0,
        );
        if (hPipe == _invalidHandleValue) {
          // Pipe not available, fall back to file
          _signalWakeupFile();
          return;
        }
        final msg = 'wakeup'.codeUnits;
        final buf = calloc<Uint8>(msg.length);
        final written = calloc<Uint32>(1);
        try {
          for (var i = 0; i < msg.length; i++) {
            buf[i] = msg[i];
          }
          w.writeFile(hPipe, buf, msg.length, written, nullptr);
        } finally {
          calloc.free(buf);
          calloc.free(written);
          w.closeHandle(hPipe);
        }
      } finally {
        calloc.free(name);
      }
    } catch (_) {
      _signalWakeupFile();
    }
  }

  static void _signalWakeupFile() {
    try {
      File(_wakeupFilePath()).writeAsStringSync('wakeup');
      if (Platform.isWindows) {
        _Win32.instance.allowSetForegroundWindow(0xFFFFFFFF);
      }
    } catch (_) {}
  }

  /// Starts listening for wakeup signals. On Windows uses a named pipe server
  /// running in a separate isolate; on other platforms polls a file.
  static void listenForWakeup(void Function() onWakeup) {
    _wakeupSubscription?.cancel();
    if (Platform.isWindows) {
      _listenForWakeupPipe(onWakeup);
    } else {
      _listenForWakeupFile(onWakeup);
    }
  }

  static void _listenForWakeupPipe(void Function() onWakeup) {
    _pipeReceivePort?.close();
    _pipeIsolate?.kill(priority: Isolate.immediate);

    _pipeReceivePort = ReceivePort();
    _pipeReceivePort!.listen((message) {
      if (message == 'wakeup') _callWakeup(onWakeup);
    });

    // Also keep file-based polling as safety net
    _listenForWakeupFile(onWakeup);

    Isolate.spawn(_pipeServerLoop, _pipeReceivePort!.sendPort)
        .then((isolate) {
          _pipeIsolate = isolate;
        })
        .catchError((_) {
          // Isolate spawn failed; file-based fallback is already running
        });
  }

  /// Runs in a dedicated isolate. Blocks on ConnectNamedPipe waiting for
  /// second-instance clients, then reads their message and forwards it.
  static void _pipeServerLoop(SendPort sendPort) {
    final kernel32 = DynamicLibrary.open('kernel32.dll');
    final createNamedPipe = kernel32
        .lookupFunction<_CreateNamedPipeWNative, _CreateNamedPipeWDart>(
          'CreateNamedPipeW',
        );
    final connectNamedPipe = kernel32
        .lookupFunction<_ConnectNamedPipeNative, _ConnectNamedPipeDart>(
          'ConnectNamedPipe',
        );
    final disconnectNamedPipe = kernel32
        .lookupFunction<_DisconnectNamedPipeNative, _DisconnectNamedPipeDart>(
          'DisconnectNamedPipe',
        );
    final readFile = kernel32.lookupFunction<_ReadFileNative, _ReadFileDart>(
      'ReadFile',
    );
    final closeHandle = kernel32
        .lookupFunction<_CloseHandleNative, _CloseHandleDart>('CloseHandle');
    final getLastError = kernel32
        .lookupFunction<_GetLastErrorNative, _GetLastErrorDart>('GetLastError');

    while (true) {
      final name = _pipeName.toNativeUtf16();
      final hPipe = createNamedPipe(
        name,
        _pipeAccessInbound,
        _pipeTypeByte | _pipeWait,
        _pipeUnlimitedInstances,
        512,
        512,
        5000,
        nullptr,
      );
      calloc.free(name);

      if (hPipe == _invalidHandleValue) {
        // Cannot create pipe; wait and retry
        sleep(const Duration(seconds: 2));
        continue;
      }

      final connected = connectNamedPipe(hPipe, nullptr);
      if (connected == 0) {
        const errorPipeConnected = 535;
        if (getLastError() != errorPipeConnected) {
          disconnectNamedPipe(hPipe);
          closeHandle(hPipe);
          continue;
        }
      }

      final buf = calloc<Uint8>(512);
      final bytesRead = calloc<Uint32>(1);
      try {
        final ok = readFile(hPipe, buf, 512, bytesRead, nullptr);
        if (ok != 0 && bytesRead.value > 0) {
          final data = List<int>.generate(bytesRead.value, (i) => buf[i]);
          final msg = String.fromCharCodes(data);
          if (msg.contains('wakeup')) {
            sendPort.send('wakeup');
          }
        }
      } finally {
        calloc.free(buf);
        calloc.free(bytesRead);
      }

      disconnectNamedPipe(hPipe);
      closeHandle(hPipe);
    }
  }

  static void _listenForWakeupFile(void Function() onWakeup) {
    try {
      final stale = File(_wakeupFilePath());
      if (stale.existsSync()) {
        final age = DateTime.now().difference(stale.lastModifiedSync());
        if (age.inSeconds > 30) stale.deleteSync();
      }
    } catch (_) {}
    _wakeupSubscription =
        Stream<void>.periodic(const Duration(milliseconds: 500)).listen((_) {
          final f = File(_wakeupFilePath());
          if (f.existsSync()) {
            try {
              f.deleteSync();
            } catch (_) {}
            _callWakeup(onWakeup);
          }
        });
  }

  /// Stops listening for wakeup signals.
  static void stopListening() {
    _wakeupSubscription?.cancel();
    _wakeupSubscription = null;
    _pipeReceivePort?.close();
    _pipeReceivePort = null;
    _pipeIsolate?.kill(priority: Isolate.immediate);
    _pipeIsolate = null;
    _lastWakeup = null;
  }

  static String _wakeupFilePath() =>
      '${Directory.systemTemp.path}/$_wakeupFileName';

  static bool _acquireWindows() {
    if (_mutexHandle != 0) return false;
    final w = _Win32.instance;
    final name = _mutexName.toNativeUtf16();
    try {
      final handle = w.createMutex(nullptr, 0, name);
      if (handle == 0) return false;

      // WaitForSingleObject is reliable regardless of GetLastError state;
      // the Dart FFI trampoline can clobber the thread-local error between
      // consecutive calls, making GetLastError-based checks unreliable.
      const waitObject0 = 0;
      const waitAbandoned = 0x80;
      final result = w.waitForSingleObject(handle, 0);
      if (result == waitObject0 || result == waitAbandoned) {
        _mutexHandle = handle;
        return true;
      }
      w.closeHandle(handle);
      return false;
    } finally {
      calloc.free(name);
    }
  }

  static void _releaseWindows() {
    stopListening();
    if (_mutexHandle != 0) {
      final w = _Win32.instance;
      w.releaseMutex(_mutexHandle);
      w.closeHandle(_mutexHandle);
      _mutexHandle = 0;
    }
  }

  static bool _acquireUnix() {
    final lockPath = _lockFilePath();
    try {
      _lockFile = File(lockPath).openSync(mode: FileMode.write);
      _lockFile!.lockSync(FileLock.exclusive);
      _lockFile!.writeStringSync('$pid\n');
      _lockFile!.flushSync();
      return true;
    } catch (_) {
      _lockFile = null;
      if (_isLockStale(lockPath)) {
        try {
          File(lockPath).deleteSync();
        } catch (_) {}
        try {
          _lockFile = File(lockPath).openSync(mode: FileMode.write);
          _lockFile!.lockSync(FileLock.exclusive);
          _lockFile!.writeStringSync('$pid\n');
          _lockFile!.flushSync();
          return true;
        } catch (_) {
          _lockFile = null;
        }
      }
      return false;
    }
  }

  static bool _isLockStale(String lockPath) {
    try {
      final content = File(lockPath).readAsStringSync().trim();
      final existingPid = int.tryParse(content);
      if (existingPid == null) return true;
      final result = Process.runSync('kill', ['-0', '$existingPid']);
      return result.exitCode != 0;
    } catch (_) {
      return true;
    }
  }

  static void _releaseUnix() {
    try {
      _lockFile?.unlockSync();
      _lockFile?.closeSync();
      File(_lockFilePath()).deleteSync();
    } catch (_) {}
    _lockFile = null;
  }

  static String _lockFilePath() {
    final tmpDir = Directory.systemTemp.path;
    return '$tmpDir/copypaste.lock';
  }
}
