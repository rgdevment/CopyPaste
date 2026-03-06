// coverage:ignore-file
import 'dart:io';

import 'package:auto_updater/auto_updater.dart';
import 'package:core/core.dart';

const _isStoreBuild = bool.fromEnvironment('STORE_BUILD', defaultValue: false);

class AutoUpdateService {
  static const _feedUrl =
      'https://gist.githubusercontent.com/rgdevment/7e343fffd2920f2de7f8899c10e18ca4/raw/appcast.xml';
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
