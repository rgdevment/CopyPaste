import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:core/core.dart';
import 'package:flutter/foundation.dart' show visibleForTesting;
import 'package:path/path.dart' as p;

import 'manifest_signature.dart';

const _defaultManifestUrl =
    'https://github.com/rgdevment/CopyPaste/releases/latest/download/release-manifest.json';
const _defaultSignatureUrl =
    'https://github.com/rgdevment/CopyPaste/releases/latest/download/release-manifest.json.sig';

const _cacheFileName = 'release_manifest.json';
const _cacheMetaFileName = 'release_manifest.meta';

const _cacheMaxAge = Duration(days: 15);
const _checkInterval = Duration(hours: 24);
const _httpTimeout = Duration(seconds: 10);

/// Severity declared by the release manifest.
enum ManifestSeverity { patch, minor, major, critical }

ManifestSeverity _severityFromString(String? raw) {
  switch (raw) {
    case 'major':
      return ManifestSeverity.major;
    case 'minor':
      return ManifestSeverity.minor;
    case 'critical':
      return ManifestSeverity.critical;
    case 'patch':
    default:
      return ManifestSeverity.patch;
  }
}

class ChannelInfo {
  const ChannelInfo({this.url, this.command});
  final String? url;
  final String? command;

  static ChannelInfo? fromJson(Object? raw) {
    if (raw is! Map) return null;
    final url = raw['url'];
    final command = raw['command'];
    return ChannelInfo(
      url: url is String ? url : null,
      command: command is String ? command : null,
    );
  }
}

class ReleaseNotes {
  const ReleaseNotes({required this.summary, this.url});
  final String summary;
  final String? url;
}

class ReleaseManifest {
  ReleaseManifest({
    required this.schema,
    required this.latest,
    required this.minimumSupported,
    required this.blockedVersions,
    required this.channels,
    required this.notes,
    required this.severity,
  });

  final int schema;
  final String latest;
  final String minimumSupported;
  final List<String> blockedVersions;
  final Map<String, ChannelInfo> channels;
  final Map<String, ReleaseNotes> notes;
  final ManifestSeverity severity;

  ReleaseNotes? notesFor(String locale) {
    if (notes.isEmpty) return null;
    final key = locale.toLowerCase();
    return notes[key] ??
        notes[key.split('_').first] ??
        notes['en'] ??
        notes.values.first;
  }

  static ReleaseManifest? tryParse(String body) {
    Object? decoded;
    try {
      decoded = jsonDecode(body);
    } catch (_) {
      return null;
    }
    if (decoded is! Map) return null;

    final schema = decoded['schema'];
    if (schema is! int || schema != 1) return null;

    final latest = decoded['latest'];
    if (latest is! String || !_isValidSemver(latest)) return null;

    final minimumSupported = decoded['minimumSupported'];
    if (minimumSupported is! String || !_isValidSemver(minimumSupported)) {
      return null;
    }

    final blockedRaw = decoded['blockedVersions'];
    final blocked = <String>[];
    if (blockedRaw is List) {
      for (final v in blockedRaw) {
        if (v is String && _isValidSemver(v)) blocked.add(v);
      }
    }

    final channelsRaw = decoded['channels'];
    final channels = <String, ChannelInfo>{};
    if (channelsRaw is Map) {
      channelsRaw.forEach((k, v) {
        if (k is! String) return;
        final info = ChannelInfo.fromJson(v);
        if (info == null) return;
        if (info.url != null && !_isValidUrl(info.url!)) return;
        channels[k] = info;
      });
    }

    final notesRaw = decoded['releaseNotes'];
    final notes = <String, ReleaseNotes>{};
    if (notesRaw is Map) {
      notesRaw.forEach((k, v) {
        if (k is! String || v is! Map) return;
        final summary = v['summary'];
        if (summary is! String) return;
        final url = v['url'];
        notes[k.toLowerCase()] = ReleaseNotes(
          summary: summary,
          url: url is String && _isValidUrl(url) ? url : null,
        );
      });
    }

    return ReleaseManifest(
      schema: schema,
      latest: latest,
      minimumSupported: minimumSupported,
      blockedVersions: blocked,
      channels: channels,
      notes: notes,
      severity: _severityFromString(decoded['severity'] as String?),
    );
  }

