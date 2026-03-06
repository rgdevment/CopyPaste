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

class SingleInstance {
  static const String _mutexName = r'Global\CopyPaste_SingleInstance_Mutex';
  static const int _errorAlreadyExists = 183;

  static final _kernel32 = DynamicLibrary.open('kernel32.dll');

  static final _createMutex = _kernel32
      .lookupFunction<_CreateMutexWNative, _CreateMutexWDart>('CreateMutexW');
  static final _getLastError = _kernel32
      .lookupFunction<_GetLastErrorNative, _GetLastErrorDart>('GetLastError');
  static final _closeHandle = _kernel32
      .lookupFunction<_CloseHandleNative, _CloseHandleDart>('CloseHandle');
  static final _releaseMutex = _kernel32
      .lookupFunction<_ReleaseMutexNative, _ReleaseMutexDart>('ReleaseMutex');

  static int _mutexHandle = 0;

  static bool acquire() {
    if (!Platform.isWindows) return true;

    final name = _mutexName.toNativeUtf16();
    try {
      _mutexHandle = _createMutex(nullptr, 1, name);
      if (_mutexHandle == 0) return false;

      final error = _getLastError();
      if (error == _errorAlreadyExists) {
        _closeHandle(_mutexHandle);
        _mutexHandle = 0;
        return false;
      }
      return true;
    } finally {
      calloc.free(name);
    }
  }

  static void release() {
    if (_mutexHandle != 0) {
      _releaseMutex(_mutexHandle);
      _closeHandle(_mutexHandle);
      _mutexHandle = 0;
    }
  }
}
