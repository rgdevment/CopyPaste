import 'dart:io';

import 'package:auto_updater/auto_updater.dart';
import 'package:core/core.dart';

const _isStoreBuild = bool.fromEnvironment('STORE_BUILD', defaultValue: false);

class AutoUpdateService {
  static const _feedUrl =
      'https://raw.githubusercontent.com/rgdevment/CopyPaste/main/appcast.xml';
  static const _checkIntervalSeconds = 43200; // 12 hours

  static Future<void> initialize() async {
    if (_isStoreBuild || !Platform.isWindows) return;
    try {
      final updater = AutoUpdater.instance;
      await updater.setFeedURL(_feedUrl);
      await updater.setScheduledCheckInterval(_checkIntervalSeconds);
      await updater.checkForUpdates(inBackground: true);
    } catch (e) {
      AppLogger.error('AutoUpdateService init failed: $e');
    }
  }
}
