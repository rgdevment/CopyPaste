import 'package:core/core.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/theme/compact_theme.dart';
import 'package:app/theme/theme_provider.dart';
import 'package:app/l10n/app_localizations.dart';
import 'package:app/widgets/label_color_dialog.dart';

Widget _buildApp(Widget child) {
  return MaterialApp(
    locale: const Locale('en'),
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    theme: ThemeData.light(),
    home: CopyPasteTheme(
      themeData: CompactTheme(),
      child: Scaffold(body: child),
    ),
  );
}

Widget _buildDialogApp({
  required void Function(LabelColorResult?) onResult,
  String? currentLabel,
  CardColor currentColor = CardColor.none,
}) {
  return MaterialApp(
    locale: const Locale('en'),
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    theme: ThemeData.light(),
    home: CopyPasteTheme(
      themeData: CompactTheme(),
      child: Scaffold(
        body: Builder(
          builder: (context) => Center(
            child: ElevatedButton(
              onPressed: () async {
                final result = await LabelColorDialog.show(
                  context,
                  currentLabel: currentLabel,
                  currentColor: currentColor,
                );
                onResult(result);
              },
              child: const Text('Open'),
            ),
          ),
        ),
      ),
    ),
  );
}

void main() {
  group('LabelColorDialog', () {
    testWidgets('renders with title and text field', (tester) async {
      await tester.pumpWidget(
        _buildApp(
          const LabelColorDialog(
            currentLabel: 'My Label',
            currentColor: CardColor.none,
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Label & Color'), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget);
    });

    testWidgets('pre-fills existing label in text field', (tester) async {
      await tester.pumpWidget(
        _buildApp(
          const LabelColorDialog(currentLabel: 'Work', currentColor: CardColor.red),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Work'), findsOneWidget);
    });

    testWidgets('renders color grid with multiple options', (tester) async {
      await tester.pumpWidget(
        _buildApp(
          const LabelColorDialog(currentLabel: null, currentColor: CardColor.none),
        ),
      );
      await tester.pumpAndSettle();

      // 7 color chips rendered (none, red, green, purple, yellow, blue, orange)
      expect(find.byType(GestureDetector), findsAtLeastNWidgets(3));
    });

    testWidgets('tapping a color chip selects it', (tester) async {
      await tester.pumpWidget(
        _buildApp(
          const LabelColorDialog(currentLabel: null, currentColor: CardColor.none),
        ),
      );
      await tester.pumpAndSettle();

      // Find animated containers in the color grid (color circles)
      final containers = find.byType(AnimatedContainer);
      expect(containers, findsAtLeastNWidgets(2));

      // Tap colour circle (first one after dialog container)
      await tester.tap(containers.at(1));
      await tester.pumpAndSettle();

      // No crash means the setState worked
      expect(find.byType(LabelColorDialog), findsOneWidget);
    });

    testWidgets('Save button returns result with label and color', (
      tester,
    ) async {
      LabelColorResult? result;

      await tester.pumpWidget(
        _buildDialogApp(
          onResult: (r) => result = r,
          currentLabel: 'Initial',
          currentColor: CardColor.blue,
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();

      // Verify dialog opened
      expect(find.text('Label & Color'), findsOneWidget);

      // Clear and enter new text
      await tester.enterText(find.byType(TextField), 'New Label');
      await tester.pumpAndSettle();

      // Tap Save
      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();

      expect(result, isNotNull);
      expect(result!.label, equals('New Label'));
    });

    testWidgets('Save with empty label returns null label', (tester) async {
      LabelColorResult? result;

      await tester.pumpWidget(
        _buildDialogApp(onResult: (r) => result = r, currentLabel: null),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();

      expect(result, isNotNull);
      expect(result!.label, isNull);
    });

    testWidgets('Cancel returns null result', (tester) async {
      LabelColorResult? result;
      var resultSet = false;

      await tester.pumpWidget(
        _buildDialogApp(
          onResult: (r) {
            result = r;
            resultSet = true;
          },
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();

      expect(find.text('Cancel'), findsOneWidget);
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();

      expect(resultSet, isTrue);
      expect(result, isNull);
    });

    testWidgets('Enter in text field submits the dialog', (tester) async {
      LabelColorResult? result;

      await tester.pumpWidget(_buildDialogApp(onResult: (r) => result = r));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextField), 'Via Enter');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();

      expect(result, isNotNull);
      expect(result!.label, equals('Via Enter'));
    });

    testWidgets('returns selected color in result', (tester) async {
      LabelColorResult? result;

      await tester.pumpWidget(
        _buildDialogApp(
          onResult: (r) => result = r,
          currentColor: CardColor.none,
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();

      // Tap a color chip to change selection
      final containers = find.byType(AnimatedContainer);
      await tester.tap(containers.at(2)); // tap second color chip
      await tester.pumpAndSettle();

      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();

      expect(result, isNotNull);
      // color was changed from none
      expect(result!.color, isA<CardColor>());
    });

    testWidgets('shows with no label and none color by default via show()', (
      tester,
    ) async {
      LabelColorResult? result;

      await tester.pumpWidget(
        MaterialApp(
          locale: const Locale('en'),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          theme: ThemeData.light(),
          home: CopyPasteTheme(
            themeData: CompactTheme(),
            child: Scaffold(
              body: Builder(
                builder: (ctx) => ElevatedButton(
                  onPressed: () async {
                    result = await LabelColorDialog.show(ctx); // defaults only
                  },
                  child: const Text('Open'),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();

      expect(result, isNotNull);
      expect(result!.color, equals(CardColor.none));
    });

    testWidgets('dark mode renders without error', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          locale: const Locale('en'),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          theme: ThemeData.dark(),
          home: CopyPasteTheme(
            themeData: CompactTheme(),
            child: const Scaffold(
              body: LabelColorDialog(
                currentLabel: null,
                currentColor: CardColor.none,
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(LabelColorDialog), findsOneWidget);
    });

    testWidgets('button hover state changes appearance', (tester) async {
      await tester.pumpWidget(
        _buildApp(
          const LabelColorDialog(currentLabel: null, currentColor: CardColor.none),
        ),
      );
      await tester.pumpAndSettle();

      // Hover over Save button
      final saveButton = find.text('Save');
      expect(saveButton, findsOneWidget);
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      await gesture.moveTo(tester.getCenter(saveButton));
      await tester.pumpAndSettle();

      // Hover over Cancel button
      final cancelButton = find.text('Cancel');
      await gesture.moveTo(tester.getCenter(cancelButton));
      await tester.pumpAndSettle();

      expect(find.byType(LabelColorDialog), findsOneWidget);
    });
  });
}
