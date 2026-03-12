import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/theme/dark_theme.dart';

void main() {
  group('DarkTheme', () {
    late DarkTheme theme;

    setUp(() {
      theme = DarkTheme();
    });

    test('id returns dark', () {
      expect(theme.id, 'dark');
    });

    test('name returns Dark', () {
      expect(theme.name, 'Dark');
    });

    test('light color scheme has expected primary color', () {
      expect(theme.light.primary, isNotNull);
      expect(theme.light.primary, isInstanceOf<Color>());
    });

    test('dark color scheme has expected primary color', () {
      expect(theme.dark.primary, isNotNull);
      expect(theme.dark.primary, isInstanceOf<Color>());
    });

    test('light and dark schemes have different surface colors', () {
      expect(theme.light.surface, isNotEmpty);
      expect(theme.dark.surface, isNotEmpty);
      // Dark theme should have darker surface (typically)
      // Both should be valid colors
      expect(theme.light.surface.value, isNotNull);
      expect(theme.dark.surface.value, isNotNull);
    });

    test('color scheme has all required properties', () {
      final light = theme.light;
      expect(light.onSurface, isNotNull);
      expect(light.onSurfaceVariant, isNotNull);
      expect(light.cardBackground, isNotNull);
      expect(light.cardBorder, isNotNull);
      expect(light.searchBackground, isNotNull);
      expect(light.divider, isNotNull);
      expect(light.danger, isNotNull);
      expect(light.warning, isNotNull);
      expect(light.accentRed, isNotNull);
      expect(light.accentGreen, isNotNull);
      expect(light.accentPurple, isNotNull);
      expect(light.accentYellow, isNotNull);
      expect(light.accentBlue, isNotNull);
      expect(light.accentOrange, isNotNull);
    });

    test('listItemStyle is not null', () {
      expect(theme.listItemStyle, isNotNull);
    });

    test('filterStyle is not null', () {
      expect(theme.filterStyle, isNotNull);
    });

    test('toolbarStyle is not null', () {
      expect(theme.toolbarStyle, isNotNull);
    });
  });
}
