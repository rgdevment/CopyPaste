import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/l10n/app_localizations.dart';
import 'package:app/theme/compact_theme.dart';
import 'package:app/theme/theme_provider.dart';
import 'package:app/widgets/accessibility_dialog.dart';

import '../helpers/test_wrapper.dart';

void _setMockHandler(
  MethodChannel channel,
  Future<Object?> Function(MethodCall) handler,
) {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(channel, handler);
}

void _clearMockHandler(MethodChannel channel) {
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(channel, null);
}

void main() {
  const channel = MethodChannel('copypaste/clipboard_writer');

  setUp(() {
    _setMockHandler(channel, (call) async {
      switch (call.method) {
        case 'requestAccessibility':
          return false;
        case 'checkAccessibility':
          return false;
        case 'openAccessibilitySettings':
          return null;
        default:
          return null;
      }
    });
  });

  tearDown(() => _clearMockHandler(channel));

  group('AccessibilityDialog', () {
    testWidgets('renders dialog with icon and buttons', (tester) async {
      await tester.pumpWidget(
        wrapWidget(const AccessibilityDialog(previouslyGranted: false)),
      );
      await tester.pumpAndSettle();

      expect(find.byType(AccessibilityDialog), findsOneWidget);
      expect(find.byType(AlertDialog), findsOneWidget);
      expect(find.byIcon(Icons.security), findsOneWidget);
      expect(find.byType(FilledButton), findsOneWidget);
      expect(find.byType(TextButton), findsOneWidget);
    });

    testWidgets('dismiss TextButton pops dialog', (tester) async {
      var popped = false;

      await tester.pumpWidget(
        MaterialApp(
          locale: const Locale('en'),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: CopyPasteTheme(
            themeData: CompactTheme(),
            child: Scaffold(
              body: Builder(
                builder: (ctx) => ElevatedButton(
                  onPressed: () async {
                    await showDialog<void>(
                      context: ctx,
                      builder: (_) =>
                          const AccessibilityDialog(previouslyGranted: false),
                    );
                    popped = true;
                  },
                  child: const Text('Open'),
                ),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();

      expect(find.byType(AlertDialog), findsOneWidget);

      await tester.tap(find.byType(TextButton));
      await tester.pumpAndSettle();

      expect(popped, isTrue);
      expect(find.byType(AlertDialog), findsNothing);
    });

    testWidgets('open settings FilledButton calls openAccessibilitySettings', (
      tester,
    ) async {
      var settingsOpened = false;

      _setMockHandler(channel, (call) async {
        if (call.method == 'openAccessibilitySettings') settingsOpened = true;
        return null;
      });

      await tester.pumpWidget(
        wrapWidget(const AccessibilityDialog(previouslyGranted: false)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byType(FilledButton));
      await tester.pump();

      expect(settingsOpened, isTrue);
    });

    testWidgets(
      'checkAndShow skips dialog when accessibility already granted',
      (tester) async {
        _setMockHandler(channel, (call) async {
          if (call.method == 'checkAccessibility') return true;
          return null;
        });

        var completed = false;

        await tester.pumpWidget(
          MaterialApp(
            home: Builder(
              builder: (ctx) => ElevatedButton(
                onPressed: () async {
                  await AccessibilityDialog.checkAndShow(ctx);
                  completed = true;
                },
                child: const Text('Check'),
              ),
            ),
          ),
        );

        await tester.tap(find.text('Check'));
        await tester.pumpAndSettle();

        expect(find.byType(AlertDialog), findsNothing);
        expect(completed, isTrue);
      },
    );

    testWidgets(
      'poll timer auto-closes dialog when accessibility becomes granted',
      (tester) async {
        var checkCallCount = 0;

        _setMockHandler(channel, (call) async {
          if (call.method == 'requestAccessibility') return false;
          if (call.method == 'checkAccessibility') {
            checkCallCount++;
            return checkCallCount >= 2;
          }
          return null;
        });

        await tester.pumpWidget(
          MaterialApp(
            locale: const Locale('en'),
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            home: CopyPasteTheme(
              themeData: CompactTheme(),
              child: Scaffold(
                body: Builder(
                  builder: (ctx) =>
                      const AccessibilityDialog(previouslyGranted: false),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();

        expect(find.byType(AccessibilityDialog), findsOneWidget);

        // Advance clock past two timer ticks (1s each)
        await tester.pump(const Duration(seconds: 3));
        await tester.pumpAndSettle();

        expect(checkCallCount, greaterThanOrEqualTo(2));
      },
    );

    testWidgets('dispose cancels poll timer without errors', (tester) async {
      await tester.pumpWidget(
        wrapWidget(const AccessibilityDialog(previouslyGranted: false)),
      );
      await tester.pumpAndSettle();

      // Replacing widget tree disposes the dialog state
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pumpAndSettle();

      // No exception after disposal
      expect(find.byType(AccessibilityDialog), findsNothing);
    });
  });
}
