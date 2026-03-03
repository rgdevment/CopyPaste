import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  group('UpdateChecker.isNewer', () {
    test('true when major version is higher', () {
      expect(UpdateChecker.isNewer('2.0.0', '1.9.9'), isTrue);
    });

    test('true when minor version is higher', () {
      expect(UpdateChecker.isNewer('1.5.0', '1.4.9'), isTrue);
    });

    test('true when patch version is higher', () {
      expect(UpdateChecker.isNewer('1.0.2', '1.0.1'), isTrue);
    });

    test('false for equal versions', () {
      expect(UpdateChecker.isNewer('1.0.0', '1.0.0'), isFalse);
    });

    test('false when latest is lower than current', () {
      expect(UpdateChecker.isNewer('1.0.0', '2.0.0'), isFalse);
      expect(UpdateChecker.isNewer('1.4.9', '1.5.0'), isFalse);
    });

    test('handles version without patch segment', () {
      expect(UpdateChecker.isNewer('1.1', '1.0.0'), isTrue);
      expect(UpdateChecker.isNewer('1.0', '1.0.0'), isFalse);
    });

    test('returns false for invalid version strings', () {
      expect(UpdateChecker.isNewer('invalid', '1.0.0'), isFalse);
      expect(UpdateChecker.isNewer('1.0.0', 'bad'), isFalse);
    });
  });

  group('UpdateInfo', () {
    test('stores version and downloadUrl', () {
      const info = UpdateInfo(
        version: '2.0.0',
        downloadUrl: 'https://github.com/releases/v2.0.0',
      );
      expect(info.version, equals('2.0.0'));
      expect(info.downloadUrl, equals('https://github.com/releases/v2.0.0'));
    });
  });

  group('UpdateChecker', () {
    test('onUpdateAvailable is a broadcast stream', () {
      final checker = UpdateChecker();
      expect(checker.onUpdateAvailable.isBroadcast, isTrue);
      checker.dispose();
    });

    test('dispose closes stream without error', () {
      final checker = UpdateChecker();
      expect(checker.dispose, returnsNormally);
    });
  });
}
