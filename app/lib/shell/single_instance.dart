// coverage:ignore-file
import 'dart:ffi';
import 'dart:io';

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

class _Win32Mutex {
  _Win32Mutex._() {
    assert(Platform.isWindows, '_Win32Mutex requires Windows');
  }
  static _Win32Mutex? _instance;
  static _Win32Mutex get instance => _instance ??= _Win32Mutex._();

  late final _kernel32 = DynamicLibrary.open('kernel32.dll');
  late final createMutex = _kernel32
      .lookupFunction<_CreateMutexWNative, _CreateMutexWDart>('CreateMutexW');
  late final getLastError = _kernel32
      .lookupFunction<_GetLastErrorNative, _GetLastErrorDart>('GetLastError');
  late final closeHandle = _kernel32
      .lookupFunction<_CloseHandleNative, _CloseHandleDart>('CloseHandle');
  late final releaseMutex = _kernel32
      .lookupFunction<_ReleaseMutexNative, _ReleaseMutexDart>('ReleaseMutex');
}

class SingleInstance {
  static const String _mutexName = r'Global\CopyPaste_SingleInstance_Mutex';
  static const int _errorAlreadyExists = 183;

  static int _mutexHandle = 0;
  static RandomAccessFile? _lockFile;

  static bool acquire() {
    if (Platform.isWindows) {
      return _acquireWindows();
    } else if (Platform.isMacOS || Platform.isLinux) {
      return _acquireUnix();
    }
    return true;
  }

  static void release() {
    if (Platform.isWindows) {
      _releaseWindows();
    } else {
      _releaseUnix();
    }
  }

  static bool _acquireWindows() {
    final w = _Win32Mutex.instance;
    final name = _mutexName.toNativeUtf16();
    try {
      _mutexHandle = w.createMutex(nullptr, 1, name);
      if (_mutexHandle == 0) return false;

      final error = w.getLastError();
      if (error == _errorAlreadyExists) {
        w.closeHandle(_mutexHandle);
        _mutexHandle = 0;
        return false;
      }
      return true;
    } finally {
      calloc.free(name);
    }
  }

  static void _releaseWindows() {
    if (_mutexHandle != 0) {
      final w = _Win32Mutex.instance;
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
