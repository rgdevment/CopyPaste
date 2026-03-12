import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/widgets/empty_state.dart';

import '../helpers/test_wrapper.dart';

void main() {
  group('EmptyState', () {
    testWidgets('renders without error', (tester) async {
      await tester.pumpWidget(wrapWidget(const EmptyState()));
      await tester.pumpAndSettle();

      expect(find.byType(EmptyState), findsOneWidget);
    });

    testWidgets('shows paste icon', (tester) async {
      await tester.pumpWidget(wrapWidget(const EmptyState()));
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.content_paste_rounded), findsOneWidget);
    });

    testWidgets('renders in dark mode without error', (tester) async {
      await tester.pumpWidget(
        wrapWidget(const EmptyState(), brightness: Brightness.dark),
      );
      await tester.pumpAndSettle();

      expect(find.byType(EmptyState), findsOneWidget);
    });
  });
}
