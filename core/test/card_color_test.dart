import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  group('CardColor', () {
    test('none has value 0 and transparent argb', () {
      expect(CardColor.none.value, equals(0));
      expect(CardColor.none.argb, equals(0x00000000));
    });

    test('all non-none colors have non-zero argb', () {
      for (final color in CardColor.values) {
        if (color == CardColor.none) continue;
        expect(
          color.argb,
          isNot(equals(0)),
          reason: '${color.name} should have non-zero argb',
        );
      }
    });

    test('all values are unique', () {
      final values = CardColor.values.map((c) => c.value).toList();
      expect(
        values.toSet().length,
        equals(values.length),
        reason: 'all CardColor values must be unique',
      );
    });

    test('fromValue returns correct color for each defined value', () {
      expect(CardColor.fromValue(0), equals(CardColor.none));
      expect(CardColor.fromValue(1), equals(CardColor.red));
      expect(CardColor.fromValue(2), equals(CardColor.green));
      expect(CardColor.fromValue(3), equals(CardColor.purple));
      expect(CardColor.fromValue(4), equals(CardColor.yellow));
      expect(CardColor.fromValue(5), equals(CardColor.blue));
      expect(CardColor.fromValue(6), equals(CardColor.orange));
    });

    test('fromValue returns none for unknown positive value', () {
      expect(CardColor.fromValue(99), equals(CardColor.none));
      expect(CardColor.fromValue(100), equals(CardColor.none));
    });

    test('fromValue returns none for negative value', () {
      expect(CardColor.fromValue(-1), equals(CardColor.none));
      expect(CardColor.fromValue(-100), equals(CardColor.none));
    });

    test('roundtrip: value → fromValue for all colors', () {
      for (final color in CardColor.values) {
        expect(
          CardColor.fromValue(color.value),
          equals(color),
          reason: 'roundtrip failed for ${color.name}',
        );
      }
    });

    test('none is not equal to red', () {
      expect(CardColor.none == CardColor.red, isFalse);
    });
    test('toString returns enum name', () {
      expect(CardColor.red.toString(), contains('CardColor.red'));
    });
    test('none value is 0', () {
      expect(CardColor.none.value, equals(0));
    });

    test('7 total colors exist', () {
      expect(CardColor.values.length, equals(7));
    });

    test('each color has correct argb value', () {
      expect(CardColor.red.argb, equals(0xFFE74C3C));
      expect(CardColor.green.argb, equals(0xFF2ECC71));
      expect(CardColor.purple.argb, equals(0xFF9B59B6));
      expect(CardColor.yellow.argb, equals(0xFFF1C40F));
      expect(CardColor.blue.argb, equals(0xFF3498DB));
      expect(CardColor.orange.argb, equals(0xFFE67E22));
    });
  });
}
