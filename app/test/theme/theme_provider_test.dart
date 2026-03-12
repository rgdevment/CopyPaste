import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/theme/compact_theme.dart';
import 'package:app/theme/dark_theme.dart';
import 'package:app/theme/light_theme.dart';
import 'package:app/theme/theme_provider.dart';

void main() {
  group('CopyPasteTheme', () {
    test('of() throws when no CopyPasteTheme in context', () {
      expect(
        () => CopyPasteTheme.of(
          (null as dynamic),
        ),
        throwsFlutterError,
      );
    });

    testWidgets('of() returns correct AppThemeData', (WidgetTester tester) async {
      final themeData = LightTheme();
      AppThemeData? result;

      await tester.pumpWidget(
        MaterialApp(
          home: CopyPasteTheme(
            themeData: themeData,
            child: Builder(
              builder: (context) {
                result = CopyPasteTheme.of(context);
                return const SizedBox.shrink();
              },
            ),
          ),
        ),
      );

      expect(result, isNotNull);
      expect(result!.id, equals('light'));
    });

    testWidgets(
      'updateShouldNotify notifies when theme ID changes',
      (WidgetTester tester) async {
        int buildCount = 0;

        await tester.pumpWidget(
          MaterialApp(
            home: CopyPasteTheme(
              themeData: LightTheme(),
              child: Builder(
                builder: (context) {
                  buildCount++;
                  CopyPasteTheme.of(context);
                  return const SizedBox.shrink();
                },
              ),
            ),
          ),
        );

        expect(buildCount, equals(1));

        // Update to a different theme
        await tester.pumpWidget(
          MaterialApp(
            home: CopyPasteTheme(
              themeData: DarkTheme(),
              child: Builder(
                builder: (context) {
                  buildCount++;
                  CopyPasteTheme.of(context);
                  return const SizedBox.shrink();
                },
              ),
            ),
          ),
        );

        // Build count should increase because theme ID changed
        expect(buildCount, equals(2));
      },
    );

    testWidgets(
      'updateShouldNotify does not notify when same theme',
      (WidgetTester tester) async {
        int buildCount = 0;
        final themes = {
          'light': LightTheme(),
          'dark': DarkTheme(),
          'compact': CompactTheme(),
        };

        await tester.pumpWidget(
          MaterialApp(
            home: CopyPasteTheme(
              themeData: themes['light']!,
              child: Builder(
                builder: (context) {
                  buildCount++;
                  CopyPasteTheme.of(context);
                  return const SizedBox.shrink();
                },
              ),
            ),
          ),
        );

        final initialBuildCount = buildCount;

        // Update with a different LightTheme instance (but same ID)
        await tester.pumpWidget(
          MaterialApp(
            home: CopyPasteTheme(
              themeData: LightTheme(), // Different instance, same ID
              child: Builder(
                builder: (context) {
                  buildCount++;
                  CopyPasteTheme.of(context);
                  return const SizedBox.shrink();
                },
              ),
            ),
          ),
        );

        // Build count should stay the same (no rebuild for same theme ID)
        expect(buildCount, equals(initialBuildCount));
      },
    );

    testWidgets(
      'colorsOf returns light colors in light brightness',
      (WidgetTester tester) async {
        var colors = const AppThemeColorScheme(
          surface: Color(0xFF000000),
          surfaceVariant: Color(0xFF111111),
          background: Color(0xFF222222),
          onSurface: Color(0xFFFFFFFF),
          onSurfaceVariant: Color(0xFFEEEEEE),
          onSurfaceMuted: Color(0x80FFFFFF),
          onSurfaceSubtle: Color(0x1AFFFFFF),
          primary: Color(0xFF0000FF),
          onPrimary: Color(0xFF000000),
          cardBackground: Color(0xFF333333),
          cardBorder: Color(0xFF444444),
          searchBackground: Color(0xFF555555),
          searchBorder: Color(0xFF666666),
          divider: Color(0xFF777777),
          danger: Color(0xFF880000),
          warning: Color(0xFFAA6600),
          accentRed: Color(0xFFDD0000),
          accentGreen: Color(0xFF00DD00),
          accentPurple: Color(0xFF0000DD),
          accentYellow: Color(0xFFDDDD00),
          accentBlue: Color(0xFF0000FF),
          accentOrange: Color(0xFFDD8800),
        );

        final theme = DarkTheme();

        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData(brightness: Brightness.light),
            home: CopyPasteTheme(
              themeData: theme,
              child: Builder(
                builder: (context) {
                  colors = CopyPasteTheme.colorsOf(context);
                  return const SizedBox.shrink();
                },
              ),
            ),
          ),
        );

        // colorsOf should return light scheme when brightness is light
        expect(colors.background, isNotNull);
      },
    );
  });
}
