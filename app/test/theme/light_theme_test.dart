import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/theme/light_theme.dart';

void main() {
  group('lightColorScheme', () {
    test('surface is light gray', () {
      expect(lightColorScheme.surface, equals(const Color(0xFFEBEBF0)));
    });

    test('background matches surface', () {
      expect(lightColorScheme.background, equals(const Color(0xFFEBEBF0)));
    });

    test('onSurface is black', () {
      expect(lightColorScheme.onSurface, equals(const Color(0xFF000000)));
    });

    test('primary is indigo', () {
      expect(lightColorScheme.primary, equals(const Color(0xFF4F46E5)));
    });

    test('cardBackground is white', () {
      expect(lightColorScheme.cardBackground, equals(const Color(0xFFFFFFFF)));
    });

    test('danger color is dark red', () {
      expect(lightColorScheme.danger, equals(const Color(0xFFB91C1C)));
    });

    test('warning color is dark amber', () {
      expect(lightColorScheme.warning, equals(const Color(0xFF92400E)));
    });

    test('accent colors are defined', () {
      expect(lightColorScheme.accentRed, equals(const Color(0xFFDC2626)));
      expect(lightColorScheme.accentGreen, equals(const Color(0xFF166534)));
      expect(lightColorScheme.accentPurple, equals(const Color(0xFF3730A3)));
      expect(lightColorScheme.accentYellow, equals(const Color(0xFF92400E)));
      expect(lightColorScheme.accentBlue, equals(const Color(0xFF3730A3)));
      expect(lightColorScheme.accentOrange, equals(const Color(0xFFC2410C)));
    });

    test('accentForIndex returns transparent for index 0', () {
      expect(lightColorScheme.accentForIndex(0), equals(Colors.transparent));
    });

    test('accentForIndex returns accentRed for index 1', () {
      expect(
        lightColorScheme.accentForIndex(1),
        equals(lightColorScheme.accentRed),
      );
    });

    test('onPrimary is white', () {
      expect(lightColorScheme.onPrimary, equals(const Color(0xFFFFFFFF)));
    });
  });
}
