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
  });
}
