import 'dart:async';
import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import '../shell/linux_session.dart';

@immutable
class LinuxCapabilities {
  const LinuxCapabilities({
    required this.session,
    required this.isX11,
    required this.hasXTest,
    required this.hasAppIndicator,
    required this.hasEwmh,
    required this.detectedDesktopEnv,
    required this.detectedWmName,
    required this.detectionTimedOut,
  });

  final LinuxSessionInfo session;
  final bool isX11;
  final bool hasXTest;
  final bool hasAppIndicator;
  final bool hasEwmh;
  final String detectedDesktopEnv;
  final String detectedWmName;
  final bool detectionTimedOut;

  bool get isLinux => Platform.isLinux;
  bool get isWayland => session.isWayland;
  bool get isUsable => isLinux && isX11;

  static const LinuxCapabilities unsupported = LinuxCapabilities(
    session: LinuxSessionInfo.unsupported,
    isX11: false,
    hasXTest: false,
    hasAppIndicator: false,
    hasEwmh: false,
    detectedDesktopEnv: '',
    detectedWmName: '',
    detectionTimedOut: false,
  );

  LinuxCapabilities copyWith({
    bool? isX11,
    bool? hasXTest,
    bool? hasAppIndicator,
    bool? hasEwmh,
    String? detectedDesktopEnv,
    String? detectedWmName,
    bool? detectionTimedOut,
  }) {
    return LinuxCapabilities(
      session: session,
      isX11: isX11 ?? this.isX11,
      hasXTest: hasXTest ?? this.hasXTest,
      hasAppIndicator: hasAppIndicator ?? this.hasAppIndicator,
      hasEwmh: hasEwmh ?? this.hasEwmh,
      detectedDesktopEnv: detectedDesktopEnv ?? this.detectedDesktopEnv,
      detectedWmName: detectedWmName ?? this.detectedWmName,
      detectionTimedOut: detectionTimedOut ?? this.detectionTimedOut,
    );
  }

  @override
  String toString() =>
      'LinuxCapabilities(isX11=$isX11, hasXTest=$hasXTest, '
      'hasAppIndicator=$hasAppIndicator, '
      'hasEwmh=$hasEwmh, desktopEnv=$detectedDesktopEnv, wm=$detectedWmName, '
      'timedOut=$detectionTimedOut, session=$session)';
}

abstract class LinuxCapabilitiesChannel {
  Future<Map<Object?, Object?>?> invokeShell(String method);
  Future<Map<Object?, Object?>?> invokeListener(String method);
}

class _DefaultLinuxCapabilitiesChannel implements LinuxCapabilitiesChannel {
  const _DefaultLinuxCapabilitiesChannel();

  static const MethodChannel _shell = MethodChannel('copypaste/linux_shell');
  static const MethodChannel _listener = MethodChannel(
    'copypaste/clipboard_writer',
  );

  @override
  Future<Map<Object?, Object?>?> invokeShell(String method) async {
    final result = await _shell.invokeMethod<Object>(method);
    return result is Map ? Map<Object?, Object?>.from(result) : null;
  }

  @override
  Future<Map<Object?, Object?>?> invokeListener(String method) async {
    final result = await _listener.invokeMethod<Object>(method);
    return result is Map ? Map<Object?, Object?>.from(result) : null;
  }
}

class LinuxCapabilitiesService {
  LinuxCapabilitiesService._(); // coverage:ignore-line

  static LinuxCapabilities _cache = LinuxCapabilities.unsupported;
  static bool _initialized = false;

  static LinuxCapabilities get current => _cache;
  static bool get isInitialized => _initialized;

  @visibleForTesting
  static void resetForTesting([LinuxCapabilities? value]) {
    _cache = value ?? LinuxCapabilities.unsupported;
    _initialized = value != null;
  }

  static Future<LinuxCapabilities> detect({
    LinuxCapabilitiesChannel channel = const _DefaultLinuxCapabilitiesChannel(),
    Duration timeout = const Duration(milliseconds: 800),
    @visibleForTesting LinuxSessionInfo? sessionOverride,
  }) async {
    if (!Platform.isLinux) {
      _cache = LinuxCapabilities.unsupported;
      _initialized = true;
      return _cache;
    }

    final session = sessionOverride ?? detectLinuxSession();
    final base = LinuxCapabilities.unsupported.copyWith().copyWithSession(
      session,
    );

    if (!session.isX11) {
      _cache = base;
      _initialized = true;
      return _cache;
    }

    bool timedOut = false;
    Map<Object?, Object?>? shellCaps;
    Map<Object?, Object?>? listenerCaps;

    try {
      final results =
          await Future.wait([
            channel.invokeShell('getCapabilities').catchError((_) => null),
            channel.invokeListener('getCapabilities').catchError((_) => null),
          ]).timeout(
            timeout,
            onTimeout: () {
              timedOut = true;
              return [null, null];
            },
          );
      shellCaps = results[0];
      listenerCaps = results[1];
    } catch (e) {
      AppLogger.warn('LinuxCapabilities.detect failed: $e');
    }

    final result = LinuxCapabilities(
      session: session,
      isX11: _readBool(shellCaps, 'isX11', fallback: true),
      hasXTest: _readBool(listenerCaps, 'hasXTest'),
      hasAppIndicator: _readBool(shellCaps, 'hasAppIndicator'),
      hasEwmh: _readBool(shellCaps, 'hasEwmh'),
      detectedDesktopEnv: _readString(shellCaps, 'desktopEnv'),
      detectedWmName: _readString(shellCaps, 'wmName'),
      detectionTimedOut: timedOut,
    );

    _cache = result;
    _initialized = true;
    return result;
  }

  static bool _readBool(
    Map<Object?, Object?>? map,
    String key, {
    bool fallback = false,
  }) {
    if (map == null) return fallback;
    final value = map[key];
    return value is bool ? value : fallback;
  }

  static String _readString(Map<Object?, Object?>? map, String key) {
    if (map == null) return '';
    final value = map[key];
    return value is String ? value : '';
  }
}

extension _LinuxCapabilitiesSession on LinuxCapabilities {
  LinuxCapabilities copyWithSession(LinuxSessionInfo session) {
    return LinuxCapabilities(
      session: session,
      isX11: isX11,
      hasXTest: hasXTest,
      hasAppIndicator: hasAppIndicator,
      hasEwmh: hasEwmh,
      detectedDesktopEnv: detectedDesktopEnv,
      detectedWmName: detectedWmName,
      detectionTimedOut: detectionTimedOut,
    );
  }
}
