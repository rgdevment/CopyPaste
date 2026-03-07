import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  group('AppConfig', () {
    test('default values are correct', () {
      const config = AppConfig();
      expect(config.preferredLanguage, equals('auto'));
      expect(config.runOnStartup, isTrue);
      expect(config.retentionDays, equals(30));
      expect(config.pageSize, equals(30));
      expect(config.hotkeyUseWin, isTrue);
      expect(config.hotkeyUseAlt, isTrue);
      expect(config.hotkeyKeyName, equals('V'));
    });

    test('serialization round-trip preserves all fields', () {
      const config = AppConfig(
        preferredLanguage: 'es-CL',
        retentionDays: 60,
        pageSize: 50,
        hotkeyUseCtrl: true,
      );
      final json = config.toJson();
      final restored = AppConfig.fromJson(json);
      expect(restored.preferredLanguage, equals('es-CL'));
      expect(restored.retentionDays, equals(60));
      expect(restored.pageSize, equals(50));
      expect(restored.hotkeyUseCtrl, isTrue);
    });

    test('copyWith produces new instance with changes', () {
      const config = AppConfig();
      final updated = config.copyWith(retentionDays: 90, pageSize: 20);
      expect(updated.retentionDays, equals(90));
      expect(updated.pageSize, equals(20));
      expect(config.retentionDays, equals(30));
    });

    test('fromJson uses defaults for missing fields', () {
      final config = AppConfig.fromJson({});
      expect(config.preferredLanguage, equals('auto'));
      expect(config.runOnStartup, isTrue);
    });

    test('load returns default when file does not exist', () async {
      final config = await AppConfig.load('/nonexistent/path/config.json');
      expect(config.preferredLanguage, equals('auto'));
    });

    test('save and load round-trip', () async {
      final dir = Directory.systemTemp.createTempSync('config_test_');
      final path = '${dir.path}/config.json';
      try {
        const original = AppConfig(
          preferredLanguage: 'es-CL',
          pageSize: 50,
          retentionDays: 90,
        );
        await original.save(path);
        final loaded = await AppConfig.load(path);
        expect(loaded.preferredLanguage, equals('es-CL'));
        expect(loaded.pageSize, equals(50));
        expect(loaded.retentionDays, equals(90));
      } finally {
        dir.deleteSync(recursive: true);
      }
    });

    test('lastBackupDateUtc serializes correctly', () {
      final date = DateTime.utc(2026, 3, 1, 12);
      final config = AppConfig(lastBackupDateUtc: date);
      final json = config.toJson();
      final restored = AppConfig.fromJson(json);
      expect(restored.lastBackupDateUtc, equals(date));
    });
  });

  group('AppConfig.copyWith sentinel fields', () {
    test('copyWith can clear lastBackupDateUtc to null', () {
      final date = DateTime.utc(2026, 1, 1);
      final config = AppConfig(lastBackupDateUtc: date);
      final updated = config.copyWith(lastBackupDateUtc: null);
      expect(updated.lastBackupDateUtc, isNull);
    });

    test('copyWith without lastBackupDateUtc preserves existing value', () {
      final date = DateTime.utc(2026, 1, 1);
      final config = AppConfig(lastBackupDateUtc: date);
      final updated = config.copyWith(pageSize: 20);
      expect(updated.lastBackupDateUtc, equals(date));
    });
  });

  group('AppConfig hotkey fields', () {
    test('hotkey defaults are correct', () {
      const config = AppConfig();
      expect(config.hotkeyUseCtrl, isFalse);
      expect(config.hotkeyUseWin, isTrue);
      expect(config.hotkeyUseAlt, isTrue);
      expect(config.hotkeyUseShift, isFalse);
      expect(config.hotkeyVirtualKey, equals(0x56));
      expect(config.hotkeyKeyName, equals('V'));
    });

    test('copyWith all hotkey fields', () {
      const config = AppConfig();
      final updated = config.copyWith(
        hotkeyUseCtrl: true,
        hotkeyUseWin: false,
        hotkeyUseAlt: false,
        hotkeyUseShift: true,
        hotkeyVirtualKey: 0x43,
        hotkeyKeyName: 'C',
      );
      expect(updated.hotkeyUseCtrl, isTrue);
      expect(updated.hotkeyUseWin, isFalse);
      expect(updated.hotkeyUseAlt, isFalse);
      expect(updated.hotkeyUseShift, isTrue);
      expect(updated.hotkeyVirtualKey, equals(0x43));
      expect(updated.hotkeyKeyName, equals('C'));
    });

    test('hotkey fields round-trip via JSON', () {
      const config = AppConfig(
        hotkeyUseCtrl: true,
        hotkeyUseWin: false,
        hotkeyUseAlt: false,
        hotkeyUseShift: true,
        hotkeyVirtualKey: 0x43,
        hotkeyKeyName: 'C',
      );
      final restored = AppConfig.fromJson(config.toJson());
      expect(restored.hotkeyUseCtrl, isTrue);
      expect(restored.hotkeyUseWin, isFalse);
      expect(restored.hotkeyKeyName, equals('C'));
    });
  });

  group('AppConfig appearance and behavior fields', () {
    test('themeMode defaults to auto', () {
      const config = AppConfig();
      expect(config.themeMode, equals('auto'));
    });

    test('themeMode round-trips via JSON', () {
      const config = AppConfig(themeMode: 'dark');
      expect(AppConfig.fromJson(config.toJson()).themeMode, equals('dark'));
    });

    test('colorLabels round-trip', () {
      const config = AppConfig(colorLabels: {'1': 'Work', '2': 'Home'});
      final restored = AppConfig.fromJson(config.toJson());
      expect(restored.colorLabels['1'], equals('Work'));
      expect(restored.colorLabels['2'], equals('Home'));
    });

    test('colorLabels defaults to empty map', () {
      const config = AppConfig();
      expect(config.colorLabels, isEmpty);
    });

    test('timing fields have correct defaults', () {
      const config = AppConfig();
      expect(config.duplicateIgnoreWindowMs, equals(450));
      expect(config.delayBeforeFocusMs, equals(100));
      expect(config.delayBeforePasteMs, equals(180));
      expect(config.maxFocusVerifyAttempts, equals(15));
    });

    test('popup size defaults', () {
      const config = AppConfig();
      expect(config.popupWidth, equals(368));
      expect(config.popupHeight, equals(500));
    });

    test('card line defaults', () {
      const config = AppConfig();
      expect(config.cardMinLines, equals(2));
      expect(config.cardMaxLines, equals(5));
    });

    test('toJson omits lastBackupDateUtc when null', () {
      const config = AppConfig();
      expect(config.toJson().containsKey('lastBackupDateUtc'), isFalse);
    });

    test('toJson includes lastBackupDateUtc when set', () {
      final config = AppConfig(lastBackupDateUtc: DateTime.utc(2026, 3, 5));
      expect(config.toJson()['lastBackupDateUtc'], isA<String>());
    });

    test('full serialization round-trip with all fields', () {
      const config = AppConfig(
        preferredLanguage: 'es',
        runOnStartup: false,
        hotkeyUseCtrl: true,
        pageSize: 50,
        retentionDays: 60,
        colorLabels: {'1': 'Work'},
        duplicateIgnoreWindowMs: 600,
        delayBeforeFocusMs: 120,
        delayBeforePasteMs: 200,
        maxFocusVerifyAttempts: 20,
        popupWidth: 400,
        popupHeight: 520,
        cardMinLines: 3,
        cardMaxLines: 8,
        hideOnDeactivate: false,
        resetScrollOnShow: false,
        resetSearchOnShow: false,
        hasSeenHint: true,
        themeMode: 'light',
      );
      final restored = AppConfig.fromJson(config.toJson());
      expect(restored.preferredLanguage, equals('es'));
      expect(restored.runOnStartup, isFalse);
      expect(restored.hotkeyUseCtrl, isTrue);
      expect(restored.pageSize, equals(50));
      expect(restored.retentionDays, equals(60));
      expect(restored.colorLabels['1'], equals('Work'));
      expect(restored.duplicateIgnoreWindowMs, equals(600));
      expect(restored.popupWidth, equals(400));
      expect(restored.cardMinLines, equals(3));
      expect(restored.hideOnDeactivate, isFalse);
      expect(restored.hasSeenHint, isTrue);
      expect(restored.themeMode, equals('light'));
    });
  });
}
