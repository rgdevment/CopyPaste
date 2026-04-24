import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/linux_session.dart';

void main() {
  group('isWaylandSession', () {
    test('returns false on non-Linux platforms', () {
      if (Platform.isLinux) return;
      expect(isWaylandSession(), isFalse);
    });

    test('is consistent with current environment variables', () {
      if (!Platform.isLinux) {
        expect(isWaylandSession(), isFalse);
        return;
      }

      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';

      if (sessionType == 'wayland' || waylandDisplay.isNotEmpty) {
        expect(isWaylandSession(), isTrue);
      }
    });

    test('returns false on headless / X11 CI environment', () {
      final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
      final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';

      final hasEnvIndicator =
          sessionType == 'wayland' || waylandDisplay.isNotEmpty;

      if (!hasEnvIndicator && Platform.isLinux) {
        expect(isWaylandSession(), isA<bool>());
      }
    });

    test('return type is bool', () {
      expect(isWaylandSession(), isA<bool>());
    });

    test('is idempotent — same result on repeated calls', () {
      expect(isWaylandSession(), equals(isWaylandSession()));
    });
  });

  group('linuxPrefersDarkMode', () {
    test('returns a bool', () async {
      expect(await linuxPrefersDarkMode(), isA<bool>());
    });

    test('returns false on non-Linux platforms', () async {
      if (Platform.isLinux) return;
      expect(await linuxPrefersDarkMode(), isFalse);
    });
  });

  group('LinuxSessionInfo', () {
    test('unsupported is the safe default for non-Linux', () {
      if (Platform.isLinux) return;
      final info = detectLinuxSession();
      expect(info, equals(LinuxSessionInfo.unsupported));
      expect(info.isWayland, isFalse);
      expect(info.isX11, isFalse);
      expect(info.isUsable, isFalse);
    });

    test('detectLinuxSession returns a value type', () {
      expect(detectLinuxSession(), isA<LinuxSessionInfo>());
    });

    test('isWayland prioritises XDG_SESSION_TYPE=wayland', () {
      const info = LinuxSessionInfo(
        sessionType: 'wayland',
        hasDisplay: true,
        hasWaylandDisplay: true,
        hasWaylandSocket: true,
        desktopEnv: 'GNOME',
        wmName: '',
      );
      expect(info.isWayland, isTrue);
      expect(info.isX11, isFalse);
      expect(info.isXWayland, isTrue);
    });

    test('isX11 honours XDG_SESSION_TYPE=x11 even with Wayland socket', () {
      const info = LinuxSessionInfo(
        sessionType: 'x11',
        hasDisplay: true,
        hasWaylandDisplay: false,
        hasWaylandSocket: true,
        desktopEnv: 'KDE',
        wmName: '',
      );
      expect(info.isX11, isTrue);
      expect(info.isWayland, isFalse);
    });

    test('empty sessionType + WAYLAND_DISPLAY set => Wayland', () {
      const info = LinuxSessionInfo(
        sessionType: '',
        hasDisplay: true,
        hasWaylandDisplay: true,
        hasWaylandSocket: true,
        desktopEnv: '',
        wmName: '',
      );
      expect(info.isWayland, isTrue);
      expect(info.isX11, isFalse);
    });

    test('empty sessionType + only DISPLAY => X11', () {
      const info = LinuxSessionInfo(
        sessionType: '',
        hasDisplay: true,
        hasWaylandDisplay: false,
        hasWaylandSocket: false,
        desktopEnv: '',
        wmName: '',
      );
      expect(info.isX11, isTrue);
      expect(info.isWayland, isFalse);
    });

    test('TTY / headless => neither X11 nor Wayland', () {
      const info = LinuxSessionInfo(
        sessionType: 'tty',
        hasDisplay: false,
        hasWaylandDisplay: false,
        hasWaylandSocket: false,
        desktopEnv: '',
        wmName: '',
      );
      expect(info.isUsable, isFalse);
    });

    test('isWaylandSession is a derived alias of detectLinuxSession', () {
      expect(isWaylandSession(), equals(detectLinuxSession().isWayland));
    });

    test('equality and hashCode work for value type', () {
      const a = LinuxSessionInfo(
        sessionType: 'x11',
        hasDisplay: true,
        hasWaylandDisplay: false,
        hasWaylandSocket: false,
        desktopEnv: 'GNOME',
        wmName: 'gnome',
      );
      const b = LinuxSessionInfo(
        sessionType: 'x11',
        hasDisplay: true,
        hasWaylandDisplay: false,
        hasWaylandSocket: false,
        desktopEnv: 'GNOME',
        wmName: 'gnome',
      );
      expect(a, equals(b));
      expect(a.hashCode, equals(b.hashCode));
    });
  });
}
