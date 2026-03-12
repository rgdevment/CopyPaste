import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/theme/app_theme_data.dart';
import 'package:app/theme/compact_theme.dart';
import 'package:app/theme/theme_provider.dart';

// Test-only subclasses to simulate different theme IDs
class _ThemeA extends CompactTheme {
  @override
  String get id => 'theme_a';
  @override
  String get name => 'Theme A';
}

class _ThemeB extends CompactTheme {
  @override
  String get id => 'theme_b';
  @override
  String get name => 'Theme B';
}

void main() {
  group('CopyPasteTheme', () {
    testWidgets('of() throws when no CopyPasteTheme in context', (
      WidgetTester tester,
    ) async {
      FlutterError? error;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              try {
                CopyPasteTheme.of(context);
              } on FlutterError catch (e) {
                error = e;
              }
              return const SizedBox.shrink();
            },
          ),
        ),
      );
      expect(error, isNotNull);
    });

    testWidgets('of() returns correct AppThemeData', (
      WidgetTester tester,
    ) async {
      final themeData = _ThemeA();
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
      expect(result!.id, equals('theme_a'));
    });

    testWidgets('updateShouldNotify notifies when theme ID changes', (
      WidgetTester tester,
    ) async {
      int buildCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: CopyPasteTheme(
            themeData: _ThemeA(),
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
            themeData: _ThemeB(),
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
    });

    testWidgets('updateShouldNotify does not notify when same theme', (
      WidgetTester tester,
    ) async {
      int buildCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: CopyPasteTheme(
            themeData: _ThemeA(),
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

      // Update with a different instance of the same theme ID
      await tester.pumpWidget(
        MaterialApp(
          home: CopyPasteTheme(
            themeData: _ThemeA(), // Different instance, same ID
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
    });

    testWidgets('colorsOf returns light colors in light brightness', (
      WidgetTester tester,
    ) async {
      AppThemeColorScheme? colors;

      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData(brightness: Brightness.light),
          home: CopyPasteTheme(
            themeData: _ThemeA(),
            child: Builder(
              builder: (context) {
                colors = CopyPasteTheme.colorsOf(context);
                return const SizedBox.shrink();
              },
            ),
          ),
        ),
      );

      // colorsOf should return the light scheme when Material brightness is light
      expect(colors, isNotNull);
      expect(colors!.surface, equals(_ThemeA().light.surface));
    });

    testWidgets('colorsOf returns dark colors in dark brightness', (
      WidgetTester tester,
    ) async {
      AppThemeColorScheme? colors;

      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData(brightness: Brightness.dark),
          home: CopyPasteTheme(
            themeData: _ThemeA(),
            child: Builder(
              builder: (context) {
                colors = CopyPasteTheme.colorsOf(context);
                return const SizedBox.shrink();
              },
            ),
          ),
        ),
      );

      // colorsOf should return the dark scheme when Material brightness is dark
      expect(colors, isNotNull);
      expect(colors!.surface, equals(_ThemeA().dark.surface));
    });
  });
}
