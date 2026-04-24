import 'package:copypaste/services/install_channel.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('InstallChannelDetector.detect', () {
    test('detects homebrew on macOS Cellar paths', () {
      final c = InstallChannelDetector.detect(
        execPathOverride: '/opt/homebrew/Cellar/copypaste/2.3.0/bin/copypaste',
        platformOverride: HostPlatform.macos,
      );
      expect(c, InstallChannel.homebrew);
    });

    test('detects appImage paths', () {
      final c = InstallChannelDetector.detect(
        execPathOverride: '/home/user/Apps/CopyPaste-2.3.0.AppImage',
        platformOverride: HostPlatform.linux,
      );
      expect(c, InstallChannel.appImage);
    });

    test('detects snap paths', () {
      final c = InstallChannelDetector.detect(
        execPathOverride: '/snap/copypaste/x1/copypaste',
        platformOverride: HostPlatform.linux,
      );
      expect(c, InstallChannel.snap);
    });
  });

  group('manifestKey', () {
    test('maps every channel to a non-empty key', () {
      for (final c in InstallChannel.values) {
        expect(InstallChannelDetector.manifestKey(c), isNotEmpty);
      }
    });

    test('appImage falls back to github_linux bucket', () {
      expect(
        InstallChannelDetector.manifestKey(InstallChannel.appImage),
        'github_linux',
      );
    });
  });
}
