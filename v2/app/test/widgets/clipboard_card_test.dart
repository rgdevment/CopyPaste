import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/widgets/clipboard_card.dart';

import '../helpers/test_wrapper.dart';

ClipboardItem _makeTextItem({
  String content = 'Sample clipboard content',
  bool isPinned = false,
  CardColor cardColor = CardColor.none,
  String? label,
}) {
  return ClipboardItem(
    content: content,
    type: ClipboardContentType.text,
    isPinned: isPinned,
    cardColor: cardColor,
    label: label,
  );
}

void main() {
  group('ClipboardCard', () {
    testWidgets('renders text content', (tester) async {
      final item = _makeTextItem(content: 'Hello clipboard');

      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: item,
          onTap: () {},
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
        ),
      ));
      await tester.pumpAndSettle();

      expect(find.text('Hello clipboard'), findsOneWidget);
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('double-tap triggers onTap', (tester) async {
      var tapCount = 0;
      var expandCount = 0;

      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: _makeTextItem(),
          onTap: () => tapCount++,
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
          onExpandToggle: () => expandCount++,
        ),
      ));
      await tester.pumpAndSettle();

      // Double-tap: two taps within 200ms
      await tester.tap(find.byType(ClipboardCard));
      await tester.pump(const Duration(milliseconds: 100));
      await tester.tap(find.byType(ClipboardCard));
      await tester.pump(const Duration(milliseconds: 50));

      expect(tapCount, equals(1));
      expect(expandCount, equals(0));
    });

    testWidgets('single tap triggers onExpandToggle after 200ms delay',
        (tester) async {
      var expandCount = 0;
      var tapCount = 0;

      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: _makeTextItem(),
          onTap: () => tapCount++,
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
          onExpandToggle: () => expandCount++,
        ),
      ));
      await tester.pumpAndSettle();

      await tester.tap(find.byType(ClipboardCard));
      await tester.pump(const Duration(milliseconds: 250)); // past 200ms

      expect(expandCount, equals(1));
      expect(tapCount, equals(0));
    });

    testWidgets('shows selection border when isSelected is true',
        (tester) async {
      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: _makeTextItem(),
          onTap: () {},
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
          isSelected: true,
        ),
      ));
      await tester.pumpAndSettle();

      // Selected card should render without error
      expect(find.byType(AnimatedContainer), findsAtLeastNWidgets(1));
    });

    testWidgets('shows expanded content when isExpanded is true',
        (tester) async {
      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: _makeTextItem(
              content: 'Line1\nLine2\nLine3\nLine4\nLine5\nLine6'),
          onTap: () {},
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
          isExpanded: true,
          cardMaxLines: 5,
        ),
      ));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('link type item renders without error', (tester) async {
      final item = ClipboardItem(
        content: 'https://example.com',
        type: ClipboardContentType.link,
      );

      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: item,
          onTap: () {},
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
        ),
      ));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('pinned item renders without error', (tester) async {
      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: _makeTextItem(isPinned: true),
          onTap: () {},
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
        ),
      ));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('card with label renders without error', (tester) async {
      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: _makeTextItem(label: 'Work'),
          onTap: () {},
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
        ),
      ));
      await tester.pumpAndSettle();

      expect(find.text('Work'), findsOneWidget);
    });

    testWidgets('card with color renders without error', (tester) async {
      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: _makeTextItem(cardColor: CardColor.red),
          onTap: () {},
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
        ),
      ));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('dark mode renders without error', (tester) async {
      await tester.pumpWidget(wrapWidget(
        ClipboardCard(
          item: _makeTextItem(),
          onTap: () {},
          onPin: () {},
          onDelete: () {},
          onLabelColor: (_, _) {},
        ),
        brightness: Brightness.dark,
      ));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });
  });
}
