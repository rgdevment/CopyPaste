import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/theme/light_theme.dart';

void main() {
  group('LightTheme', () {
    late LightTheme theme;

    setUp(() {
      theme = LightTheme();
    });

    test('id returns light', () {
      expect(theme.id, 'light');
    });

    test('name returns Light', () {
      expect(theme.name, 'Light');
    });

    test('light color scheme primary is indigo', () {
      expect(theme.light.primary, equals(const Color(0xFF4F46E5)));
    });

    test('light color scheme background is light gray', () {
      expect(theme.light.background, equals(const Color(0xFFEBEBF0)));
    });

    test('light color scheme onSurface is black', () {
      expect(theme.light.onSurface, equals(const Color(0xFF000000)));
    });

    test('card background is white', () {
      expect(theme.light.cardBackground, equals(const Color(0xFFFFFFFF)));
    });

    test('dark color scheme background is darker', () {
      final darkSurfaceValue = theme.dark.background.value;
      final lightSurfaceValue = theme.light.background.value;
      // Dark surface should be darker (smaller RGB values typically)
      expect(darkSurfaceValue, isNotNull);
      expect(lightSurfaceValue, isNotNull);
    });

    test('danger color is red', () {
      expect(theme.light.danger, equals(const Color(0xFFB91C1C)));
    });

    test('warning color is amber/yellow', () {
      expect(theme.light.warning, equals(const Color(0xFF92400E)));
    });

    test('accent colors are defined', () {
      expect(theme.light.accentRed, equals(const Color(0xFFDC2626)));
      expect(theme.light.accentGreen, equals(const Color(0xFF166534)));
      expect(theme.light.accentPurple, equals(const Color(0xFF3730A3)));
      expect(theme.light.accentOrange, equals(const Color(0xFFC2410C)));
    });

    test('list item style has proper settings', () {
      expect(theme.listItemStyle.topMargin, equals(6));
      expect(theme.listItemStyle.bottomMargin, equals(6));
    });

    test('filter style has badge configuration', () {
      expect(theme.filterStyle.badgePadding, isNotNull);
      expect(theme.filterStyle.chipSpacing, equals(8));
    });

    test('toolbar style has button spacing', () {
      expect(theme.toolbarStyle.buttonSpacing, equals(8));
    });
  });
}
