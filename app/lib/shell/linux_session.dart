import 'dart:io';

/// Returns true when the current Linux session is running under Wayland.
///
/// Priority:
///   1. GDK_BACKEND=x11 → explicitly forced to X11, never Wayland.
///   2. XDG_SESSION_TYPE=wayland → standard session indicator.
///   3. WAYLAND_DISPLAY set → Wayland socket is active.
bool isWaylandSession() {
  if (!Platform.isLinux) return false;
  if ((Platform.environment['GDK_BACKEND'] ?? '') == 'x11') return false;
  final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
  final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';
  return sessionType == 'wayland' || waylandDisplay.isNotEmpty;
}
