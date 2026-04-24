import 'dart:io';

import 'package:flutter/foundation.dart';

@immutable
class LinuxSessionInfo {
  const LinuxSessionInfo({
    required this.sessionType,
    required this.hasDisplay,
    required this.hasWaylandDisplay,
    required this.hasWaylandSocket,
    required this.desktopEnv,
    required this.wmName,
  });

  final String sessionType;
  final bool hasDisplay;
  final bool hasWaylandDisplay;
  final bool hasWaylandSocket;
  final String desktopEnv;
  final String wmName;

  bool get isWayland {
    if (sessionType == 'wayland') return true;
    if (sessionType == 'x11' || sessionType == 'mir' || sessionType == 'tty') {
      return false;
    }
    if (hasWaylandDisplay) return true;
    if (hasWaylandSocket && !hasDisplay) return true;
    if (hasDisplay) return false;
    return hasWaylandSocket;
  }

  bool get isX11 {
    if (sessionType == 'x11') return true;
    if (sessionType == 'wayland' ||
        sessionType == 'mir' ||
        sessionType == 'tty') {
      return false;
    }
    if (hasDisplay && !hasWaylandDisplay && !hasWaylandSocket) return true;
    return false;
  }

  bool get isXWayland =>
      hasDisplay &&
      (hasWaylandDisplay || hasWaylandSocket) &&
      sessionType == 'wayland';

  bool get isUsable => isX11 || isWayland;

  static const LinuxSessionInfo unsupported = LinuxSessionInfo(
    sessionType: '',
    hasDisplay: false,
    hasWaylandDisplay: false,
    hasWaylandSocket: false,
    desktopEnv: '',
    wmName: '',
  );

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is LinuxSessionInfo &&
        other.sessionType == sessionType &&
        other.hasDisplay == hasDisplay &&
        other.hasWaylandDisplay == hasWaylandDisplay &&
        other.hasWaylandSocket == hasWaylandSocket &&
        other.desktopEnv == desktopEnv &&
        other.wmName == wmName;
  }

  @override
  int get hashCode => Object.hash(
    sessionType,
    hasDisplay,
    hasWaylandDisplay,
    hasWaylandSocket,
    desktopEnv,
    wmName,
  );

  @override
  String toString() =>
      'LinuxSessionInfo(sessionType=$sessionType, hasDisplay=$hasDisplay, '
      'hasWaylandDisplay=$hasWaylandDisplay, hasWaylandSocket=$hasWaylandSocket, '
      'desktopEnv=$desktopEnv, wmName=$wmName)';
}

LinuxSessionInfo detectLinuxSession() {
  if (!Platform.isLinux) return LinuxSessionInfo.unsupported;

  final env = Platform.environment;
  final sessionType = (env['XDG_SESSION_TYPE'] ?? '').trim().toLowerCase();
  final display = (env['DISPLAY'] ?? '').trim();
  final waylandDisplay = (env['WAYLAND_DISPLAY'] ?? '').trim();
  final desktopEnv =
      (env['XDG_CURRENT_DESKTOP'] ?? env['DESKTOP_SESSION'] ?? '').trim();
  final wmName = (env['XDG_SESSION_DESKTOP'] ?? '').trim();

  return LinuxSessionInfo(
    sessionType: sessionType,
    hasDisplay: display.isNotEmpty,
    hasWaylandDisplay: waylandDisplay.isNotEmpty,
    hasWaylandSocket: _hasWaylandSocket(env['XDG_RUNTIME_DIR']),
    desktopEnv: desktopEnv,
    wmName: wmName,
  );
}

bool isWaylandSession() => detectLinuxSession().isWayland;

bool _hasWaylandSocket(String? runtimeDir) {
  if (runtimeDir == null || runtimeDir.isEmpty) return false;
  try {
    return Directory(runtimeDir)
        .listSync(followLinks: false)
        .any((e) => e.uri.pathSegments.last.startsWith('wayland'));
  } catch (_) {
    return false;
  }
}

Future<bool> linuxPrefersDarkMode() async {
  if (!Platform.isLinux) return false;

  try {
    final result = await Process.run('gsettings', [
      'get',
      'org.gnome.desktop.interface',
      'color-scheme',
    ]);
    if (result.exitCode == 0) {
      return (result.stdout as String).contains('dark');
    }
  } catch (_) {}

  final gtkTheme = (Platform.environment['GTK_THEME'] ?? '').toLowerCase();
  return gtkTheme.contains('dark');
}
