import 'dart:io';

bool isWaylandSession() {
  if (!Platform.isLinux) return false;

  final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
  if (sessionType == 'wayland') return true;
  if (sessionType == 'x11' || sessionType == 'mir') return false;

  final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';
  if (waylandDisplay.isNotEmpty) return true;

  final display = Platform.environment['DISPLAY'] ?? '';
  if (display.isNotEmpty) return false;

  return _hasWaylandSocket();
}

bool _hasWaylandSocket() {
  final runtimeDir = Platform.environment['XDG_RUNTIME_DIR'] ?? '';
  if (runtimeDir.isEmpty) return false;
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
