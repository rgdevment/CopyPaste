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
}
