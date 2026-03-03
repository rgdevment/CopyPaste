import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;

class UpdateInfo {
  const UpdateInfo({
    required this.version,
    required this.downloadUrl,
  });

  final String version;
  final String downloadUrl;
}

class UpdateChecker {
  UpdateChecker();

  static const String _releasesUrl =
      'https://api.github.com/repos/rgdevment/CopyPaste/releases/latest';
  static const Duration _startupDelay = Duration(minutes: 1);
  static const Duration _httpTimeout = Duration(seconds: 15);

  final _onUpdateAvailable =
      StreamController<UpdateInfo>.broadcast();

  Stream<UpdateInfo> get onUpdateAvailable =>
      _onUpdateAvailable.stream;

  bool _disposed = false;

  void start(String currentVersion) {
    Future.delayed(_startupDelay, () => _check(currentVersion));
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

      if (tagName == null || downloadUrl == null) return;

      final latest =
          tagName.startsWith('v') ? tagName.substring(1) : tagName;

      if (!_disposed && isNewer(latest, currentVersion)) {
        _onUpdateAvailable.add(
          UpdateInfo(version: latest, downloadUrl: downloadUrl),
        );
      }
    } catch (_) {}
  }

  static bool isNewer(String latest, String current) {
    try {
      final l = latest.split('.').map(int.parse).toList();
      final c = current.split('.').map(int.parse).toList();
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

  void dispose() {
    _disposed = true;
    _onUpdateAvailable.close();
  }
}
