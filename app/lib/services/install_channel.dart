import 'dart:io';

import 'package:flutter/foundation.dart' show visibleForTesting;

const bool _isStoreBuild = bool.fromEnvironment(
  'STORE_BUILD',
  defaultValue: false,
);

enum InstallChannel {
  msStore,
  githubWindows,
  scoop,
  githubMacos,
  homebrew,
  unknown,
}

enum HostPlatform { macos, windows, other }

class InstallChannelDetector {
  static HostPlatform? platformOverride;

  @visibleForTesting
  static InstallChannel? channelOverride;

  static InstallChannel detect({
    String? execPathOverride,
    HostPlatform? platformOverride,
  }) {
    if (channelOverride != null) return channelOverride!;
    if (_isStoreBuild) return InstallChannel.msStore;
    final path = (execPathOverride ?? Platform.resolvedExecutable).replaceAll(
      r'\',
      '/',
    );
    final host =
        platformOverride ??
        InstallChannelDetector.platformOverride ??
        _currentPlatform();

    if (host == HostPlatform.macos) {
      if (_isHomebrewPath(path)) return InstallChannel.homebrew;
      return InstallChannel.githubMacos;
    }

    if (host == HostPlatform.windows) {
      if (_isScoopPath(path)) return InstallChannel.scoop;
      return InstallChannel.githubWindows;
    }

    return InstallChannel.unknown;
  }

  static HostPlatform _currentPlatform() {
    if (Platform.isMacOS) return HostPlatform.macos;
    if (Platform.isWindows) return HostPlatform.windows;
    return HostPlatform.other;
  }

  static String manifestKey(InstallChannel channel) {
    switch (channel) {
      case InstallChannel.msStore:
        return 'msstore';
      case InstallChannel.githubWindows:
        return 'github_windows';
      case InstallChannel.scoop:
        return 'scoop';
      case InstallChannel.githubMacos:
        return 'github_macos';
      case InstallChannel.homebrew:
        return 'homebrew';
      // Deliberately absent from the manifest: an unidentified host has no
      // install channel, so the blocked-version screen falls back to the hint.
      case InstallChannel.unknown:
        return 'unknown';
    }
  }

  static bool _isHomebrewPath(String path) {
    return path.contains('/Cellar/') ||
        path.contains('/opt/homebrew/') ||
        path.contains('/usr/local/Cellar/');
  }

  // The Scoop root is relocatable, so the layout below it is the tell.
  static bool _isScoopPath(String path) {
    final lower = path.toLowerCase();
    return lower.contains('/scoop/apps/') ||
        lower.contains('/apps/copypaste/') ||
        lower.contains('/apps/copypaste-beta/');
  }
}
