import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/desktop_notifier.dart';

void main() {
  group('DesktopNotifier – macOS', () {
    test('returns false on macOS (no-op)', () async {
      if (!Platform.isMacOS) return;
      final result = await DesktopNotifier.show(title: 'Test', body: 'Body');
      expect(result, isFalse);
    });
  });

  group('DesktopNotifier – unsupported hosts', () {
    test(
      'returns false when the platform has no notification channel',
      () async {
        if (Platform.isWindows) return;
        final result = await DesktopNotifier.show(title: 'Test', body: 'Body');
        expect(result, isFalse);
      },
    );
  });
}
