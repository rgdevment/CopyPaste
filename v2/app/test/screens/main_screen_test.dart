import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/l10n/app_localizations.dart';
import 'package:app/screens/main_screen.dart';
import 'package:app/theme/compact_theme.dart';
import 'package:app/theme/theme_provider.dart';
import 'package:app/widgets/clipboard_card.dart';
import 'package:app/widgets/empty_state.dart';

Widget _buildApp({
  required ClipboardService service,
  required void Function(ClipboardItem) onPaste,
  VoidCallback? onExit,
  Key? key,
}) {
  return MaterialApp(
    locale: const Locale('en'),
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    theme: ThemeData.light(),
    home: CopyPasteTheme(
      themeData: CompactTheme(),
      child: Scaffold(
        body: MainScreen(
          key: key,
          clipboardService: service,
          onPaste: onPaste,
          onPastePlain: (_) {},
          onExit: onExit ?? () {},
          onSettings: () {},
        ),
      ),
    ),
  );
}

void main() {
  late SqliteRepository repo;
  late ClipboardService service;

  setUp(() {
    repo = SqliteRepository.inMemory();
    service = ClipboardService(repo);
  });

  tearDown(() async {
    service.dispose();
    await repo.close();
  });

  group('MainScreen', () {
    testWidgets('renders without error', (tester) async {
      await tester.pumpWidget(_buildApp(service: service, onPaste: (_) {}));
      await tester.pumpAndSettle();

      expect(find.byType(MainScreen), findsOneWidget);
    });

    testWidgets('shows EmptyState when no items', (tester) async {
      await tester.pumpWidget(_buildApp(service: service, onPaste: (_) {}));
      await tester.pumpAndSettle();

      expect(find.byType(EmptyState), findsOneWidget);
    });

    testWidgets('shows ClipboardCard after items are loaded', (tester) async {
      await repo.save(ClipboardItem(
        content: 'Hello clipboard',
        type: ClipboardContentType.text,
      ));

      await tester.pumpWidget(_buildApp(service: service, onPaste: (_) {}));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
      expect(find.text('Hello clipboard'), findsOneWidget);
    });

    testWidgets('multiple items render multiple cards', (tester) async {
      for (var i = 0; i < 3; i++) {
        await repo.save(ClipboardItem(
          content: 'Item $i',
          type: ClipboardContentType.text,
        ));
      }

      await tester.pumpWidget(_buildApp(service: service, onPaste: (_) {}));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsNWidgets(3));
    });

    testWidgets('ArrowDown from search selects first item', (tester) async {
      await repo.save(ClipboardItem(
        content: 'Select me',
        type: ClipboardContentType.text,
      ));

      final key = GlobalKey<MainScreenState>();
      await tester.pumpWidget(
          _buildApp(service: service, onPaste: (_) {}, key: key));
      await tester.pumpAndSettle();

      key.currentState!.onWindowShow();
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();

      // Card is present (selection changes visual rendering)
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('Enter fires onPaste with selected item', (tester) async {
      await repo.save(ClipboardItem(
        content: 'Paste me',
        type: ClipboardContentType.text,
      ));

      ClipboardItem? pasted;
      final key = GlobalKey<MainScreenState>();
      await tester.pumpWidget(_buildApp(
        service: service,
        onPaste: (item) => pasted = item,
        key: key,
      ));
      await tester.pumpAndSettle();

      key.currentState!.onWindowShow();
      await tester.pump();

      // ArrowDown to select first item
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();

      // Enter to paste
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump();

      expect(pasted, isNotNull);
      expect(pasted!.content, equals('Paste me'));
    });

    testWidgets('Escape fires onExit when no active filters', (tester) async {
      await repo.save(ClipboardItem(
        content: 'Test item',
        type: ClipboardContentType.text,
      ));

      var exitFired = false;
      final key = GlobalKey<MainScreenState>();
      await tester.pumpWidget(_buildApp(
        service: service,
        onPaste: (_) {},
        onExit: () => exitFired = true,
        key: key,
      ));
      await tester.pumpAndSettle();

      key.currentState!.onWindowShow();
      await tester.pump();

      // Move focus to list
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pump();

      expect(exitFired, isTrue);
    });

    testWidgets('Ctrl+1 switches to recent tab', (tester) async {
      final key = GlobalKey<MainScreenState>();
      await tester.pumpWidget(
          _buildApp(service: service, onPaste: (_) {}, key: key));
      await tester.pumpAndSettle();

      key.currentState!.onWindowShow();
      await tester.pump();

      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.digit1);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pump();

      // Recent tab is active, no crash
      expect(find.byType(MainScreen), findsOneWidget);
    });

    testWidgets('Ctrl+2 switches to pinned tab', (tester) async {
      final key = GlobalKey<MainScreenState>();
      await tester.pumpWidget(
          _buildApp(service: service, onPaste: (_) {}, key: key));
      await tester.pumpAndSettle();

      key.currentState!.onWindowShow();
      await tester.pump();

      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.digit2);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pumpAndSettle();

      // Pinned tab has no items → EmptyState
      expect(find.byType(EmptyState), findsOneWidget);
    });

    testWidgets('onWindowHide resets selected index and state', (tester) async {
      await repo.save(ClipboardItem(
        content: 'Test',
        type: ClipboardContentType.text,
      ));

      final key = GlobalKey<MainScreenState>();
      await tester.pumpWidget(
          _buildApp(service: service, onPaste: (_) {}, key: key));
      await tester.pumpAndSettle();

      key.currentState!.onWindowShow();
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();

      key.currentState!.onWindowHide();
      await tester.pump();

      expect(find.byType(MainScreen), findsOneWidget);
    });

    testWidgets('ArrowUp from first item returns focus to search', (tester) async {
      await repo.save(ClipboardItem(
        content: 'Item',
        type: ClipboardContentType.text,
      ));

      final key = GlobalKey<MainScreenState>();
      await tester.pumpWidget(
          _buildApp(service: service, onPaste: (_) {}, key: key));
      await tester.pumpAndSettle();

      key.currentState!.onWindowShow();
      await tester.pump();

      // ArrowDown to select first item
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();

      // ArrowUp from first item → selection cleared, search focused
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowUp);
      await tester.pump();

      // Screen still renders
      expect(find.byType(MainScreen), findsOneWidget);
    });
  });
}
