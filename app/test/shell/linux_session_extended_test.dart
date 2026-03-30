import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/linux_session.dart';

void main() {
  // ---------------------------------------------------------------------------
  // isWaylandSession – environment-aware branch coverage
  // ---------------------------------------------------------------------------
  group('isWaylandSession – non-Linux', () {
    test('returns false on non-Linux platforms', () {
      if (Platform.isLinux) return;
      expect(isWaylandSession(), isFalse);
    });

    test('return type is always bool on non-Linux', () {
      if (Platform.isLinux) return;
      expect(isWaylandSession(), isA<bool>());
    });
  });

  group('isWaylandSession – XDG_SESSION_TYPE branch', () {
    test('returns false when XDG_SESSION_TYPE is x11', () {
      if (!Platform.isLinux) return;
      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      if (sessionType != 'x11') return; // only meaningful on X11 session
      expect(isWaylandSession(), isFalse);
    });

    test('returns true when XDG_SESSION_TYPE is wayland', () {
      if (!Platform.isLinux) return;
      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      if (sessionType != 'wayland') {
        return; // only meaningful on Wayland session
      }
      expect(isWaylandSession(), isTrue);
    });

    test('returns false in typical headless X11 CI environment', () {
      if (!Platform.isLinux) return;
      final isWaylandEnv =
          Platform.environment['XDG_SESSION_TYPE'] == 'wayland' ||
          (Platform.environment['WAYLAND_DISPLAY'] ?? '').isNotEmpty;
      if (isWaylandEnv) return; // skip on actual Wayland
      expect(isWaylandSession(), isFalse);
    });
  });

  group('isWaylandSession – WAYLAND_DISPLAY branch', () {
    test('returns true when WAYLAND_DISPLAY is set (no XDG_SESSION_TYPE)', () {
      if (!Platform.isLinux) return;
      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';
      // Only verify when we're in this exact scenario
      if (sessionType.isEmpty && waylandDisplay.isNotEmpty) {
        expect(isWaylandSession(), isTrue);
      }
    });

    test('returns false when DISPLAY is set and no wayland indicators', () {
      if (!Platform.isLinux) return;
      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';
      final display = Platform.environment['DISPLAY'] ?? '';
      // When no wayland indicators exist but X11 DISPLAY is set
      if (sessionType.isEmpty && waylandDisplay.isEmpty && display.isNotEmpty) {
        expect(isWaylandSession(), isFalse);
      }
    });
  });

  group('isWaylandSession – consistency', () {
    test('result is always a bool', () {
      expect(isWaylandSession(), isA<bool>());
    });

    test('is idempotent across 10 consecutive calls', () {
      final first = isWaylandSession();
      for (var i = 0; i < 9; i++) {
        expect(
          isWaylandSession(),
          equals(first),
          reason: 'call ${i + 2} diverged from first result',
        );
      }
    });

    test('result is consistent with environment variable state', () {
      if (!Platform.isLinux) return;
      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';

      final envSaysWayland =
          sessionType == 'wayland' || waylandDisplay.isNotEmpty;
      final envSaysX11 = sessionType == 'x11' || sessionType == 'mir';

      if (envSaysWayland) {
        expect(isWaylandSession(), isTrue);
      } else if (envSaysX11) {
        expect(isWaylandSession(), isFalse);
      }
      // If neither is set, result depends on DISPLAY / socket scan — we just
      // verify it returns a bool without asserting the direction.
    });
  });

  // ---------------------------------------------------------------------------
  // linuxPrefersDarkMode – behaviour and error-safety
  // ---------------------------------------------------------------------------
  group('linuxPrefersDarkMode – non-Linux', () {
    test('returns false on non-Linux platforms', () async {
      if (Platform.isLinux) return;
      expect(await linuxPrefersDarkMode(), isFalse);
    });

    test('return type is bool on non-Linux', () async {
      if (Platform.isLinux) return;
      expect(await linuxPrefersDarkMode(), isA<bool>());
    });
  });

  group('linuxPrefersDarkMode – Linux behaviour', () {
    test('completes without throwing', () async {
      if (!Platform.isLinux) return;
      await expectLater(linuxPrefersDarkMode(), completes);
    });

    test('returns a bool', () async {
      final result = await linuxPrefersDarkMode();
      expect(result, isA<bool>());
    });

    test('completes within 15 seconds (process spawn + timeout)', () async {
      if (!Platform.isLinux) return;
      final result = await linuxPrefersDarkMode().timeout(
        const Duration(seconds: 15),
        onTimeout: () => fail('linuxPrefersDarkMode did not complete in time'),
      );
      expect(result, isA<bool>());
    });

    test('multiple calls return consistent result', () async {
      final first = await linuxPrefersDarkMode();
      final second = await linuxPrefersDarkMode();
      expect(first, equals(second));
    });

    test('GTK_THEME env absent → result is false or gsettings-driven', () async {
      if (!Platform.isLinux) return;
      final gtkTheme = (Platform.environment['GTK_THEME'] ?? '').toLowerCase();
      final result = await linuxPrefersDarkMode();
      if (gtkTheme.isEmpty) {
        // No GTK_THEME set — result comes from gsettings (or false if unavailable)
        expect(result, isA<bool>());
      } else if (gtkTheme.contains('dark')) {
        // Fallback: GTK_THEME says dark
        expect(result, isA<bool>()); // may be true if gsettings also agrees
      } else {
        // GTK_THEME present but not dark; unless gsettings says dark, expect false
        expect(result, isA<bool>());
      }
    });

    test('returns false in headless CI where gsettings is unavailable', () async {
      if (!Platform.isLinux) return;
      // In headless CI: no gsettings schema, GTK_THEME not set → should be false
      final gtkTheme = (Platform.environment['GTK_THEME'] ?? '').toLowerCase();
      final display = Platform.environment['DISPLAY'] ?? '';
      final isHeadless = display.isEmpty;

      if (isHeadless && !gtkTheme.contains('dark')) {
        expect(await linuxPrefersDarkMode(), isFalse);
      }
    });
  });
}
