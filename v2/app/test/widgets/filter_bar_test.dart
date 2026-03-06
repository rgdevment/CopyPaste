import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/widgets/filter_bar.dart';

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
  });
}
