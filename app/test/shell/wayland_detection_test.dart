import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/main.dart' show isWaylandSession;

void main() {
  group('isWaylandSession', () {
    test('returns false on non-Linux platforms', () {
      if (Platform.isLinux) return;
      expect(isWaylandSession(), isFalse);
    });

    test(
      'returns false when neither env var is set (typical X11 / headless)',
      () {
        final sessionType = Platform.environment['XDG_SESSION_TYPE'] ?? '';
        final waylandDisplay = Platform.environment['WAYLAND_DISPLAY'] ?? '';

        final expected = sessionType == 'wayland' || waylandDisplay.isNotEmpty;
        if (!expected) {
          expect(isWaylandSession(), isFalse);
        }
      },
    );
  });
}
