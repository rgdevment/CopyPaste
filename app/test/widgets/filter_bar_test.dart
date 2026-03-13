import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/widgets/filter_bar.dart';

import '../helpers/test_wrapper.dart';

void main() {
  group('FilterBar', () {
    testWidgets('renders without error when no colors selected', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            selectedTypes: const [],
            selectedColors: const [],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(FilterBar), findsOneWidget);
    });

    testWidgets('shows no badge when no colors selected', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            selectedTypes: const [],
            selectedColors: const [],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // With no active filters, badge text should not appear
      expect(find.text('1'), findsNothing);
      expect(find.text('2'), findsNothing);
    });

    testWidgets('shows badge with count when colors selected', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            selectedTypes: const [],
            selectedColors: const [CardColor.red, CardColor.blue],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Badge should show count "2"
      expect(find.text('2'), findsOneWidget);
    });

    testWidgets('shows badge count 1 for single color', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            selectedTypes: const [],
            selectedColors: const [CardColor.green],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('1'), findsOneWidget);
    });

    testWidgets('has tappable button area', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            selectedTypes: const [],
            selectedColors: const [],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // The FilterBar renders a tappable button (GestureDetector or InkWell)
      expect(
        find.byWidgetPredicate(
          (w) => w is GestureDetector || w is InkWell || w is MouseRegion,
        ),
        findsAtLeastNWidgets(1),
      );
    });

    testWidgets('openMenu can be called via GlobalKey', (tester) async {
      final key = GlobalKey<FilterBarState>();

      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            key: key,
            selectedTypes: const [],
            selectedColors: const [],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Should not throw
      key.currentState!.openMenu();
      await tester.pumpAndSettle();
    });

    testWidgets('dark mode renders without error', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            selectedTypes: const [],
            selectedColors: const [CardColor.red],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
          ),
          brightness: Brightness.dark,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(FilterBar), findsOneWidget);
    });

    testWidgets('openMenu with active color shows clear option', (
      tester,
    ) async {
      final key = GlobalKey<FilterBarState>();
      var clearCalled = false;

      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            key: key,
            selectedTypes: const [],
            selectedColors: const [CardColor.red],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
            onClear: () => clearCalled = true,
          ),
        ),
      );
      await tester.pumpAndSettle();

      key.currentState!.openMenu();
      await tester.pumpAndSettle();

      // Menu is open - clear option should be first item
      final menuItems = find.byType(PopupMenuItem<void>);
      expect(menuItems, findsAtLeastNWidgets(1));

      // Tap first menu item (Clear all filters)
      await tester.tap(menuItems.first);
      await tester.pumpAndSettle();

      expect(clearCalled, isTrue);
    });

    testWidgets('openMenu with active color, tap color removes it', (
      tester,
    ) async {
      final key = GlobalKey<FilterBarState>();
      List<CardColor>? updated;

      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            key: key,
            selectedTypes: const [],
            selectedColors: const [CardColor.red],
            onTypesChanged: (_) {},
            onColorsChanged: (c) => updated = c,
            onClear: () {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      key.currentState!.openMenu();
      await tester.pumpAndSettle();

      // Menu items when selectedColors=[red]:
      //  0: "Clear all filters"
      //  1: Color section label (disabled)
      //  2: Red (selected) ← tapping this removes red
      //  3: Green, 4: Purple, 5: Yellow, 6: Blue, 7: Orange
      final menuItems = find.byType(PopupMenuItem<void>);
      if (menuItems.evaluate().length >= 3) {
        await tester.tap(menuItems.at(2)); // Red = selected → removes it
        await tester.pumpAndSettle();
        expect(updated, isNotNull);
        expect(updated!.contains(CardColor.red), isFalse);
      }
    });

    testWidgets('openMenu, tap unselected color adds it', (tester) async {
      final key = GlobalKey<FilterBarState>();
      List<CardColor>? updated;

      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            key: key,
            selectedTypes: const [],
            selectedColors: const [],
            onTypesChanged: (_) {},
            onColorsChanged: (c) => updated = c,
            onClear: () {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      key.currentState!.openMenu();
      await tester.pumpAndSettle();

      // No clear item since no active filters - first PopupMenuItems are label + color
      final menuItems = find.byType(PopupMenuItem<void>);
      // Skip label item (0), tap color item (1)
      if (menuItems.evaluate().length >= 2) {
        await tester.tap(menuItems.at(1));
        await tester.pumpAndSettle();
        expect(updated, isNotNull);
        expect(updated!.length, 1);
      }
    });

    testWidgets('tapping filter button opens menu', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          FilterBar(
            selectedTypes: const [],
            selectedColors: const [],
            onTypesChanged: (_) {},
            onColorsChanged: (_) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Tap the InkWell (filter button)
      await tester.tap(find.byType(InkWell).first);
      await tester.pumpAndSettle();

      // Menu should be shown (PopupMenuItems rendered)
      expect(find.byType(PopupMenuItem<void>), findsAtLeastNWidgets(1));
    });
  });
}
