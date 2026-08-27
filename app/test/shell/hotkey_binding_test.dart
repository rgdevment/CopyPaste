import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/hotkey_binding.dart';

HotkeyBinding _binding({
  int virtualKey = 0x56,
  String keyName = 'V',
  bool useCtrl = false,
  bool useWin = false,
  bool useAlt = false,
  bool useShift = false,
}) => HotkeyBinding(
  virtualKey: virtualKey,
  keyName: keyName,
  useCtrl: useCtrl,
  useWin: useWin,
  useAlt: useAlt,
  useShift: useShift,
);

void main() {
  group('HotkeyBinding.label', () {
    test('macOS order is Control, Option, Shift, Command, key', () {
      final label = _binding(
        useCtrl: true,
        useWin: true,
        useAlt: true,
        useShift: true,
      ).label(isMac: true);
      expect(label, equals('Control+Option+Shift+Command+V'));
    });

    test('desktop order is Ctrl, Win, Alt, Shift, key', () {
      if (Platform.isMacOS) return;
      final label = _binding(
        useCtrl: true,
        useWin: true,
        useAlt: true,
        useShift: true,
      ).label();
      expect(label, equals('Ctrl+Win+Alt+Shift+V'));
    });

    test('omits unset modifiers', () {
      if (Platform.isMacOS) return;
      expect(
        _binding(useCtrl: true, useAlt: true).label(),
        equals('Ctrl+Alt+V'),
      );
      expect(_binding(useWin: true).label(), equals('Win+V'));
      expect(_binding(keyName: 'C').label(), equals('C'));
    });

    test('meta renders as Command on macOS and Win elsewhere', () {
      expect(_binding(useWin: true).label(isMac: true), equals('Command+V'));
      if (!Platform.isMacOS) {
        expect(_binding(useWin: true).label(), equals('Win+V'));
      }
    });
  });

  group('HotkeyBinding equality', () {
    test('identical field sets are equal and share a hashCode', () {
      final a = _binding(useCtrl: true, useShift: true);
      final b = _binding(useCtrl: true, useShift: true);
      expect(a, equals(b));
      expect(a.hashCode, equals(b.hashCode));
      expect(identical(a, a) && a == a, isTrue);
    });

    test('any differing field breaks equality', () {
      final base = _binding(useCtrl: true);
      expect(base, isNot(equals(_binding(useCtrl: true, virtualKey: 0x43))));
      expect(base, isNot(equals(_binding(useCtrl: true, keyName: 'C'))));
      expect(base, isNot(equals(_binding())));
      expect(base, isNot(equals(_binding(useCtrl: true, useWin: true))));
      expect(base, isNot(equals(_binding(useCtrl: true, useAlt: true))));
      expect(base, isNot(equals(_binding(useCtrl: true, useShift: true))));
    });

    test('is not equal to a different type', () {
      expect(_binding(), isNot(equals('V')));
    });
  });

  group('HotkeyRegistrationResult', () {
    test('carries the requested binding and an optional effective one', () {
      final requested = _binding(useWin: true);
      final effective = _binding(useCtrl: true);
      const failed = HotkeyRegistrationStatus.failed;

      final result = HotkeyRegistrationResult(
        status: failed,
        requestedBinding: requested,
      );
      expect(result.status, equals(failed));
      expect(result.requestedBinding, equals(requested));
      expect(result.effectiveBinding, isNull);

      final fallback = HotkeyRegistrationResult(
        status: HotkeyRegistrationStatus.fallbackRegistered,
        requestedBinding: requested,
        effectiveBinding: effective,
      );
      expect(fallback.effectiveBinding, equals(effective));
    });
  });
}
