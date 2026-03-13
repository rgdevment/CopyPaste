import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

// isWaylandSession() reads Platform.environment which is immutable in Dart,
// so we test by verifying consistent behavior given known env values.
// On the CI/test machine XDG_SESSION_TYPE and WAYLAND_DISPLAY are not set
// (X11 or headless), so isWaylandSession() should return false by default.

import 'package:copypaste/main.dart' show isWaylandSession;

void main() {
  group('isWaylandSession', () {
    test('returns false on non-Linux platforms', () {
      if (Platform.isLinux) return; // skip on Linux — env varies
      expect(isWaylandSession(), isFalse);
    });

    test(
      'returns false when neither env var is set (typical X11 / headless)',
      () {
        // In the test environment there should be no Wayland vars.
        // This test is meaningful on Linux CI (X11 or headless).
        final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
        final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';

        final expected = sessionType == 'wayland' || waylandDisplay.isNotEmpty;
        expect(isWaylandSession(), equals(expected));
      },
    );
  });
}
