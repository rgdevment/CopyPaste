import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/l10n/app_localizations.dart';
import 'package:copypaste/screens/permission_gate_screen.dart';
import 'package:copypaste/theme/compact_theme.dart';
import 'package:copypaste/theme/theme_provider.dart';

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

    testWidgets('_manualCheck success calls onGranted', (tester) async {
      _setMockHandler(channel, (call) async {
        if (call.method == 'requestAccessibility') return true;
        if (call.method == 'checkAccessibility') return false;
        return null;
      });

      var granted = false;

      await tester.pumpWidget(
        _wrap(
          PermissionGateScreen(
            previouslyGranted: true,
            onGranted: () => granted = true,
          ),
        ),
      );
      await tester.pump();

      expect(find.byType(OutlinedButton), findsOneWidget);

      await tester.tap(find.byType(OutlinedButton));
      await tester.pump();
      await tester.pump();

      expect(granted, isTrue);
    });

    testWidgets('onRestart button tap calls onRestart callback', (
      tester,
    ) async {
      var restarted = false;

      await tester.pumpWidget(
        _wrap(
          PermissionGateScreen(
            previouslyGranted: true,
            onGranted: () {},
            onRestart: () => restarted = true,
          ),
        ),
      );
      await tester.pump();

      await tester.tap(find.byType(TextButton));
      await tester.pump();

      expect(restarted, isTrue);
    });

    testWidgets('shows ... and disables button while checking', (tester) async {
      final completer = Completer<Object?>();
      _setMockHandler(channel, (call) async {
        if (call.method == 'requestAccessibility') return completer.future;
        if (call.method == 'checkAccessibility') return false;
        return null;
      });

      await tester.pumpWidget(
        _wrap(PermissionGateScreen(previouslyGranted: true, onGranted: () {})),
      );
      await tester.pump();

      await tester.tap(find.byType(OutlinedButton));
      await tester.pump();

      expect(find.text('...'), findsOneWidget);
      final btn = tester.widget<OutlinedButton>(find.byType(OutlinedButton));
      expect(btn.onPressed, isNull);

      completer.complete(false);
      await tester.pump();
      await tester.pump();

      expect(find.text('...'), findsNothing);
    });
  });
}
