import 'package:flutter_test/flutter_test.dart';

import 'package:app/theme/compact_theme.dart';

void main() {
  group('CompactTheme', () {
    late CompactTheme theme;

    setUp(() {
      theme = CompactTheme();
    });

    test('id returns compact', () {
      expect(theme.id, 'compact');
    });

    test('name returns Compact', () {
      expect(theme.name, 'Compact');
    });

    test('filterStyle has expected chipSpacing', () {
      expect(theme.filterStyle.chipSpacing, 6);
    });

    test('toolbarStyle has expected buttonSpacing', () {
      expect(theme.toolbarStyle.buttonSpacing, 2);
    });
  });
}
