import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/l10n/app_localizations.dart';
import 'package:copypaste/screens/windows_onboarding_screen.dart';
import 'package:copypaste/theme/compact_theme.dart';
import 'package:copypaste/theme/theme_provider.dart';

Widget _wrap(Widget child, {Locale locale = const Locale('en')}) {
  return MaterialApp(
    locale: locale,
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    home: CopyPasteTheme(themeData: CompactTheme(), child: child),
  );
}

Future<void> _pump(
  WidgetTester tester,
  Widget child, {
  Locale locale = const Locale('en'),
}) async {
  tester.view.physicalSize = const Size(1080, 1920);
  tester.view.devicePixelRatio = 2.0;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(_wrap(child, locale: locale));
  await tester.pump();
}

void main() {
  const hotkey = 'Ctrl+Shift+V';

  Widget screen({VoidCallback? onDismiss, VoidCallback? onSettings}) =>
      WindowsOnboardingScreen(
        hotkey: hotkey,
        initialConfig: const AppConfig(),
        onDismiss: (_) => (onDismiss ?? () {})(),
        onSettings: (_) => (onSettings ?? () {})(),
      );

  group('WindowsOnboardingScreen', () {
    testWidgets('renders title and subtitle', (tester) async {
      await _pump(tester, screen());

      expect(find.text('Welcome to CopyPaste'), findsOneWidget);
      expect(find.text('Everything you copy, saved.'), findsOneWidget);
    });

    testWidgets('renders privacy badge with lock icon', (tester) async {
      await _pump(tester, screen());

      expect(find.byIcon(Icons.lock_outline_rounded), findsOneWidget);
      expect(find.text('No cloud · No tracking · 100% local'), findsOneWidget);
    });

    testWidgets('renders hotkey chip with keyboard icon', (tester) async {
      await _pump(tester, screen());

      expect(find.byIcon(Icons.keyboard_rounded), findsOneWidget);
      expect(find.text(hotkey), findsOneWidget);
    });

    testWidgets('renders tray hint text', (tester) async {
      await _pump(tester, screen());

      expect(
        find.text('Look for the CP icon next to your clock.'),
        findsOneWidget,
      );
    });

    testWidgets('renders description containing the hotkey', (tester) async {
      await _pump(tester, screen());

      expect(find.textContaining(hotkey), findsWidgets);
    });

    testWidgets('tapping dismiss button invokes onDismiss', (tester) async {
      var dismissed = false;
      await _pump(tester, screen(onDismiss: () => dismissed = true));

      await tester.tap(find.byType(FilledButton));
      await tester.pump();

      expect(dismissed, isTrue);
    });

    testWidgets('tapping settings button invokes onSettings', (tester) async {
      var opened = false;
      await _pump(tester, screen(onSettings: () => opened = true));

      await tester.tap(find.byType(OutlinedButton));
      await tester.pump();

      expect(opened, isTrue);
    });

    testWidgets('renders both action buttons', (tester) async {
      await _pump(tester, screen());

      expect(find.byType(FilledButton), findsOneWidget);
      expect(find.byType(OutlinedButton), findsOneWidget);
    });

    testWidgets('renders in dark mode without errors', (tester) async {
      tester.view.physicalSize = const Size(1080, 1920);
      tester.view.devicePixelRatio = 2.0;
      addTearDown(tester.view.reset);

      await tester.pumpWidget(
        MaterialApp(
          locale: const Locale('en'),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          theme: ThemeData(brightness: Brightness.dark),
          home: CopyPasteTheme(themeData: CompactTheme(), child: screen()),
        ),
      );
      await tester.pump();

      expect(find.text('Welcome to CopyPaste'), findsOneWidget);
    });

    testWidgets('renders in Spanish locale', (tester) async {
      await _pump(tester, screen(), locale: const Locale('es'));

      expect(find.text('Bienvenido a CopyPaste'), findsOneWidget);
      expect(find.text('Sin nube · Sin rastreo · 100% local'), findsOneWidget);
    });

    testWidgets('app icon is displayed', (tester) async {
      await _pump(tester, screen());

      expect(find.byType(Image), findsOneWidget);
    });

    testWidgets('uses different hotkey string when provided', (tester) async {
      const customHotkey = 'Ctrl+Alt+V';
      tester.view.physicalSize = const Size(1080, 1920);
      tester.view.devicePixelRatio = 2.0;
      addTearDown(tester.view.reset);

      await tester.pumpWidget(
        _wrap(
          WindowsOnboardingScreen(
            hotkey: customHotkey,
            initialConfig: const AppConfig(),
            onDismiss: (_) {},
            onSettings: (_) {},
          ),
        ),
      );
      await tester.pump();

      expect(find.text(customHotkey), findsOneWidget);
    });
  });
}
