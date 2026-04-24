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
}
