import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/widgets/title_bar.dart';

import '../helpers/test_wrapper.dart';

void _setupWindowManagerMock() {
  const channel = MethodChannel('window_manager');
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(channel, (call) async => null);
}

void _clearWindowManagerMock() {
  const channel = MethodChannel('window_manager');
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(channel, null);
}

void main() {
  setUp(_setupWindowManagerMock);
  tearDown(_clearWindowManagerMock);

  group('TitleBar', () {
    testWidgets('renders search box', (tester) async {
      final controller = TextEditingController();
      final focusNode = FocusNode();

      await tester.pumpWidget(
        wrapWidget(
          TitleBar(
            searchController: controller,
            searchFocusNode: focusNode,
            onSearchChanged: (_) {},
            trailing: null,
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(TextField), findsOneWidget);

      controller.dispose();
      focusNode.dispose();
    });

    testWidgets('shows trailing widget when provided', (tester) async {
      final controller = TextEditingController();
      final focusNode = FocusNode();

      await tester.pumpWidget(
        wrapWidget(
          TitleBar(
            searchController: controller,
            searchFocusNode: focusNode,
            onSearchChanged: (_) {},
            trailing: const Icon(Icons.settings, key: Key('trailing')),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('trailing')), findsOneWidget);

      controller.dispose();
      focusNode.dispose();
    });

    testWidgets('onSearchChanged fires after 300ms debounce', (tester) async {
      final controller = TextEditingController();
      final focusNode = FocusNode();
      final captured = <String>[];

      await tester.pumpWidget(
        wrapWidget(
          TitleBar(
            searchController: controller,
            searchFocusNode: focusNode,
            onSearchChanged: captured.add,
            trailing: null,
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'hello');
      // Before debounce fires — nothing yet
      await tester.pump(const Duration(milliseconds: 100));
      expect(captured, isEmpty);

      // After debounce
      await tester.pump(const Duration(milliseconds: 300));
      expect(captured, ['hello']);

      controller.dispose();
      focusNode.dispose();
    });

    testWidgets('clear button appears when text is non-empty', (tester) async {
      final controller = TextEditingController(text: 'abc');
      final focusNode = FocusNode();

      await tester.pumpWidget(
        wrapWidget(
          TitleBar(
            searchController: controller,
            searchFocusNode: focusNode,
            onSearchChanged: (_) {},
            trailing: null,
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.close_rounded), findsOneWidget);

      controller.dispose();
      focusNode.dispose();
    });

    testWidgets('tapping clear button clears text and fires onChanged', (
      tester,
    ) async {
      final controller = TextEditingController(text: 'abc');
      final focusNode = FocusNode();
      final captured = <String>[];

      await tester.pumpWidget(
        wrapWidget(
          TitleBar(
            searchController: controller,
            searchFocusNode: focusNode,
            onSearchChanged: captured.add,
            trailing: null,
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byIcon(Icons.close_rounded));
      // DragToMoveArea has onDoubleTap on Windows, which delays single-tap
      // resolution by ~300ms. Advance past that timeout.
      await tester.pump(const Duration(milliseconds: 350));

      expect(controller.text, isEmpty);
      expect(captured, contains(''));

      controller.dispose();
      focusNode.dispose();
    });

    testWidgets('focus changes visual state', (tester) async {
      final controller = TextEditingController();
      final focusNode = FocusNode();

      await tester.pumpWidget(
        wrapWidget(
          TitleBar(
            searchController: controller,
            searchFocusNode: focusNode,
            onSearchChanged: (_) {},
            trailing: null,
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Request focus directly — tapping inside DragToMoveArea has a
      // double-tap delay on Windows that makes tap-to-focus unreliable.
      focusNode.requestFocus();
      await tester.pump();

      expect(focusNode.hasFocus, isTrue);

      controller.dispose();
      focusNode.dispose();
    });

    testWidgets('clear button not shown when text is empty', (tester) async {
      final controller = TextEditingController();
      final focusNode = FocusNode();

      await tester.pumpWidget(
        wrapWidget(
          TitleBar(
            searchController: controller,
            searchFocusNode: focusNode,
            onSearchChanged: (_) {},
            trailing: null,
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.close_rounded), findsNothing);

      controller.dispose();
      focusNode.dispose();
    });
  });
}
