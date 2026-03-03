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
        thumbnailWidth: 200,
      );
      final json = config.toJson();
      final restored = AppConfig.fromJson(json);
      expect(restored.preferredLanguage, equals('es-CL'));
      expect(restored.retentionDays, equals(60));
      expect(restored.pageSize, equals(50));
      expect(restored.hotkeyUseCtrl, isTrue);
      expect(restored.thumbnailWidth, equals(200));
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
}
