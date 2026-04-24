// coverage:ignore-file

import 'dart:async';

import 'package:core/core.dart';

import 'release_manifest_service.dart';

const _isStoreBuild = bool.fromEnvironment('STORE_BUILD', defaultValue: false);

class AutoUpdateService {
  static StreamSubscription<ManifestState?>? _sub;

  static void Function(String version)? onUpdateAvailable;

  static bool get isStoreBuild => _isStoreBuild;

  static Future<void> initialize({required String storageConfigDir}) async {
    await ReleaseManifestService.initialize(storageConfigDir: storageConfigDir);
    _sub ??= ReleaseManifestService.stream.listen((state) {
      if (state == null) return;
      final latest = state.manifest.latest;
      if (ReleaseManifestService.compareVersions(latest, AppConfig.appVersion) >
          0) {
        AppLogger.info('Update available: ${AppConfig.appVersion} → $latest');
        onUpdateAvailable?.call(latest);
      }
    });

    final cached = ReleaseManifestService.current;
    if (cached != null) {
      final latest = cached.manifest.latest;
      if (ReleaseManifestService.compareVersions(latest, AppConfig.appVersion) >
          0) {
        onUpdateAvailable?.call(latest);
      }
    }
  }

  static Future<void> dispose() async {
    await _sub?.cancel();
    _sub = null;
    ReleaseManifestService.dispose();
  }
}
