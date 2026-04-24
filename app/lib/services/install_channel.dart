import 'dart:io';

const bool _isStoreBuild = bool.fromEnvironment(
  'STORE_BUILD',
  defaultValue: false,
);

enum InstallChannel {
  msStore,
  githubWindows,
  githubMacos,
  homebrew,
  githubLinux,
  appImage,
  snap,
  unknown,
}

enum HostPlatform { macos, linux, windows, other }

class InstallChannelDetector {
  static InstallChannel detect({
    String? execPathOverride,
    HostPlatform? platformOverride,
  }) {
    if (_isStoreBuild) return InstallChannel.msStore;
    final path = (execPathOverride ?? Platform.resolvedExecutable).replaceAll(
      r'\',
      '/',
    );
    final host = platformOverride ?? _currentPlatform();

    if (host == HostPlatform.macos) {
      if (_isHomebrewPath(path)) return InstallChannel.homebrew;
      return InstallChannel.githubMacos;
    }

    if (host == HostPlatform.linux) {
      if (path.contains('.AppImage')) return InstallChannel.appImage;
      if (path.startsWith('/snap/')) return InstallChannel.snap;
      return InstallChannel.githubLinux;
    }

    if (host == HostPlatform.windows) return InstallChannel.githubWindows;

    return InstallChannel.unknown;
  }

  static HostPlatform _currentPlatform() {
    if (Platform.isMacOS) return HostPlatform.macos;
    if (Platform.isLinux) return HostPlatform.linux;
    if (Platform.isWindows) return HostPlatform.windows;
    return HostPlatform.other;
  }

  static String manifestKey(InstallChannel channel) {
    switch (channel) {
      case InstallChannel.msStore:
        return 'msstore';
      case InstallChannel.githubWindows:
        return 'github_windows';
      case InstallChannel.githubMacos:
        return 'github_macos';
      case InstallChannel.homebrew:
        return 'homebrew';
      case InstallChannel.githubLinux:
        return 'github_linux';
      case InstallChannel.appImage:
        return 'github_linux';
      case InstallChannel.snap:
        return 'snap';
      case InstallChannel.unknown:
        return 'github_linux';
    }
  }

  static bool _isHomebrewPath(String path) {
    return path.contains('/Cellar/') ||
        path.contains('/opt/homebrew/') ||
        path.contains('/usr/local/Cellar/');
  }
}
