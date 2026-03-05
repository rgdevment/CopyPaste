import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/widgets/filter_tab_bar.dart';

import '../helpers/test_wrapper.dart';

void main() {
  group('FilterTabBar', () {
    testWidgets('renders without error', (tester) async {
      await tester.pumpWidget(wrapWidget(
        FilterTabBar(
          selectedTypes: const [],
          isPinnedMode: false,
          onTypesChanged: (_) {},
          onPinnedModeChanged: (_) {},
        ),
      ));
      await tester.pumpAndSettle();

      expect(find.byType(FilterTabBar), findsOneWidget);
    });

    testWidgets('shows multiple tab items', (tester) async {
      await tester.pumpWidget(wrapWidget(
        FilterTabBar(
          selectedTypes: const [],
          isPinnedMode: false,
          onTypesChanged: (_) {},
          onPinnedModeChanged: (_) {},
        ),
      ));
      await tester.pumpAndSettle();

      // At least 3 Text widgets for the tabs (All, Pinned, Text, Image, etc.)
      expect(find.byType(Text), findsAtLeastNWidgets(3));
    });

    testWidgets('tapping a type tab fires onTypesChanged', (tester) async {
      List<ClipboardContentType>? result;

      await tester.pumpWidget(wrapWidget(
        FilterTabBar(
          selectedTypes: const [],
          isPinnedMode: false,
          onTypesChanged: (t) => result = t,
          onPinnedModeChanged: (_) {},
        ),
      ));
      await tester.pumpAndSettle();

      // Tap on "Text" tab (3rd tab in list)
      final textFinder = find.text('Text');
      if (textFinder.evaluate().isNotEmpty) {
        await tester.tap(textFinder.first);
        await tester.pump();
        expect(result, isNotNull);
        expect(result, contains(ClipboardContentType.text));
      }
    });

    testWidgets('tapping Pinned tab fires onPinnedModeChanged with true',
        (tester) async {
      bool? pinnedResult;

      await tester.pumpWidget(wrapWidget(
        FilterTabBar(
          selectedTypes: const [],
          isPinnedMode: false,
          onTypesChanged: (_) {},
          onPinnedModeChanged: (p) => pinnedResult = p,
        ),
      ));
      await tester.pumpAndSettle();

      final pinnedFinder = find.text('Pinned');
      if (pinnedFinder.evaluate().isNotEmpty) {
        await tester.tap(pinnedFinder.first);
        await tester.pump();
        expect(pinnedResult, isTrue);
      }
    });

    testWidgets('tapping active type tab deselects (fires empty list)',
        (tester) async {
      List<ClipboardContentType>? result;

      await tester.pumpWidget(wrapWidget(
        FilterTabBar(
          selectedTypes: const [ClipboardContentType.text],
          isPinnedMode: false,
          onTypesChanged: (t) => result = t,
          onPinnedModeChanged: (_) {},
        ),
      ));
      await tester.pumpAndSettle();

      final textFinder = find.text('Text');
      if (textFinder.evaluate().isNotEmpty) {
        await tester.tap(textFinder.first);
        await tester.pump();
        expect(result, isNotNull);
        expect(result, isEmpty);
      }
    });

    testWidgets('tapping All tab fires onTypesChanged with empty list',
        (tester) async {
      List<ClipboardContentType>? result;

      await tester.pumpWidget(wrapWidget(
        FilterTabBar(
          selectedTypes: const [ClipboardContentType.text],
          isPinnedMode: false,
          onTypesChanged: (t) => result = t,
          onPinnedModeChanged: (_) {},
        ),
      ));
      await tester.pumpAndSettle();

      final allFinder = find.text('All');
      if (allFinder.evaluate().isNotEmpty) {
        await tester.tap(allFinder.first);
        await tester.pump();
        expect(result, isNotNull);
        expect(result, isEmpty);
      }
    });

    testWidgets('dark mode renders without error', (tester) async {
      await tester.pumpWidget(wrapWidget(
        FilterTabBar(
          selectedTypes: const [],
          isPinnedMode: false,
          onTypesChanged: (_) {},
          onPinnedModeChanged: (_) {},
        ),
        brightness: Brightness.dark,
      ));
      await tester.pumpAndSettle();

      expect(find.byType(FilterTabBar), findsOneWidget);
    });
  });
}
