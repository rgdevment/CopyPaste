import 'dart:io';

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

    test('handles pre-release suffixes by comparing base version', () {
      expect(UpdateChecker.isNewer('2.0.0-beta.1', '1.9.0'), isTrue);
      expect(UpdateChecker.isNewer('1.0.0-beta.1', '1.0.0'), isFalse);
      expect(UpdateChecker.isNewer('1.1.0-rc.2', '1.0.0'), isTrue);
      expect(UpdateChecker.isNewer('1.0.0', '1.0.0-beta.1'), isFalse);
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

  group('UpdateChecker dismiss', () {
    late Directory tempDir;

    setUp(() {
      tempDir = Directory.systemTemp.createTempSync('dismiss_test_');
    });

    tearDown(() => tempDir.deleteSync(recursive: true));

    test('dismissVersion writes version to file', () {
      final checker = UpdateChecker(configPath: tempDir.path);
      checker.dismissVersion('2.1.0');
      expect(checker.isVersionDismissed('2.1.0'), isTrue);
      checker.dispose();
    });

    test('isVersionDismissed returns false for different version', () {
      final checker = UpdateChecker(configPath: tempDir.path);
      checker.dismissVersion('2.1.0');
      expect(checker.isVersionDismissed('2.2.0'), isFalse);
      checker.dispose();
    });

    test('isVersionDismissed returns false when no file exists', () {
      final checker = UpdateChecker(configPath: tempDir.path);
      expect(checker.isVersionDismissed('1.0.0'), isFalse);
      checker.dispose();
    });

    test('isVersionDismissed returns false when configPath is null', () {
      final checker = UpdateChecker();
      expect(checker.isVersionDismissed('1.0.0'), isFalse);
      checker.dispose();
    });

    test('dismissVersion is no-op when configPath is null', () {
      final checker = UpdateChecker();
      expect(() => checker.dismissVersion('1.0.0'), returnsNormally);
      checker.dispose();
    });

    test('dismiss overwrites previous dismissal', () {
      final checker = UpdateChecker(configPath: tempDir.path);
      checker.dismissVersion('2.0.0');
      checker.dismissVersion('2.1.0');
      expect(checker.isVersionDismissed('2.0.0'), isFalse);
      expect(checker.isVersionDismissed('2.1.0'), isTrue);
      checker.dispose();
    });
  });
}
