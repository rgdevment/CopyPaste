import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/helpers/url_helper.dart';
import 'package:copypaste/l10n/app_localizations.dart';
import 'package:copypaste/screens/blocked_version_screen.dart';
import 'package:copypaste/services/install_channel.dart';
import 'package:copypaste/services/release_manifest_service.dart';
import 'package:copypaste/theme/compact_theme.dart';
import 'package:copypaste/theme/theme_provider.dart';

Widget _wrap(Widget child) => MaterialApp(
  locale: const Locale('en'),
  localizationsDelegates: AppLocalizations.localizationsDelegates,
  supportedLocales: AppLocalizations.supportedLocales,
  theme: ThemeData.light(),
  home: CopyPasteTheme(themeData: CompactTheme(), child: child),
);

ReleaseManifest _manifest({
  String? githubWindowsUrl,
  String? homebrewCommand,
  String? msStoreUrl,
  String? snapCommand,
}) {
  return ReleaseManifest(
    schema: 1,
    latest: '2.3.0',
    minimumSupported: '2.3.0',
    blockedVersions: const ['2.2.6'],
    severity: ManifestSeverity.critical,
    channels: {
      if (githubWindowsUrl != null)
        'github_windows': ChannelInfo(url: githubWindowsUrl),
      if (homebrewCommand != null)
        'homebrew': ChannelInfo(command: homebrewCommand),
      if (msStoreUrl != null) 'msstore': ChannelInfo(url: msStoreUrl),
      if (snapCommand != null) 'snap': ChannelInfo(command: snapCommand),
      if (githubWindowsUrl != null)
        'github_linux': ChannelInfo(url: githubWindowsUrl),
      if (githubWindowsUrl != null)
        'github_macos': ChannelInfo(url: githubWindowsUrl),
    },
    notes: const {'en': ReleaseNotes(summary: 'Critical security fix.')},
  );
}

void main() {
  tearDown(() {
    UrlHelper.platformOverride = null;
    InstallChannelDetector.platformOverride = null;
    InstallChannelDetector.channelOverride = null;
  });

  group('BlockedVersionScreen', () {
    testWidgets('renders title and current version', (tester) async {
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: _manifest(githubWindowsUrl: 'https://example.com'),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('2.2.6'), findsWidgets);
      expect(find.textContaining('2.3.0'), findsWidgets);
    });

    testWidgets('shows release notes summary', (tester) async {
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: _manifest(githubWindowsUrl: 'https://example.com'),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('Critical security fix'), findsOneWidget);
    });

    testWidgets('shows Download button for github_windows channel', (
      tester,
    ) async {
      InstallChannelDetector.platformOverride = HostPlatform.windows;
      UrlHelper.platformOverride = 'other';
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: _manifest(githubWindowsUrl: 'https://example.com/latest'),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l.updateActionDownload), findsOneWidget);
    });

    testWidgets('shows Copy command button for homebrew channel', (
      tester,
    ) async {
      InstallChannelDetector.channelOverride = InstallChannel.homebrew;
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: _manifest(homebrewCommand: 'brew upgrade copypaste'),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l.updateActionCopyBrew), findsOneWidget);
    });

    testWidgets('shows Copy command button for snap channel', (tester) async {
      InstallChannelDetector.channelOverride = InstallChannel.snap;
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: _manifest(snapCommand: 'sudo snap refresh copypaste'),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l.updateActionCopyBrew), findsOneWidget);
    });

    testWidgets('shows fallback hint when channel has no info', (tester) async {
      InstallChannelDetector.platformOverride = HostPlatform.windows;
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: ReleaseManifest(
              schema: 1,
              latest: '2.3.0',
              minimumSupported: '2.3.0',
              blockedVersions: const [],
              channels: const {},
              notes: const {},
              severity: ManifestSeverity.critical,
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l.blockedFallbackHint), findsOneWidget);
    });

    testWidgets('shows generic reason when notes is empty', (tester) async {
      InstallChannelDetector.platformOverride = HostPlatform.windows;
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: ReleaseManifest(
              schema: 1,
              latest: '2.3.0',
              minimumSupported: '2.3.0',
              blockedVersions: const [],
              channels: const {
                'github_windows': ChannelInfo(url: 'https://example.com'),
              },
              notes: const {},
              severity: ManifestSeverity.critical,
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l.blockedReasonGeneric), findsOneWidget);
    });

    testWidgets('Quit button is visible', (tester) async {
      InstallChannelDetector.platformOverride = HostPlatform.windows;
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: _manifest(githubWindowsUrl: 'https://example.com'),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));
      expect(find.text(l.blockedQuit), findsOneWidget);
    });

    testWidgets('tapping Download button invokes UrlHelper', (tester) async {
      InstallChannelDetector.platformOverride = HostPlatform.windows;
      UrlHelper.platformOverride = 'other';
      await tester.pumpWidget(
        _wrap(
          BlockedVersionScreen(
            currentVersion: '2.2.6',
            manifest: _manifest(githubWindowsUrl: 'https://example.com/latest'),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final l = await AppLocalizations.delegate.load(const Locale('en'));
      await tester.tap(find.text(l.updateActionDownload));
      await tester.pumpAndSettle();
    });
  });
}
