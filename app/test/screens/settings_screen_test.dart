import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/l10n/app_localizations.dart';
import 'package:copypaste/screens/settings_screen.dart';
import 'package:copypaste/theme/compact_theme.dart';
import 'package:copypaste/theme/theme_provider.dart';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

late StorageConfig _storage;
late ClipboardService _service;

Widget _wrap(Widget child) => MaterialApp(
  locale: const Locale('en'),
  localizationsDelegates: AppLocalizations.localizationsDelegates,
  supportedLocales: AppLocalizations.supportedLocales,
  home: CopyPasteTheme(themeData: CompactTheme(), child: child),
);

Future<void> _pump(WidgetTester tester, Widget child) async {
  // Use a landscape desktop-ish size so the sidebar + content area both fit.
  tester.view.physicalSize = const Size(1280, 800);
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(_wrap(child));
  await tester.pump();
}

Widget _screen() => SettingsScreen(
  config: const AppConfig(),
  configPath: Directory.systemTemp.path,
  clipboardService: _service,
  storage: _storage,
  onSave: (config, changed) async {},
  onSoftReset: () async {},
  onHardReset: () async {},
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

void main() {
  setUpAll(() async {
    _storage = await StorageConfig.create(baseDir: Directory.systemTemp.path);
  });

  setUp(() {
    _service = ClipboardService(SqliteRepository.inMemory());
  });

  tearDown(() async {
    await _service.dispose();
  });

  group('SettingsScreen – smoke', () {
    testWidgets('renders sidebar title and all navigation items', (
      tester,
    ) async {
      await _pump(tester, _screen());

      expect(find.text('Settings'), findsOneWidget);
      expect(find.text('General'), findsOneWidget);
      expect(find.text('Shortcuts'), findsOneWidget);
      expect(find.text('Performance'), findsOneWidget);
      expect(find.text('Cleanup & Privacy'), findsOneWidget);
      expect(find.text('Backup & Support'), findsOneWidget);
      expect(find.text('About'), findsOneWidget);
    });

    testWidgets('General tab (default) renders without crashing', (
      tester,
    ) async {
      await _pump(tester, _screen());
      expect(find.byType(SettingsScreen), findsOneWidget);
    });

    testWidgets('Shortcuts tab renders without crashing', (tester) async {
      await _pump(tester, _screen());
      await tester.tap(find.text('Shortcuts'));
      await tester.pump();
      expect(find.byType(SettingsScreen), findsOneWidget);
    });

    testWidgets('Performance tab renders without crashing', (tester) async {
      await _pump(tester, _screen());
      await tester.tap(find.text('Performance'));
      await tester.pump();
      expect(find.byType(SettingsScreen), findsOneWidget);
    });

    testWidgets('Performance tab shows localized paste preset dropdown items', (
      tester,
    ) async {
      await _pump(tester, _screen());
      await tester.tap(find.text('Performance'));
      await tester.pump();

      // Open the DropdownButton to reveal items.
      final dropdown = find.byType(DropdownButton<String>);
      expect(dropdown, findsOneWidget);
      await tester.tap(dropdown);
      await tester.pumpAndSettle();

      // Localized labels should appear (not raw 'Fast'/'Slow' keys).
      expect(find.text('Fast'), findsWidgets);
      expect(find.text('Normal'), findsWidgets);
      expect(find.text('Safe'), findsWidgets);
      expect(find.text('Slow'), findsWidgets);
    });

    testWidgets('Performance tab shows Switch to All on open toggle', (
      tester,
    ) async {
      await _pump(tester, _screen());
      await tester.tap(find.text('Performance'));
      await tester.pumpAndSettle();

      await tester.scrollUntilVisible(
        find.text('Switch to All on open'),
        100,
        scrollable: find.byType(Scrollable).last,
      );

      expect(find.text('Switch to All on open'), findsOneWidget);
    });

    testWidgets('Cleanup & Privacy tab renders without crashing', (
      tester,
    ) async {
      await _pump(tester, _screen());
      await tester.tap(find.text('Cleanup & Privacy'));
      await tester.pump();
      expect(find.byType(SettingsScreen), findsOneWidget);
    });

    testWidgets('Backup & Support tab renders without crashing', (
      tester,
    ) async {
      await _pump(tester, _screen());
      await tester.tap(find.text('Backup & Support'));
      await tester.pump();
      expect(find.byType(SettingsScreen), findsOneWidget);
    });

    testWidgets('About tab renders without crashing', (tester) async {
      await _pump(tester, _screen());
      await tester.tap(find.text('About'));
      await tester.pump();
      expect(find.byType(SettingsScreen), findsOneWidget);
    });
  });
}
