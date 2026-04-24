import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/l10n/app_localizations.dart';
import 'package:copypaste/screens/linux_capabilities_banner.dart';
import 'package:copypaste/services/linux_capabilities.dart';
import 'package:copypaste/shell/linux_session.dart';
import 'package:copypaste/theme/compact_theme.dart';
import 'package:copypaste/theme/theme_provider.dart';
import 'package:core/core.dart';

LinuxCapabilities _caps({bool hasAppIndicator = true, bool hasXTest = true}) {
  return LinuxCapabilities(
    session: const LinuxSessionInfo(
      sessionType: 'x11',
      hasDisplay: true,
      hasWaylandDisplay: false,
      hasWaylandSocket: false,
      desktopEnv: 'gnome',
      wmName: 'mutter',
    ),
    isX11: true,
    hasXTest: hasXTest,
    hasAppIndicator: hasAppIndicator,
    hasEwmh: true,
    detectedDesktopEnv: 'gnome',
    detectedWmName: 'mutter',
    detectionTimedOut: false,
  );
}

Widget _wrap(Widget child) {
  return MaterialApp(
    locale: const Locale('en'),
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    home: CopyPasteTheme(
      themeData: CompactTheme(),
      child: Scaffold(body: child),
    ),
  );
}

void main() {
  group('LinuxCapabilitiesBanner', () {
    testWidgets('renders nothing when not on Linux', (tester) async {
      if (Platform.isLinux) return;
      await tester.pumpWidget(
        _wrap(
          LinuxCapabilitiesBanner(
            config: const AppConfig(),
            capabilities: _caps(hasAppIndicator: false),
            onDismiss: (_) async {},
          ),
        ),
      );
      expect(find.byType(Icon), findsNothing);
    });

    testWidgets('renders AppIndicator banner when missing and not dismissed', (
      tester,
    ) async {
      if (!Platform.isLinux) return;
      await tester.pumpWidget(
        _wrap(
          LinuxCapabilitiesBanner(
            config: const AppConfig(),
            capabilities: _caps(hasAppIndicator: false),
            onDismiss: (_) async {},
          ),
        ),
      );
      expect(find.byIcon(Icons.warning_amber_rounded), findsOneWidget);
      expect(find.byIcon(Icons.close_rounded), findsOneWidget);
    });

    testWidgets(
      'renders nothing when capability missing but already dismissed',
      (tester) async {
        if (!Platform.isLinux) return;
        await tester.pumpWidget(
          _wrap(
            LinuxCapabilitiesBanner(
              config: const AppConfig(
                linuxAppindicatorWarningDismissed: true,
                linuxXtestWarningDismissed: true,
              ),
              capabilities: _caps(hasAppIndicator: false, hasXTest: false),
              onDismiss: (_) async {},
            ),
          ),
        );
        expect(find.byIcon(Icons.warning_amber_rounded), findsNothing);
      },
    );

    testWidgets('dismiss callback fires when close icon tapped', (
      tester,
    ) async {
      if (!Platform.isLinux) return;
      AppConfig? captured;
      await tester.pumpWidget(
        _wrap(
          LinuxCapabilitiesBanner(
            config: const AppConfig(),
            capabilities: _caps(hasAppIndicator: false),
            onDismiss: (update) async {
              captured = update(const AppConfig());
            },
          ),
        ),
      );
      await tester.tap(find.byIcon(Icons.close_rounded));
      await tester.pump();
      expect(captured?.linuxAppindicatorWarningDismissed, isTrue);
    });
  });
}
