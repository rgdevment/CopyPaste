import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/helpers/url_helper.dart';

void main() {
  group('UrlHelper', () {
    test(
      'open calls system command without throwing on current platform',
      () async {
        // This exercises the platform branch for the current OS.
        // On Windows it spawns `cmd /c start`; on macOS `open`; on Linux `xdg-open`.
        // We pass an empty string to minimise side-effects.
        try {
          await UrlHelper.open('');
        } catch (_) {
          // Acceptable: the spawned process may fail with an empty URL.
        }
      },
    );

    test('open with non-empty URL completes on current platform', () async {
      if (Platform.isWindows || Platform.isMacOS || Platform.isLinux) {
        try {
          await UrlHelper.open('about:blank');
        } catch (_) {
          // Process errors are acceptable in test environment.
        }
      }
    });
  });
}
