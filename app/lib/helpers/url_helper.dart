import 'dart:io';

import 'package:flutter/foundation.dart' show visibleForTesting;

class UrlHelper {
  UrlHelper._();

  @visibleForTesting
  static String? platformOverride;

  static Future<void> open(String url) async {
    final platform = platformOverride ?? _currentPlatform();
    if (platform == 'windows') {
      await Process.start('cmd', ['/c', 'start', '', url], runInShell: true);
    } else if (platform == 'macos') {
      await Process.start('open', [url]);
    }
  }

  static String _currentPlatform() {
    if (Platform.isWindows) return 'windows';
    if (Platform.isMacOS) return 'macos';
    return 'other';
  }
}
