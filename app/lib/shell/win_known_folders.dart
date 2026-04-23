// coverage:ignore-file
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

final class _Guid extends Struct {
  @Uint32()
  external int data1;
  @Uint16()
  external int data2;
  @Uint16()
  external int data3;
  @Array(8)
  external Array<Uint8> data4;
}

typedef _SHGetKnownFolderPathNative =
    Int32 Function(
      Pointer<_Guid> rfid,
      Uint32 dwFlags,
      IntPtr hToken,
      Pointer<Pointer<Utf16>> ppszPath,
    );
typedef _SHGetKnownFolderPathDart =
    int Function(
      Pointer<_Guid> rfid,
      int dwFlags,
      int hToken,
      Pointer<Pointer<Utf16>> ppszPath,
    );

typedef _CoTaskMemFreeNative = Void Function(Pointer<Void> pv);
typedef _CoTaskMemFreeDart = void Function(Pointer<Void> pv);

class WinKnownFolders {
  WinKnownFolders._();

  static String? localAppData() => _resolve(_folderIdLocalAppData);

  static String? roamingAppData() => _resolve(_folderIdRoamingAppData);

  static const _folderIdLocalAppData = (
    0xF1B32785, 0x6FBA, 0x4FCF, [0x9D, 0x55, 0x7B, 0x8E, 0x7F, 0x15, 0x70, 0x91],
  );
  static const _folderIdRoamingAppData = (
    0x3EB685DB, 0x65F9, 0x4CF6, [0xA0, 0x3A, 0xE3, 0xEF, 0x65, 0x72, 0x9F, 0x3D],
  );

  static String? _resolve(
    (int, int, int, List<int>) guidParts,
  ) {
    if (!Platform.isWindows) return null;
    try {
      final shell32 = DynamicLibrary.open('shell32.dll');
      final ole32 = DynamicLibrary.open('ole32.dll');
      final shGetKnownFolderPath = shell32.lookupFunction<
          _SHGetKnownFolderPathNative,
          _SHGetKnownFolderPathDart>('SHGetKnownFolderPath');
      final coTaskMemFree = ole32
          .lookupFunction<_CoTaskMemFreeNative, _CoTaskMemFreeDart>(
            'CoTaskMemFree',
          );

      final guid = calloc<_Guid>();
      final outPtr = calloc<Pointer<Utf16>>();
      try {
        guid.ref.data1 = guidParts.$1;
        guid.ref.data2 = guidParts.$2;
        guid.ref.data3 = guidParts.$3;
        for (var i = 0; i < 8; i++) {
          guid.ref.data4[i] = guidParts.$4[i];
        }
        final hr = shGetKnownFolderPath(guid, 0, 0, outPtr);
        if (hr != 0) return null;
        final path = outPtr.value.toDartString();
        coTaskMemFree(outPtr.value.cast());
        return path;
      } finally {
        calloc.free(guid);
        calloc.free(outPtr);
      }
    } catch (_) {
      return null;
    }
  }
}
