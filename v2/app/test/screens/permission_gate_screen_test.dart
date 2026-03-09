import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:app/l10n/app_localizations.dart';
import 'package:app/screens/permission_gate_screen.dart';
import 'package:app/theme/compact_theme.dart';
import 'package:app/theme/theme_provider.dart';

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

Widget _wrap(Widget child) {
  return MaterialApp(
    locale: const Locale('en'),
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    home: CopyPasteTheme(themeData: CompactTheme(), child: child),
  );
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

  group('PermissionGateScreen', () {
    testWidgets('renders app icon, title, and action buttons', (tester) async {
      await tester.pumpWidget(
        _wrap(PermissionGateScreen(previouslyGranted: false, onGranted: () {})),
      );
      await tester.pump();

      expect(find.text('CopyPaste'), findsOneWidget);
      expect(find.byType(FilledButton), findsOneWidget);
      expect(find.byIcon(Icons.lock_outline_rounded), findsOneWidget);
    });

    testWidgets('shows stale icon when previouslyGranted is true', (
      tester,
    ) async {
      await tester.pumpWidget(
        _wrap(PermissionGateScreen(previouslyGranted: true, onGranted: () {})),
      );
      await tester.pump();

      expect(find.byIcon(Icons.warning_amber_rounded), findsOneWidget);
    });

    testWidgets('open settings button calls openAccessibilitySettings', (
      tester,
    ) async {
      var opened = false;

      _setMockHandler(channel, (call) async {
        if (call.method == 'openAccessibilitySettings') opened = true;
        if (call.method == 'checkAccessibility') return false;
        return null;
      });

      await tester.pumpWidget(
        _wrap(PermissionGateScreen(previouslyGranted: false, onGranted: () {})),
      );
      await tester.pump();

      await tester.tap(find.byType(FilledButton));
      await tester.pump();

      expect(opened, isTrue);
    });

    testWidgets('poll timer calls onGranted when permission is detected', (
      tester,
    ) async {
      var checkCount = 0;
      var granted = false;

      _setMockHandler(channel, (call) async {
        if (call.method == 'checkAccessibility') {
          checkCount++;
          return checkCount >= 3;
        }
        return null;
      });

      await tester.pumpWidget(
        _wrap(
          PermissionGateScreen(
            previouslyGranted: false,
            onGranted: () => granted = true,
          ),
        ),
      );
      await tester.pump();

      // Advance past 3 poll ticks (1s each)
      for (var i = 0; i < 4; i++) {
        await tester.pump(const Duration(seconds: 1));
      }

      expect(granted, isTrue);
      expect(checkCount, greaterThanOrEqualTo(3));
    });

    testWidgets('shows check-again button after timeout', (tester) async {
      await tester.pumpWidget(
        _wrap(PermissionGateScreen(previouslyGranted: false, onGranted: () {})),
      );
      await tester.pump();

      // Should not have OutlinedButton initially
      expect(find.byType(OutlinedButton), findsNothing);

      // Advance past _maxPollsBeforeHint (30s) one tick at a time
      for (var i = 0; i < 31; i++) {
        await tester.pump(const Duration(seconds: 1));
      }

      expect(find.byType(OutlinedButton), findsOneWidget);
    });

    testWidgets('shows check-again button immediately when stale', (
      tester,
    ) async {
      await tester.pumpWidget(
        _wrap(PermissionGateScreen(previouslyGranted: true, onGranted: () {})),
      );
      await tester.pump();

      expect(find.byType(OutlinedButton), findsOneWidget);
    });

    testWidgets('dispose cancels timer without errors', (tester) async {
      await tester.pumpWidget(
        _wrap(PermissionGateScreen(previouslyGranted: false, onGranted: () {})),
      );
      await tester.pump();

      await tester.pumpWidget(_wrap(const SizedBox.shrink()));
      await tester.pump();

      expect(find.byType(PermissionGateScreen), findsNothing);
    });
  });
}
