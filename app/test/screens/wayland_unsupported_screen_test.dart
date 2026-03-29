import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/l10n/app_localizations.dart';
import 'package:copypaste/screens/wayland_unsupported_screen.dart';
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
  tester.view.physicalSize = const Size(640, 960);
  tester.view.devicePixelRatio = 2.0;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(_wrap(child, locale: locale));
  await tester.pump();
}

void main() {
  Widget screen({VoidCallback? onClose}) =>
      WaylandUnsupportedScreen(onClose: onClose ?? () {});

  group('WaylandUnsupportedScreen', () {
    testWidgets('renders title text', (tester) async {
      await _pump(tester, screen());

      expect(find.text('Wayland is not supported yet'), findsOneWidget);
    });

    testWidgets('renders badge chip', (tester) async {
      await _pump(tester, screen());

      expect(find.text('Open source · X11 only'), findsOneWidget);
    });

    testWidgets('renders body text', (tester) async {
      await _pump(tester, screen());

      expect(
        find.textContaining('Linux support is still limited'),
        findsOneWidget,
      );
    });

    testWidgets('renders GitHub FilledButton', (tester) async {
      await _pump(tester, screen());

      expect(find.byType(FilledButton), findsOneWidget);
      expect(find.text('Contribute on GitHub'), findsOneWidget);
    });

    testWidgets('renders Close OutlinedButton', (tester) async {
      await _pump(tester, screen());

      expect(find.byType(OutlinedButton), findsOneWidget);
      expect(find.text('Close'), findsOneWidget);
    });

    testWidgets('tapping close button invokes onClose', (tester) async {
      var closed = false;
      await _pump(tester, screen(onClose: () => closed = true));

      await tester.ensureVisible(find.byType(OutlinedButton));
      await tester.tap(find.byType(OutlinedButton));
      await tester.pump();

      expect(closed, isTrue);
    });

    testWidgets('app icon is displayed', (tester) async {
      await _pump(tester, screen());

      expect(find.byType(Image), findsOneWidget);
    });

    testWidgets('renders both action buttons', (tester) async {
      await _pump(tester, screen());

      expect(find.byType(FilledButton), findsOneWidget);
      expect(find.byType(OutlinedButton), findsOneWidget);
    });

    testWidgets('renders in Spanish locale', (tester) async {
      await _pump(tester, screen(), locale: const Locale('es'));

      expect(find.text('Wayland no está soportado aún'), findsOneWidget);
      expect(find.text('Open source · Solo X11'), findsOneWidget);
      expect(find.text('Cerrar'), findsOneWidget);
    });

    testWidgets('renders in dark mode without errors', (tester) async {
      tester.view.physicalSize = const Size(640, 960);
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

      expect(find.text('Wayland is not supported yet'), findsOneWidget);
    });

    testWidgets('open_in_new icon is present on GitHub button', (tester) async {
      await _pump(tester, screen());

      expect(find.byIcon(Icons.open_in_new_rounded), findsOneWidget);
    });
  });
}