  static bool _isValidSemver(String v) {
    final parts = v.split('-').first.split('.');
    if (parts.length != 3) return false;
    return parts.every((p) => int.tryParse(p) != null);
  }

  static bool _isValidUrl(String u) {
    return u.startsWith('https://') || u.startsWith('ms-windows-store://');
  }
}

/// Outcome reported back to the UI.
class ManifestState {
  ManifestState({
    required this.manifest,
    required this.fetchedAt,
    required this.expired,
  });

  final ReleaseManifest manifest;
  final DateTime fetchedAt;
  final bool expired;
}

class ReleaseManifestService {
  ReleaseManifestService._();

  @visibleForTesting
  static String? cacheDirOverride;

  @visibleForTesting
  static String manifestUrlOverride = '';

  @visibleForTesting
  static String signatureUrlOverride = '';

  @visibleForTesting
  static HttpClient Function()? httpClientFactory;

  static Timer? _timer;
  static ManifestState? _current;

  static final StreamController<ManifestState?> _controller =
      StreamController<ManifestState?>.broadcast();
  static Stream<ManifestState?> get stream => _controller.stream;

  static ManifestState? get current => _current;

  static String get _effectiveManifestUrl => manifestUrlOverride.isNotEmpty
      ? manifestUrlOverride
      : _defaultManifestUrl;

  static String get _effectiveSignatureUrl => signatureUrlOverride.isNotEmpty
      ? signatureUrlOverride
      : _defaultSignatureUrl;

  static Future<void> initialize({required String storageConfigDir}) async {
    cacheDirOverride ??= storageConfigDir;
    final cached = await _readCache();
    if (cached != null) {
      _current = cached;
      _controller.add(cached);
    }
    unawaited(_refresh());
    _timer = Timer.periodic(_checkInterval, (_) => _refresh());
  }

  static Future<void> _refresh() async {
    final fresh = await _fetchAndVerify();
    if (fresh == null) {
      final cached = await _readCache();
      if (cached != null && cached.expired != _current?.expired) {
        _current = cached;
        _controller.add(cached);
      }
      return;
    }
    await _writeCache(fresh);
    _current = ManifestState(
      manifest: fresh,
      fetchedAt: DateTime.now().toUtc(),
      expired: false,
    );
    _controller.add(_current);
  }

  static Future<ReleaseManifest?> _fetchAndVerify() async {
    final client = (httpClientFactory ?? HttpClient.new)()
      ..connectionTimeout = _httpTimeout;
    try {
      final manifestBytes = await _fetchBytes(client, _effectiveManifestUrl);
      if (manifestBytes == null) return null;
      final sigBody = await _fetchString(client, _effectiveSignatureUrl);
      if (sigBody == null) return null;

      final ok = await ManifestSignature.verify(manifestBytes, sigBody);
      if (!ok) return null;

      final body = utf8.decode(manifestBytes, allowMalformed: false);
      return ReleaseManifest.tryParse(body);
    } catch (_) {
      return null;
    } finally {
      client.close();
    }
  }

  static Future<List<int>?> _fetchBytes(HttpClient client, String url) async {
    try {
      final req = await client.getUrl(Uri.parse(url));
      req.headers.set('User-Agent', 'CopyPaste-ReleaseManifest');
      final res = await req.close();
      if (res.statusCode != 200) {
        await res.drain<void>();
        return null;
      }
      final builder = BytesBuilder(copy: false);
      await for (final chunk in res) {
        builder.add(chunk);
      }
      return builder.takeBytes();
    } catch (_) {
      return null;
    }
  }

  static Future<String?> _fetchString(HttpClient client, String url) async {
    final bytes = await _fetchBytes(client, url);
    if (bytes == null) return null;
    try {
      return utf8.decode(bytes);
    } catch (_) {
      return null;
    }
  }

  static String? get _cacheDir => cacheDirOverride;

