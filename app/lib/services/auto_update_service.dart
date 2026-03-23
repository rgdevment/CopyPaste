// coverage:ignore-file
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:auto_updater/auto_updater.dart';
import 'package:core/core.dart';
import 'package:flutter/foundation.dart' show visibleForTesting;

const _isStoreBuild = bool.fromEnvironment('STORE_BUILD', defaultValue: false);

class AutoUpdateService {
  static const _feedUrl =
      'https://gist.githubusercontent.com/rgdevment/7e343fffd2920f2de7f8899c10e18ca4/raw/appcast.xml';
  static const _checkIntervalSeconds = 86400; // 24 hours

  static const _releasesUrl =
      'https://api.github.com/repos/rgdevment/CopyPaste/releases/latest';

  static Timer? _timer;
  static void Function(String version)? onUpdateAvailable;

  /// Overrideable for testing — defaults to the real GitHub API URL.
  @visibleForTesting
  static String releasesUrlOverride = '';

  static String get _effectiveReleasesUrl =>
      releasesUrlOverride.isNotEmpty ? releasesUrlOverride : _releasesUrl;

  static Future<void> initialize() async {
    if (_isStoreBuild) return;

    if (Platform.isWindows) {
      await _initWindows();
    } else if (Platform.isMacOS || Platform.isLinux) {
      await _checkGitHubRelease();
      _timer = Timer.periodic(
        const Duration(seconds: _checkIntervalSeconds),
        (_) => _checkGitHubRelease(),
      );
    }
  }

  static Future<void> _initWindows() async {
    try {
      final updater = AutoUpdater.instance;
      await updater.setFeedURL(_feedUrl);
      await updater.setScheduledCheckInterval(_checkIntervalSeconds);
      await updater.checkForUpdates(inBackground: true);
    } catch (e) {
      AppLogger.error('AutoUpdateService init failed: $e');
    }
  }

  static Future<void> _checkGitHubRelease() async {
    final client = HttpClient()
      ..connectionTimeout = const Duration(seconds: 10);
    try {
      final request = await client.getUrl(Uri.parse(_effectiveReleasesUrl));
      request.headers.set('Accept', 'application/vnd.github.v3+json');
      request.headers.set('User-Agent', 'CopyPaste-UpdateChecker');

      final response = await request.close();
      if (response.statusCode != 200) {
        await response.drain<void>();
        return;
      }

      final body = await response.transform(utf8.decoder).join();
      final decoded = jsonDecode(body);
      if (decoded is! Map<String, dynamic>) return;
      final json = decoded;
      final tagName = json['tag_name'] as String?;
      if (tagName == null) return;

      final latest = tagName.startsWith('v') ? tagName.substring(1) : tagName;
      if (!latest.startsWith('2.')) return; // only v2 releases

      const current = AppConfig.appVersion;
      if (_isNewer(latest, current)) {
        AppLogger.info('Update available: $current → $latest');
        onUpdateAvailable?.call(latest);
      }
    } on SocketException {
      // No network — silently ignore
    } on HttpException {
      // Bad response — silently ignore
    } catch (e) {
      AppLogger.error('Update check failed: $e');
    } finally {
      client.close();
    }
  }

  static bool _isNewer(String latest, String current) =>
      isNewerVersion(latest, current);

  @visibleForTesting
  static bool isNewerVersion(String latest, String current) {
    final latestParts = latest.split('-');
    final currentParts = current.split('-');
    final latestBase = latestParts[0].split('.').map(int.tryParse).toList();
    final currentBase = currentParts[0].split('.').map(int.tryParse).toList();

    for (var i = 0; i < 3; i++) {
      final l = i < latestBase.length ? (latestBase[i] ?? 0) : 0;
      final c = i < currentBase.length ? (currentBase[i] ?? 0) : 0;
      if (l > c) return true;
      if (l < c) return false;
    }

    // Same base: release > pre-release
    if (currentParts.length > 1 && latestParts.length == 1) return true;
    return false;
  }

  static void dispose() {
    _timer?.cancel();
    _timer = null;
  }

  /// Resets internal state for testing.
  @visibleForTesting
  static void reset() {
    dispose();
    onUpdateAvailable = null;
    releasesUrlOverride = '';
  }
}
