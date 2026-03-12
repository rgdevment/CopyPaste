import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/theme/dark_theme.dart';

void main() {
  group('darkColorScheme', () {
    test('surface is dark', () {
      expect(darkColorScheme.surface, equals(const Color(0xFF1A1D2E)));
    });

    test('background matches surface', () {
      expect(darkColorScheme.background, equals(const Color(0xFF1A1D2E)));
    });

    test('onSurface is white', () {
      expect(darkColorScheme.onSurface, equals(const Color(0xFFFFFFFF)));
    });

    test('primary is light indigo', () {
      expect(darkColorScheme.primary, equals(const Color(0xFF818CF8)));
    });

    test('cardBackground is slightly lighter than surface', () {
      expect(darkColorScheme.cardBackground, equals(const Color(0xFF1E2132)));
    });

    test('danger color is light red', () {
      expect(darkColorScheme.danger, equals(const Color(0xFFFCA5A5)));
    });

    test('warning color is light yellow', () {
      expect(darkColorScheme.warning, equals(const Color(0xFFFDE047)));
    });

    test('accent colors are all defined', () {
      expect(darkColorScheme.accentRed, equals(const Color(0xFFFCA5A5)));
      expect(darkColorScheme.accentGreen, equals(const Color(0xFF86EFAC)));
      expect(darkColorScheme.accentPurple, equals(const Color(0xFFA5B4FC)));
      expect(darkColorScheme.accentYellow, equals(const Color(0xFFFDE047)));
      expect(darkColorScheme.accentBlue, equals(const Color(0xFFA5B4FC)));
      expect(darkColorScheme.accentOrange, equals(const Color(0xFFFDBA74)));
    });

    test('accentForIndex returns transparent for index 0', () {
      expect(darkColorScheme.accentForIndex(0), equals(Colors.transparent));
    });

    test('accentForIndex returns accentRed for index 1', () {
      expect(
        darkColorScheme.accentForIndex(1),
        equals(darkColorScheme.accentRed),
      );
    });

    test('accentForIndex returns accentGreen for index 2', () {
      expect(
        darkColorScheme.accentForIndex(2),
        equals(darkColorScheme.accentGreen),
      );
    });
  });
}
