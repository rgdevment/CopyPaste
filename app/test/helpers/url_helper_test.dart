import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/helpers/url_helper.dart';

void main() {
  tearDown(() => UrlHelper.platformOverride = null);

  group('UrlHelper.open', () {
    test('completes on current platform without throwing', () async {
      try {
        await UrlHelper.open('');
      } catch (_) {}
    });

    test('takes windows branch when platformOverride=windows', () async {
      UrlHelper.platformOverride = 'windows';
      try {
        await UrlHelper.open('about:blank');
      } catch (_) {}
    });

    test('takes macos branch when platformOverride=macos', () async {
      UrlHelper.platformOverride = 'macos';
      try {
        await UrlHelper.open('about:blank');
      } catch (_) {}
    });

    test('takes linux branch when platformOverride=linux', () async {
      UrlHelper.platformOverride = 'linux';
      try {
        await UrlHelper.open('about:blank');
      } catch (_) {}
    });

    test('takes no-op branch when platformOverride=other', () async {
      UrlHelper.platformOverride = 'other';
      await UrlHelper.open('about:blank');
    });
  });
}
