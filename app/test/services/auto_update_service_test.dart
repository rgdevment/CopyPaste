import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/services/auto_update_service.dart';

// ---------------------------------------------------------------------------
// Minimal HTTP server helper
// ---------------------------------------------------------------------------

Future<HttpServer> _startServer(
  String responseBody, {
  int statusCode = 200,
}) async {
  final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
  server.listen((req) async {
    req.response
      ..statusCode = statusCode
      ..headers.contentType = ContentType.json
      ..write(responseBody);
    await req.response.close();
  });
  return server;
}

void main() {
  tearDown(() => AutoUpdateService.reset());

  // -------------------------------------------------------------------------
  // isNewerVersion — semver comparison logic
  // -------------------------------------------------------------------------
  group('AutoUpdateService.isNewerVersion', () {
    test('newer patch returns true', () {
      expect(AutoUpdateService.isNewerVersion('2.0.1', '2.0.0'), isTrue);
    });

    test('newer minor returns true', () {
      expect(AutoUpdateService.isNewerVersion('2.1.0', '2.0.9'), isTrue);
    });

    test('newer major returns true', () {
      expect(AutoUpdateService.isNewerVersion('3.0.0', '2.9.9'), isTrue);
    });

    test('same version returns false', () {
      expect(AutoUpdateService.isNewerVersion('2.0.0', '2.0.0'), isFalse);
    });

    test('older version returns false', () {
      expect(AutoUpdateService.isNewerVersion('2.0.0', '2.0.1'), isFalse);
    });

    test('older minor returns false', () {
      expect(AutoUpdateService.isNewerVersion('2.0.5', '2.1.0'), isFalse);
    });

    test('release is newer than pre-release with same base', () {
      // current = 2.0.0-beta.1, latest = 2.0.0 (stable release)
      expect(AutoUpdateService.isNewerVersion('2.0.0', '2.0.0-beta.1'), isTrue);
    });

    test('same pre-release is not newer', () {
      expect(
        AutoUpdateService.isNewerVersion('2.0.0-beta.1', '2.0.0-beta.1'),
        isFalse,
      );
    });

    test('newer version even without all three segments', () {
      // "2.1" treated as 2.1.0 vs 2.0.0
      expect(AutoUpdateService.isNewerVersion('2.1', '2.0.0'), isTrue);
    });

    test('tag with "v" prefix handled by caller stripping — raw comparison', () {
      // Service strips "v" before calling isNewerVersion; verify raw also works
      expect(AutoUpdateService.isNewerVersion('2.5.0', '2.4.99'), isTrue);
    });

    test('pre-release is NOT newer than the same base release', () {
      // 2.0.0-beta.1 vs 2.0.0 → stable wins
      expect(
        AutoUpdateService.isNewerVersion('2.0.0-beta.1', '2.0.0'),
        isFalse,
      );
    });

    test('same pre-release versions are equal — not newer', () {
      expect(
        AutoUpdateService.isNewerVersion('2.0.0-alpha', '2.0.0-alpha'),
        isFalse,
      );
    });

    test('newer major always wins regardless of minor/patch', () {
      expect(AutoUpdateService.isNewerVersion('3.0.0', '2.99.99'), isTrue);
      expect(AutoUpdateService.isNewerVersion('2.99.99', '3.0.0'), isFalse);
    });

    test('segments missing from latest default to 0', () {
      // '2.1' is treated as 2.1.0 — still newer than 2.0.9
      expect(AutoUpdateService.isNewerVersion('2.1', '2.0.9'), isTrue);
    });

    test('segments missing from current default to 0', () {
      // current '2.1' treated as 2.1.0; latest '2.0.9' is older
      expect(AutoUpdateService.isNewerVersion('2.0.9', '2.1'), isFalse);
    });

    test('identical three-part versions return false', () {
      expect(AutoUpdateService.isNewerVersion('2.2.2', '2.2.2'), isFalse);
    });
  });

  // -------------------------------------------------------------------------
  // initialize() — skips when STORE_BUILD is set (compile-time constant,
  // cannot change at runtime; we test the no-op on non-macOS/linux/windows).
  // -------------------------------------------------------------------------
  group('AutoUpdateService.initialize', () {
    test('does not throw on current platform', () async {
      // We override the URL so no real network is hit.
      // On Linux/macOS we serve a local response; on Windows WinSparkle is
      // mocked via the auto_updater plugin which throws gracefully in tests.
      if (Platform.isMacOS || Platform.isLinux) {
        final payload = jsonEncode({'tag_name': 'v2.0.0'});
        final server = await _startServer(payload);
        AutoUpdateService.releasesUrlOverride =
            'http://127.0.0.1:${server.port}';
        try {
          await expectLater(AutoUpdateService.initialize(), completes);
        } finally {
          AutoUpdateService.dispose();
          await server.close(force: true);
        }
      } else {
        // Windows: auto_updater requires Flutter platform channels which are
        // unavailable in unit tests. Skip rather than fail noisily.
      }
    });
  });

  // -------------------------------------------------------------------------
  // _checkGitHubRelease via releasesUrlOverride (macOS/Linux code path)
  // -------------------------------------------------------------------------
  group('AutoUpdateService._checkGitHubRelease (via initialize)', () {
    setUp(() => AutoUpdateService.reset());

    test('fires onUpdateAvailable when newer version returned', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final payload = jsonEncode({'tag_name': 'v2.99.0'});
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      String? notified;
      AutoUpdateService.onUpdateAvailable = (v) => notified = v;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, equals('2.99.0'));
    });

    test('does NOT fire onUpdateAvailable for same version', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      // AppConfig.appVersion defaults to '2.0.0' in tests.
      final payload = jsonEncode({'tag_name': 'v2.0.0'});
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      bool notified = false;
      AutoUpdateService.onUpdateAvailable = (_) => notified = true;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, isFalse);
    });

    test('does NOT fire onUpdateAvailable for older version', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final payload = jsonEncode({'tag_name': 'v1.9.9'});
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      bool notified = false;
      AutoUpdateService.onUpdateAvailable = (_) => notified = true;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, isFalse);
    });

    test('silently ignores non-v2 releases', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final payload = jsonEncode({'tag_name': 'v3.0.0'});
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      bool notified = false;
      AutoUpdateService.onUpdateAvailable = (_) => notified = true;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, isFalse);
    });

    test('silently ignores non-200 responses', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final server = await _startServer('{}', statusCode: 404);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      bool notified = false;
      AutoUpdateService.onUpdateAvailable = (_) => notified = true;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, isFalse);
    });

    test('silently ignores missing tag_name in JSON', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final payload = jsonEncode({'name': '2.5.0'}); // no tag_name key
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      bool notified = false;
      AutoUpdateService.onUpdateAvailable = (_) => notified = true;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, isFalse);
    });

    test('silently ignores SocketException (network unavailable)', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      // Port 1 is almost certainly refused — triggers SocketException.
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:1';

      await expectLater(AutoUpdateService.initialize(), completes);
      AutoUpdateService.dispose();
    });

    test('strips leading "v" from tag_name before comparing', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final payload = jsonEncode({'tag_name': 'v2.99.0'});
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      String? notified;
      AutoUpdateService.onUpdateAvailable = (v) => notified = v;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      // Must have the "v" stripped
      expect(notified, isNotNull);
      expect(notified!.startsWith('v'), isFalse);
    });

    test('silently ignores non-JSON response body', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final server = await _startServer('not json at all');
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      bool notified = false;
      AutoUpdateService.onUpdateAvailable = (_) => notified = true;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, isFalse);
    });

    test(
      'silently ignores JSON array response (decoded is not a Map)',
      () async {
        if (!Platform.isMacOS && !Platform.isLinux) return;

        // JSON array instead of map — must not crash or notify
        final payload = jsonEncode([
          {'tag_name': 'v2.99.0'},
        ]);
        final server = await _startServer(payload);
        AutoUpdateService.releasesUrlOverride =
            'http://127.0.0.1:${server.port}';

        bool notified = false;
        AutoUpdateService.onUpdateAvailable = (_) => notified = true;

        await AutoUpdateService.initialize();
        AutoUpdateService.dispose();
        await server.close(force: true);

        expect(notified, isFalse);
      },
    );

    test('silently ignores v1.x releases (non-v2 guard)', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final payload = jsonEncode({'tag_name': 'v1.99.9'});
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      bool notified = false;
      AutoUpdateService.onUpdateAvailable = (_) => notified = true;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, isFalse);
    });

    test('fires when patch is one ahead of current', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      // AppConfig.appVersion defaults to '2.0.0'; one patch ahead = 2.0.1
      final payload = jsonEncode({'tag_name': 'v2.0.1'});
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      String? notified;
      AutoUpdateService.onUpdateAvailable = (v) => notified = v;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, equals('2.0.1'));
    });

    test('onUpdateAvailable receives version without leading v', () async {
      if (!Platform.isMacOS && !Platform.isLinux) return;

      final payload = jsonEncode({'tag_name': 'v2.5.3'});
      final server = await _startServer(payload);
      AutoUpdateService.releasesUrlOverride = 'http://127.0.0.1:${server.port}';

      String? notified;
      AutoUpdateService.onUpdateAvailable = (v) => notified = v;

      await AutoUpdateService.initialize();
      AutoUpdateService.dispose();
      await server.close(force: true);

      expect(notified, equals('2.5.3'));
    });
  });

  // -------------------------------------------------------------------------
  // dispose / reset
  // -------------------------------------------------------------------------
  group('AutoUpdateService.dispose', () {
    test('dispose cancels timer without throwing', () {
      // Create a periodic timer by calling initialize on Linux/macOS is async
      // so we just verify dispose() is idempotent.
      AutoUpdateService.dispose();
      AutoUpdateService.dispose(); // double-dispose must be safe
    });

    test('reset clears onUpdateAvailable and override', () {
      AutoUpdateService.onUpdateAvailable = (_) {};
      AutoUpdateService.releasesUrlOverride = 'http://example.com';
      AutoUpdateService.reset();

      expect(AutoUpdateService.onUpdateAvailable, isNull);
      expect(AutoUpdateService.releasesUrlOverride, isEmpty);
    });
  });
}
