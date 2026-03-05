import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:path/path.dart' as p;

class UpdateInfo {
  const UpdateInfo({
    required this.version,
    required this.downloadUrl,
  });

  final String version;
  final String downloadUrl;
}

class UpdateChecker {
  UpdateChecker({this.configPath});

  static const String _releasesUrl =
      'https://api.github.com/repos/rgdevment/CopyPaste/releases/latest';
  static const Duration _startupDelay = Duration(minutes: 1);
  static const Duration _httpTimeout = Duration(seconds: 15);
  static const String _dismissedFileName = 'dismissed_update.txt';

  final String? configPath;

  final _onUpdateAvailable =
      StreamController<UpdateInfo>.broadcast();

  Stream<UpdateInfo> get onUpdateAvailable =>
      _onUpdateAvailable.stream;

  Timer? _startTimer;
  bool _disposed = false;

  void start(String currentVersion) {
    _startTimer = Timer(_startupDelay, () => _check(currentVersion));
  }

  Future<void> _check(String currentVersion) async {
    if (_disposed) return;
    try {
      final response = await http
          .get(
            Uri.parse(_releasesUrl),
            headers: {
              'User-Agent': 'CopyPaste-UpdateChecker',
              'Accept': 'application/vnd.github.v3+json',
            },
          )
          .timeout(_httpTimeout);

      if (response.statusCode != 200) return;

      final json =
          jsonDecode(response.body) as Map<String, dynamic>;
      final tagName = json['tag_name'] as String?;
      final downloadUrl = json['html_url'] as String?;
      final prerelease = json['prerelease'] as bool? ?? false;

      if (tagName == null || downloadUrl == null) return;
      if (prerelease) return;

      final latest =
          tagName.startsWith('v') ? tagName.substring(1) : tagName;

      if (!_disposed && isNewer(latest, currentVersion)) {
        if (!isVersionDismissed(latest)) {
          _onUpdateAvailable.add(
            UpdateInfo(version: latest, downloadUrl: downloadUrl),
          );
        }
      }
    } catch (_) {}
  }

  void dismissVersion(String version) {
    if (configPath == null) return;
    try {
      final file = File(p.join(configPath!, _dismissedFileName));
      file.parent.createSync(recursive: true);
      file.writeAsStringSync(version);
    } catch (_) {}
  }

  bool isVersionDismissed(String version) {
    if (configPath == null) return false;
    try {
      final file = File(p.join(configPath!, _dismissedFileName));
      if (!file.existsSync()) return false;
      return file.readAsStringSync().trim() == version;
    } catch (_) {
      return false;
    }
  }

  static bool isNewer(String latest, String current) {
    try {
      final l = _parseVersion(latest);
      final c = _parseVersion(current);
      for (var i = 0; i < 3; i++) {
        final lv = i < l.length ? l[i] : 0;
        final cv = i < c.length ? c[i] : 0;
        if (lv > cv) return true;
        if (lv < cv) return false;
      }
      return false;
    } catch (_) {
      return false;
    }
  }

  static List<int> _parseVersion(String version) {
    // Strip pre-release suffix: "1.0.0-beta.1" → "1.0.0"
    final base = version.split('-').first;
    return base.split('.').map(int.parse).toList();
  }

  void dispose() {
    _disposed = true;
    _startTimer?.cancel();
    _onUpdateAvailable.close();
  }
}
