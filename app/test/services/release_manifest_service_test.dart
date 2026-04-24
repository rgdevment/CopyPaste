import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:copypaste/services/release_manifest_service.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ReleaseManifest.tryParse', () {
    test('parses a minimal valid manifest', () {
      final m = ReleaseManifest.tryParse('''
        {
          "schema": 1,
          "latest": "2.3.0",
          "minimumSupported": "2.3.0",
          "blockedVersions": ["2.2.6"],
          "severity": "critical",
          "channels": {
            "github_windows": { "url": "https://example.com/x" }
          },
          "releaseNotes": {
            "en": { "summary": "Hello" }
          }
        }
      ''');
      expect(m, isNotNull);
      expect(m!.latest, '2.3.0');
      expect(m.severity, ManifestSeverity.critical);
      expect(m.blockedVersions, contains('2.2.6'));
      expect(m.channels.containsKey('github_windows'), isTrue);
      expect(m.notesFor('en')?.summary, 'Hello');
    });

    test('rejects unknown schema', () {
      final m = ReleaseManifest.tryParse(
        '{"schema": 2, "latest":"1.0.0", "minimumSupported":"1.0.0"}',
      );
      expect(m, isNull);
    });

    test('rejects invalid semver', () {
      final m = ReleaseManifest.tryParse(
        '{"schema": 1, "latest":"banana", "minimumSupported":"1.0.0"}',
      );
      expect(m, isNull);
    });

    test('strips channel entries with non-https URLs', () {
      final m = ReleaseManifest.tryParse('''
        {
          "schema": 1,
          "latest": "1.0.0",
          "minimumSupported": "1.0.0",
          "channels": {
            "github_linux": { "url": "http://insecure.example/x" },
            "snap": { "command": "sudo snap refresh copypaste" }
          }
        }
      ''');
      expect(m, isNotNull);
      expect(m!.channels.containsKey('github_linux'), isFalse);
      expect(m.channels['snap']?.command, 'sudo snap refresh copypaste');
    });
  });

  group('compareVersions', () {
    test('orders patch versions', () {
      expect(
        ReleaseManifestService.compareVersions('2.3.0', '2.3.1'),
        lessThan(0),
      );
      expect(
        ReleaseManifestService.compareVersions('2.3.1', '2.3.0'),
        greaterThan(0),
      );
      expect(ReleaseManifestService.compareVersions('2.3.0', '2.3.0'), 0);
    });

    test('ranks pre-release lower than the same base', () {
      expect(
        ReleaseManifestService.compareVersions('2.3.0-rc1', '2.3.0'),
        lessThan(0),
      );
    });
  });

  group('isBlocked', () {
    ManifestState stateFor({
      required String latest,
      required String minimumSupported,
      List<String> blocked = const [],
      ManifestSeverity severity = ManifestSeverity.patch,
      bool expired = false,
    }) {
      return ManifestState(
        manifest: ReleaseManifest(
          schema: 1,
          latest: latest,
          minimumSupported: minimumSupported,
          blockedVersions: blocked,
          channels: const {},
          notes: const {},
          severity: severity,
        ),
        fetchedAt: DateTime.now().toUtc(),
        expired: expired,
      );
    }

    test('blocks when current is in blockedVersions', () {
      expect(
        ReleaseManifestService.isBlocked(
          current: '2.2.6',
          state: stateFor(
            latest: '2.3.0',
            minimumSupported: '2.3.0',
            blocked: ['2.2.6'],
          ),
        ),
        isTrue,
      );
    });

    test('blocks when current < minimumSupported and severity=critical', () {
      expect(
        ReleaseManifestService.isBlocked(
          current: '2.2.0',
          state: stateFor(
            latest: '2.3.0',
            minimumSupported: '2.3.0',
            severity: ManifestSeverity.critical,
          ),
        ),
        isTrue,
      );
    });

    test('does not block when severity is not critical', () {
      expect(
        ReleaseManifestService.isBlocked(
          current: '2.2.0',
          state: stateFor(
            latest: '2.3.0',
            minimumSupported: '2.3.0',
            severity: ManifestSeverity.major,
          ),
        ),
        isFalse,
      );
    });

    test('never blocks when the cache is expired', () {
      expect(
        ReleaseManifestService.isBlocked(
          current: '2.2.6',
          state: stateFor(
            latest: '2.3.0',
            minimumSupported: '2.3.0',
            blocked: ['2.2.6'],
            severity: ManifestSeverity.critical,
            expired: true,
          ),
        ),
        isFalse,
      );
    });

    test('returns false when there is no manifest at all', () {
      expect(
        ReleaseManifestService.isBlocked(current: '2.2.6', state: null),
        isFalse,
      );
    });
  });

  group('badgeSeverity', () {
    test('returns null when current is up to date', () {
      final s = ManifestState(
        manifest: ReleaseManifest(
          schema: 1,
          latest: '2.3.0',
          minimumSupported: '2.0.0',
          blockedVersions: const [],
          channels: const {},
          notes: const {},
          severity: ManifestSeverity.patch,
        ),
        fetchedAt: DateTime.now().toUtc(),
        expired: false,
      );
      expect(
        ReleaseManifestService.badgeSeverity(current: '2.3.0', state: s),
        isNull,
      );
    });

    test('returns severity when current is older than latest', () {
      final s = ManifestState(
        manifest: ReleaseManifest(
          schema: 1,
          latest: '2.3.0',
          minimumSupported: '2.0.0',
          blockedVersions: const [],
          channels: const {},
          notes: const {},
          severity: ManifestSeverity.minor,
        ),
        fetchedAt: DateTime.now().toUtc(),
        expired: false,
      );
      expect(
        ReleaseManifestService.badgeSeverity(current: '2.2.0', state: s),
        ManifestSeverity.minor,
      );
    });
  });

  group('ReleaseManifest.tryParse — edge cases', () {
    test('returns null for non-JSON input', () {
      expect(ReleaseManifest.tryParse('not json'), isNull);
    });

    test('returns null when root is not a Map', () {
      expect(ReleaseManifest.tryParse('["array"]'), isNull);
    });

    test('returns null when minimumSupported is invalid semver', () {
      final m = ReleaseManifest.tryParse(
        '{"schema":1,"latest":"1.0.0","minimumSupported":"bad"}',
      );
      expect(m, isNull);
    });

    test('skips blockedVersions entries that are not valid semver', () {
      final m = ReleaseManifest.tryParse('''
        {
          "schema": 1, "latest": "1.0.0", "minimumSupported": "1.0.0",
          "blockedVersions": ["bad", "1.0.1", null]
        }
      ''');
      expect(m, isNotNull);
      expect(m!.blockedVersions, ['1.0.1']);
    });

    test('skips channel entries with null info', () {
      final m = ReleaseManifest.tryParse('''
        {
          "schema": 1, "latest": "1.0.0", "minimumSupported": "1.0.0",
          "channels": { "bad": null, "ok": { "command": "brew upgrade x" } }
        }
      ''');
      expect(m, isNotNull);
      expect(m!.channels.containsKey('bad'), isFalse);
      expect(m.channels['ok']?.command, 'brew upgrade x');
    });

    test('parses all severity values', () {
      for (final pair in [
        ('patch', ManifestSeverity.patch),
        ('minor', ManifestSeverity.minor),
        ('major', ManifestSeverity.major),
        ('critical', ManifestSeverity.critical),
        ('unknown_value', ManifestSeverity.patch),
      ]) {
        final m = ReleaseManifest.tryParse(
          '{"schema":1,"latest":"1.0.0","minimumSupported":"1.0.0","severity":"${pair.$1}"}',
        );
        expect(m?.severity, pair.$2, reason: 'severity=${pair.$1}');
      }
    });

    test('ms-windows-store URL is accepted', () {
      final m = ReleaseManifest.tryParse('''
        {
          "schema": 1, "latest": "1.0.0", "minimumSupported": "1.0.0",
          "channels": {
            "msstore": { "url": "ms-windows-store://pdp/?productid=XXXXX" }
          }
        }
      ''');
      expect(m, isNotNull);
      expect(m!.channels['msstore']?.url, contains('ms-windows-store://'));
    });

    test('notesFor falls back to en when locale not present', () {
      final m = ReleaseManifest.tryParse('''
        {
          "schema": 1, "latest": "1.0.0", "minimumSupported": "1.0.0",
          "releaseNotes": { "en": { "summary": "English note" } }
        }
      ''')!;
      expect(m.notesFor('es')?.summary, 'English note');
      expect(m.notesFor('fr')?.summary, 'English note');
    });

    test('notesFor returns null when notes is empty', () {
      final m = ReleaseManifest(
        schema: 1,
        latest: '1.0.0',
        minimumSupported: '1.0.0',
        blockedVersions: const [],
        channels: const {},
        notes: const {},
        severity: ManifestSeverity.patch,
      );
      expect(m.notesFor('en'), isNull);
    });

    test('notesFor matches partial locale (es_CL -> es)', () {
      final m = ReleaseManifest.tryParse('''
        {
          "schema": 1, "latest": "1.0.0", "minimumSupported": "1.0.0",
          "releaseNotes": { "es": { "summary": "Nota en español" } }
        }
      ''')!;
      expect(m.notesFor('es_CL')?.summary, 'Nota en español');
    });

    test('releaseNotes entries without summary are skipped', () {
      final m = ReleaseManifest.tryParse('''
        {
          "schema": 1, "latest": "1.0.0", "minimumSupported": "1.0.0",
          "releaseNotes": {
            "en": { "no_summary": "oops" },
            "es": { "summary": "Hola" }
          }
        }
      ''')!;
      expect(m.notesFor('en')?.summary, 'Hola');
    });
  });

  group('compareVersions — extra cases', () {
    test('major version wins', () {
      expect(
        ReleaseManifestService.compareVersions('3.0.0', '2.9.9'),
        greaterThan(0),
      );
    });
    test('minor version wins', () {
      expect(
        ReleaseManifestService.compareVersions('2.1.0', '2.0.9'),
        greaterThan(0),
      );
    });
    test('two pre-releases compare equal', () {
      expect(
        ReleaseManifestService.compareVersions('1.0.0-rc1', '1.0.0-rc2'),
        0,
      );
    });
    test('pre-release is older than release', () {
      expect(
        ReleaseManifestService.compareVersions('1.0.0', '1.0.0-rc1'),
        greaterThan(0),
      );
    });
  });

  group('isBlocked — extra cases', () {
    ManifestState makeState({
      String latest = '2.3.0',
      String minimumSupported = '2.3.0',
      List<String> blocked = const [],
      ManifestSeverity severity = ManifestSeverity.patch,
      bool expired = false,
    }) => ManifestState(
      manifest: ReleaseManifest(
        schema: 1,
        latest: latest,
        minimumSupported: minimumSupported,
        blockedVersions: blocked,
        channels: const {},
        notes: const {},
        severity: severity,
      ),
      fetchedAt: DateTime.now().toUtc(),
      expired: expired,
    );

    test('does not block when current >= minimumSupported', () {
      expect(
        ReleaseManifestService.isBlocked(
          current: '2.3.0',
          state: makeState(severity: ManifestSeverity.critical),
        ),
        isFalse,
      );
    });

    test('does not block for major severity below minimum', () {
      expect(
        ReleaseManifestService.isBlocked(
          current: '2.2.0',
          state: makeState(severity: ManifestSeverity.major),
        ),
        isFalse,
      );
    });
  });

  group('cache read/write', () {
    late Directory tmpDir;

    setUp(() async {
      tmpDir = await Directory.systemTemp.createTemp('manifest_test_');
      await ReleaseManifestService.reset();
      ReleaseManifestService.cacheDirOverride = tmpDir.path;
    });

    tearDown(() async {
      await ReleaseManifestService.reset();
      await tmpDir.delete(recursive: true);
    });

    ReleaseManifest makeManifest() => ReleaseManifest(
      schema: 1,
      latest: '2.3.0',
      minimumSupported: '2.3.0',
      blockedVersions: const ['2.2.6'],
      channels: const {},
      notes: const {},
      severity: ManifestSeverity.critical,
    );

    test('initialize reads cached manifest and emits it on stream', () async {
      final m = makeManifest();
      final json = jsonEncode(<String, Object>{
        'schema': m.schema,
        'latest': m.latest,
        'minimumSupported': m.minimumSupported,
        'blockedVersions': m.blockedVersions,
        'channels': <String, Object>{},
        'releaseNotes': <String, Object>{},
        'severity': 'critical',
      });
      final cacheFile = File('${tmpDir.path}/release_manifest.json');
      final metaFile = File('${tmpDir.path}/release_manifest.meta');
      await cacheFile.writeAsString(json);
      await metaFile.writeAsString(
        jsonEncode({'fetchedAt': DateTime.now().toUtc().toIso8601String()}),
      );

      ReleaseManifestService.manifestUrlOverride = 'https://example.com/fail';
      ReleaseManifestService.signatureUrlOverride =
          'https://example.com/fail.sig';

      final emitted = <ManifestState?>[];
      final sub = ReleaseManifestService.stream.listen(emitted.add);

      await ReleaseManifestService.initialize(storageConfigDir: tmpDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));

      unawaited(sub.cancel());

      expect(emitted, isNotEmpty);
      expect(emitted.first?.manifest.latest, '2.3.0');
      expect(emitted.first?.expired, isFalse);
    });

    test('expired flag is set when cache is older than 15 days', () async {
      final m = makeManifest();
      final json = jsonEncode(<String, Object>{
        'schema': m.schema,
        'latest': m.latest,
        'minimumSupported': m.minimumSupported,
        'blockedVersions': m.blockedVersions,
        'channels': <String, Object>{},
        'releaseNotes': <String, Object>{},
        'severity': 'critical',
      });
      final cacheFile = File('${tmpDir.path}/release_manifest.json');
      final metaFile = File('${tmpDir.path}/release_manifest.meta');
      await cacheFile.writeAsString(json);
      final old = DateTime.now().toUtc().subtract(const Duration(days: 20));
      await metaFile.writeAsString(
        jsonEncode({'fetchedAt': old.toIso8601String()}),
      );

      ReleaseManifestService.manifestUrlOverride = 'https://example.com/fail';
      ReleaseManifestService.signatureUrlOverride =
          'https://example.com/fail.sig';

      final emitted = <ManifestState?>[];
      final sub = ReleaseManifestService.stream.listen(emitted.add);

      await ReleaseManifestService.initialize(storageConfigDir: tmpDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));

      unawaited(sub.cancel());

      expect(emitted.first?.expired, isTrue);
      expect(
        ReleaseManifestService.isBlocked(
          current: '2.2.6',
          state: emitted.first,
        ),
        isFalse,
      );
    });

    test('returns null from cache when meta fetchedAt is missing', () async {
      await File('${tmpDir.path}/release_manifest.json').writeAsString(
        '{"schema":1,"latest":"1.0.0","minimumSupported":"1.0.0"}',
      );
      await File(
        '${tmpDir.path}/release_manifest.meta',
      ).writeAsString('{"no_date": true}');

      ReleaseManifestService.manifestUrlOverride = 'https://example.com/fail';
      ReleaseManifestService.signatureUrlOverride =
          'https://example.com/fail.sig';

      final emitted = <ManifestState?>[];
      final sub = ReleaseManifestService.stream.listen(emitted.add);
      await ReleaseManifestService.initialize(storageConfigDir: tmpDir.path);
      await Future<void>.delayed(const Duration(milliseconds: 100));
      unawaited(sub.cancel());

      expect(emitted, isEmpty);
    });
  });
}