  static Future<ManifestState?> _readCache() async {
    final dir = _cacheDir;
    if (dir == null) return null;
    final manifestFile = File(p.join(dir, _cacheFileName));
    final metaFile = File(p.join(dir, _cacheMetaFileName));
    if (!manifestFile.existsSync() || !metaFile.existsSync()) return null;
    try {
      final body = await manifestFile.readAsString();
      final manifest = ReleaseManifest.tryParse(body);
      if (manifest == null) return null;
      final metaRaw = jsonDecode(await metaFile.readAsString());
      if (metaRaw is! Map || metaRaw['fetchedAt'] is! String) return null;
      final fetchedAt = DateTime.tryParse(metaRaw['fetchedAt'] as String);
      if (fetchedAt == null) return null;
      final age = DateTime.now().toUtc().difference(fetchedAt);
      return ManifestState(
        manifest: manifest,
        fetchedAt: fetchedAt,
        expired: age > _cacheMaxAge,
      );
    } catch (e) {
      AppLogger.warn('ReleaseManifest cache read failed: $e');
      return null;
    }
  }

  static Future<void> _writeCache(ReleaseManifest manifest) async {
    final dir = _cacheDir;
    if (dir == null) return;
    try {
      await Directory(dir).create(recursive: true);
      final json = jsonEncode({
        'schema': manifest.schema,
        'latest': manifest.latest,
        'minimumSupported': manifest.minimumSupported,
        'blockedVersions': manifest.blockedVersions,
        'channels': manifest.channels.map(
          (k, v) => MapEntry(k, {
            if (v.url != null) 'url': v.url,
            if (v.command != null) 'command': v.command,
          }),
        ),
        'releaseNotes': manifest.notes.map(
          (k, v) => MapEntry(k, {
            'summary': v.summary,
            if (v.url != null) 'url': v.url,
          }),
        ),
        'severity': switch (manifest.severity) {
          ManifestSeverity.critical => 'critical',
          ManifestSeverity.major => 'major',
          ManifestSeverity.minor => 'minor',
          ManifestSeverity.patch => 'patch',
        },
      });
      await File(p.join(dir, _cacheFileName)).writeAsString(json);
      await File(p.join(dir, _cacheMetaFileName)).writeAsString(
        jsonEncode({'fetchedAt': DateTime.now().toUtc().toIso8601String()}),
      );
    } catch (e) {
      AppLogger.warn('ReleaseManifest cache write failed: $e');
    }
  }

  static int compareVersions(String a, String b) {
    final aParts = a.split('-');
    final bParts = b.split('-');
    final aBase = aParts[0].split('.').map(int.tryParse).toList();
    final bBase = bParts[0].split('.').map(int.tryParse).toList();
    for (var i = 0; i < 3; i++) {
      final av = i < aBase.length ? (aBase[i] ?? 0) : 0;
      final bv = i < bBase.length ? (bBase[i] ?? 0) : 0;
      if (av != bv) return av - bv;
    }
    final aPre = aParts.length > 1;
    final bPre = bParts.length > 1;
    if (aPre && !bPre) return -1;
    if (!aPre && bPre) return 1;
    return 0;
  }

  static bool isBlocked({
    required String current,
    required ManifestState? state,
  }) {
    if (state == null || state.expired) return false;
    final m = state.manifest;
    if (m.blockedVersions.contains(current)) return true;
    if (m.severity == ManifestSeverity.critical &&
        compareVersions(current, m.minimumSupported) < 0) {
      return true;
    }
    return false;
  }

  static ManifestSeverity? badgeSeverity({
    required String current,
    required ManifestState? state,
  }) {
    if (state == null) return null;
    final m = state.manifest;
    final cmp = compareVersions(current, m.latest);
    if (cmp >= 0) return null;
    return m.severity;
  }

  static void dispose() {
    _timer?.cancel();
    _timer = null;
  }

  @visibleForTesting
  static Future<void> reset() async {
    dispose();
    _current = null;
    cacheDirOverride = null;
    manifestUrlOverride = '';
    signatureUrlOverride = '';
    httpClientFactory = null;
  }

  @visibleForTesting
  static void setStateForTest(ManifestState? state) {
    _current = state;
    _controller.add(state);
  }
}
