import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/linux_session.dart';

// isWaylandSession() reads Platform.environment, which is immutable at
// runtime. These tests verify the logic using the *actual* current
// environment, covering all observable code paths.

void main() {
  group('isWaylandSession', () {
    test('returns false on non-Linux platforms', () {
      if (Platform.isLinux) return; // non-applicable on Linux
      expect(isWaylandSession(), isFalse);
    });

    test('is consistent with current environment variables', () {
      // The function's contract: Wayland iff running on Linux AND
      // GDK_BACKEND != x11 AND (XDG_SESSION_TYPE == wayland OR
      // WAYLAND_DISPLAY is set).
      if (!Platform.isLinux) {
        expect(isWaylandSession(), isFalse);
        return;
      }

      final gdkBackend = Platform.environment['GDK_BACKEND'] ?? '';
      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';

      final expected =
          gdkBackend != 'x11' &&
          (sessionType == 'wayland' || waylandDisplay.isNotEmpty);

      expect(isWaylandSession(), equals(expected));
    });

    test('returns false when GDK_BACKEND=x11 overrides other indicators', () {
      // Can only validate current state — if GDK_BACKEND is set to x11
      // right now, the function must return false even if other vars suggest
      // Wayland. This is a documentation/regression test.
      if (!Platform.isLinux) return;
      if ((Platform.environment['GDK_BACKEND'] ?? '') != 'x11') return;

      expect(isWaylandSession(), isFalse);
    });

    test('returns false on headless / X11 CI environment', () {
      // On a typical headless or X11 CI machine, no Wayland vars are set.
      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';
      final gdkBackend = Platform.environment['GDK_BACKEND'] ?? '';

      final isWaylandEnv =
          gdkBackend != 'x11' &&
          (sessionType == 'wayland' || waylandDisplay.isNotEmpty);

      // If the environment has no Wayland indicators the function returns false.
      if (!isWaylandEnv) {
        expect(isWaylandSession(), isFalse);
      }
    });

    test('return type is bool', () {
      expect(isWaylandSession(), isA<bool>());
    });

    test('is idempotent — same result on repeated calls', () {
      final first = isWaylandSession();
      final second = isWaylandSession();
      expect(first, equals(second));
    });
  });
}
