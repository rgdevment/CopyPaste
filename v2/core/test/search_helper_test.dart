import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  group('SearchHelper', () {
    test('normalizes accented characters', () {
      expect(SearchHelper.normalize('café'), equals('cafe'));
      expect(SearchHelper.normalize('España'), equals('espana'));
      expect(SearchHelper.normalize('piñata'), equals('pinata'));
      expect(SearchHelper.normalize('naïve'), equals('naive'));
    });

    test('converts to lowercase', () {
      expect(SearchHelper.normalize('Hello World'), equals('hello world'));
      expect(SearchHelper.normalize('UPPER'), equals('upper'));
    });

    test('handles empty string', () {
      expect(SearchHelper.normalize(''), equals(''));
    });

    test('handles already normalized text', () {
      expect(SearchHelper.normalize('hello'), equals('hello'));
    });

    test('normalizes ligatures', () {
      expect(SearchHelper.normalize('Straße'), equals('strasse'));
      expect(SearchHelper.normalize('Ærodynamic'), equals('aerodynamic'));
      expect(SearchHelper.normalize('œuvre'), equals('oeuvre'));
    });

    test('normalizes extended Latin characters', () {
      expect(SearchHelper.normalize('Łódź'), equals('lodz'));
      expect(SearchHelper.normalize('Česká'), equals('ceska'));
      expect(SearchHelper.normalize('Kraków'), equals('krakow'));
      expect(SearchHelper.normalize('Győr'), equals('gyor'));
      expect(SearchHelper.normalize('Zürich'), equals('zurich'));
    });
  });
}
