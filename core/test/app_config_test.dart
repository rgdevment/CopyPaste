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
      expect(config.hotkeyUseCtrl, isTrue);
      expect(config.hotkeyUseWin, isFalse);
      expect(config.hotkeyUseAlt, isFalse);
      expect(config.hotkeyUseShift, isTrue);
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
      expect(config.hotkeyUseCtrl, isTrue);
      expect(config.hotkeyUseWin, isFalse);
      expect(config.hotkeyUseAlt, isFalse);
      expect(config.hotkeyUseShift, isTrue);
      expect(config.hotkeyVirtualKey, equals(0x56));
      expect(config.hotkeyKeyName, equals('V'));
    });

    test('all platforms default to Ctrl+Shift+V', () {
      for (final platform in ['default', 'linux', 'windows', 'macos']) {
        final config = AppConfig.defaultForPlatform(platform);
        expect(config.hotkeyUseCtrl, isTrue, reason: '$platform: useCtrl');
        expect(config.hotkeyUseWin, isFalse, reason: '$platform: useWin');
        expect(config.hotkeyUseAlt, isFalse, reason: '$platform: useAlt');
        expect(config.hotkeyUseShift, isTrue, reason: '$platform: useShift');
        expect(config.hotkeyKeyName, equals('V'), reason: '$platform: key');
      }
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
    test('themeMode defaults to dark', () {
      const config = AppConfig();
      expect(config.themeMode, equals('dark'));
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
      expect(config.popupWidth, equals(380));
      expect(config.popupHeight, equals(500));
    });

    test('card line defaults', () {
      const config = AppConfig();
      expect(config.cardMinLines, equals(2));
      expect(config.cardMaxLines, equals(5));
    });

    test('hasSeenOnboarding defaults to false', () {
      const config = AppConfig();
      expect(config.hasSeenOnboarding, isFalse);
    });

    test('hasSeenOnboarding round-trips via JSON', () {
      const config = AppConfig(hasSeenOnboarding: true);
      expect(AppConfig.fromJson(config.toJson()).hasSeenOnboarding, isTrue);
    });

    test('hasSeenOnboarding absent in JSON defaults to false', () {
      expect(AppConfig.fromJson({}).hasSeenOnboarding, isFalse);
    });

    test('copyWith hasSeenOnboarding updates value', () {
      const config = AppConfig();
      expect(
        config.copyWith(hasSeenOnboarding: true).hasSeenOnboarding,
        isTrue,
      );
    });

    test('linux capability dismiss flags default to false', () {
      const config = AppConfig();
      expect(config.linuxAppindicatorWarningDismissed, isFalse);
      expect(config.linuxXtestWarningDismissed, isFalse);
    });

    test('linux capability dismiss flags round-trip via JSON', () {
      const config = AppConfig(
        linuxAppindicatorWarningDismissed: true,
        linuxXtestWarningDismissed: true,
      );
      final restored = AppConfig.fromJson(config.toJson());
      expect(restored.linuxAppindicatorWarningDismissed, isTrue);
      expect(restored.linuxXtestWarningDismissed, isTrue);
    });

    test('copyWith updates linux capability dismiss flags individually', () {
      const config = AppConfig();
      expect(
        config
            .copyWith(linuxAppindicatorWarningDismissed: true)
            .linuxAppindicatorWarningDismissed,
        isTrue,
      );
      expect(
        config
            .copyWith(linuxXtestWarningDismissed: true)
            .linuxXtestWarningDismissed,
        isTrue,
      );
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
        colorLabels: {'1': 'Work', '2': 'Home'},
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

  group('AppConfig edge cases', () {
    test('load returns default on corrupt file', () async {
      final dir = Directory.systemTemp.createTempSync('config_test_');
      final path = '${dir.path}/config.json';
      File(path).writeAsStringSync('{not valid json');
      final config = await AppConfig.load(path);
      expect(config, isA<AppConfig>());
      dir.deleteSync(recursive: true);
    });

    test('fromJson throws on wrong types', () {
      expect(
        () => AppConfig.fromJson({
          'preferredLanguage': 123,
          'runOnStartup': 'yes',
          'hotkeyUseCtrl': 'true',
          'colorLabels': 'not a map',
        }),
        throwsA(isA<TypeError>()),
      );
    });

    test('copyWith with all fields as null returns same values', () {
      const config = AppConfig();
      final updated = config.copyWith();
      expect(updated.preferredLanguage, config.preferredLanguage);
      expect(updated.runOnStartup, config.runOnStartup);
      expect(updated.hotkeyUseCtrl, config.hotkeyUseCtrl);
      expect(updated.themeMode, config.themeMode);
    });

    test('fromJson with empty string lastBackupDateUtc returns null', () {
      final config = AppConfig.fromJson({'lastBackupDateUtc': ''});
      expect(config.lastBackupDateUtc, isNull);
    });

    test(
      'fromJson with invalid date string for lastBackupDateUtc returns null',
      () {
        final config = AppConfig.fromJson({'lastBackupDateUtc': 'not-a-date'});
        expect(config.lastBackupDateUtc, isNull);
      },
    );

    test('toJson includes all fields when set', () {
      final config = AppConfig(
        preferredLanguage: 'fr',
        runOnStartup: false,
        hotkeyUseCtrl: false,
        hotkeyUseWin: true,
        hotkeyUseAlt: true,
        hotkeyUseShift: false,
        hotkeyVirtualKey: 0x41,
        hotkeyKeyName: 'A',
        pageSize: 99,
        maxItemsBeforeCleanup: 999,
        scrollLoadThreshold: 888,
        retentionDays: 77,
        colorLabels: {'x': 'y'},
        duplicateIgnoreWindowMs: 1,
        delayBeforeFocusMs: 2,
        delayBeforePasteMs: 3,
        maxFocusVerifyAttempts: 4,
        lastBackupDateUtc: DateTime.utc(2026, 1, 1),
        popupWidth: 111,
        popupHeight: 222,
        cardMinLines: 3,
        cardMaxLines: 4,
        hideOnDeactivate: false,
        resetScrollOnShow: false,
        resetSearchOnShow: false,
        hasSeenHint: true,
        themeMode: 'test',
        accessibilityWasGranted: true,
        lastRunVersion: 'v',
        hasSeenOnboarding: true,
      );
      final json = config.toJson();
      expect(json['preferredLanguage'], 'fr');
      expect(json['runOnStartup'], false);
      expect(json['hotkeyUseCtrl'], false);
      expect(json['hotkeyUseWin'], true);
      expect(json['hotkeyUseAlt'], true);
      expect(json['hotkeyUseShift'], false);
      expect(json['hotkeyVirtualKey'], 0x41);
      expect(json['hotkeyKeyName'], 'A');
      expect(json['pageSize'], 99);
      expect(json['maxItemsBeforeCleanup'], 999);
      expect(json['scrollLoadThreshold'], 888);
      expect(json['retentionDays'], 77);
      expect(json['colorLabels'], {'x': 'y'});
      expect(json['duplicateIgnoreWindowMs'], 1);
      expect(json['delayBeforeFocusMs'], 2);
      expect(json['delayBeforePasteMs'], 3);
      expect(json['maxFocusVerifyAttempts'], 4);
      expect(json['lastBackupDateUtc'], isA<String>());
      expect(json['popupWidth'], 111);
      expect(json['popupHeight'], 222);
      expect(json['cardMinLines'], 3);
      expect(json['cardMaxLines'], 4);
      expect(json['hideOnDeactivate'], false);
      expect(json['resetScrollOnShow'], false);
      expect(json['resetSearchOnShow'], false);
      expect(json['hasSeenHint'], true);
      expect(json['themeMode'], 'test');
      expect(json['accessibilityWasGranted'], true);
      expect(json['lastRunVersion'], 'v');
      expect(json['hasSeenOnboarding'], true);
    });
  });

  group('AppConfig lastRunVersion field', () {
    test('lastRunVersion defaults to empty string', () {
      const config = AppConfig();
      expect(config.lastRunVersion, equals(''));
    });

    test('lastRunVersion round-trips via JSON', () {
      const config = AppConfig(lastRunVersion: 'v2.2.2');
      expect(
        AppConfig.fromJson(config.toJson()).lastRunVersion,
        equals('v2.2.2'),
      );
    });

    test('lastRunVersion absent in JSON defaults to empty string', () {
      expect(AppConfig.fromJson({}).lastRunVersion, equals(''));
    });

    test('copyWith lastRunVersion updates value', () {
      const config = AppConfig();
      expect(
        config.copyWith(lastRunVersion: 'v2.0.0').lastRunVersion,
        equals('v2.0.0'),
      );
    });

    test('lastRunVersion preserved when copyWith changes other field', () {
      const config = AppConfig(lastRunVersion: 'v2.1.6');
      final updated = config.copyWith(pageSize: 50);
      expect(updated.lastRunVersion, equals('v2.1.6'));
    });
  });

  group('AppConfig accessibilityWasGranted field', () {
    test('accessibilityWasGranted defaults to false', () {
      const config = AppConfig();
      expect(config.accessibilityWasGranted, isFalse);
    });

    test('accessibilityWasGranted round-trips via JSON', () {
      const config = AppConfig(accessibilityWasGranted: true);
      expect(
        AppConfig.fromJson(config.toJson()).accessibilityWasGranted,
        isTrue,
      );
    });

    test('accessibilityWasGranted absent in JSON defaults to false', () {
      expect(AppConfig.fromJson({}).accessibilityWasGranted, isFalse);
    });

    test('copyWith accessibilityWasGranted updates value', () {
      const config = AppConfig();
      expect(
        config.copyWith(accessibilityWasGranted: true).accessibilityWasGranted,
        isTrue,
      );
    });

    test(
      'accessibilityWasGranted preserved when copyWith changes other field',
      () {
        const config = AppConfig(accessibilityWasGranted: true);
        expect(config.copyWith(pageSize: 20).accessibilityWasGranted, isTrue);
      },
    );
  });

  group('AppConfig behavior defaults', () {
    test('hideOnDeactivate defaults to true', () {
      const config = AppConfig();
      expect(config.hideOnDeactivate, isTrue);
    });

    test('resetScrollOnShow defaults to true', () {
      const config = AppConfig();
      expect(config.resetScrollOnShow, isTrue);
    });

    test('resetSearchOnShow defaults to true', () {
      const config = AppConfig();
      expect(config.resetSearchOnShow, isTrue);
    });

    test('hasSeenHint defaults to false', () {
      const config = AppConfig();
      expect(config.hasSeenHint, isFalse);
    });

    test('maxItemsBeforeCleanup defaults to 100', () {
      const config = AppConfig();
      expect(config.maxItemsBeforeCleanup, equals(100));
    });

    test('scrollLoadThreshold defaults to 400', () {
      const config = AppConfig();
      expect(config.scrollLoadThreshold, equals(400));
    });

    test('hideOnDeactivate round-trips via JSON', () {
      const config = AppConfig(hideOnDeactivate: false);
      expect(AppConfig.fromJson(config.toJson()).hideOnDeactivate, isFalse);
    });

    test('resetScrollOnShow round-trips via JSON', () {
      const config = AppConfig(resetScrollOnShow: false);
      expect(AppConfig.fromJson(config.toJson()).resetScrollOnShow, isFalse);
    });

    test('resetSearchOnShow round-trips via JSON', () {
      const config = AppConfig(resetSearchOnShow: false);
      expect(AppConfig.fromJson(config.toJson()).resetSearchOnShow, isFalse);
    });

    test('hasSeenHint round-trips via JSON', () {
      const config = AppConfig(hasSeenHint: true);
      expect(AppConfig.fromJson(config.toJson()).hasSeenHint, isTrue);
    });

    test('maxItemsBeforeCleanup round-trips via JSON', () {
      const config = AppConfig(maxItemsBeforeCleanup: 200);
      expect(
        AppConfig.fromJson(config.toJson()).maxItemsBeforeCleanup,
        equals(200),
      );
    });

    test('scrollLoadThreshold round-trips via JSON', () {
      const config = AppConfig(scrollLoadThreshold: 800);
      expect(
        AppConfig.fromJson(config.toJson()).scrollLoadThreshold,
        equals(800),
      );
    });

    test('copyWith behavior fields updates correctly', () {
      const config = AppConfig();
      final updated = config.copyWith(
        hideOnDeactivate: false,
        resetScrollOnShow: false,
        resetSearchOnShow: false,
        hasSeenHint: true,
        maxItemsBeforeCleanup: 50,
        scrollLoadThreshold: 200,
      );
      expect(updated.hideOnDeactivate, isFalse);
      expect(updated.resetScrollOnShow, isFalse);
      expect(updated.resetSearchOnShow, isFalse);
      expect(updated.hasSeenHint, isTrue);
      expect(updated.maxItemsBeforeCleanup, equals(50));
      expect(updated.scrollLoadThreshold, equals(200));
    });
  });

  group('AppConfig version and platform', () {
    test('appVersion is a non-empty String constant', () {
      expect(AppConfig.appVersion, isA<String>());
      expect(AppConfig.appVersion, isNotEmpty);
    });

    test('defaultForCurrentPlatform() returns an AppConfig instance', () {
      final config = AppConfig.defaultForCurrentPlatform();
      expect(config, isA<AppConfig>());
    });

    test(
      'defaultForPlatform returns same hotkey defaults for all platforms',
      () {
        final linux = AppConfig.defaultForPlatform('linux');
        final macos = AppConfig.defaultForPlatform('macos');
        final windows = AppConfig.defaultForPlatform('windows');

        expect(linux.hotkeyKeyName, equals(macos.hotkeyKeyName));
        expect(macos.hotkeyKeyName, equals(windows.hotkeyKeyName));
        expect(linux.hotkeyUseCtrl, equals(macos.hotkeyUseCtrl));
      },
    );

    test('defaultForPlatform unknown string returns default AppConfig', () {
      final config = AppConfig.defaultForPlatform('unknown-platform');
      expect(config, isA<AppConfig>());
      expect(config.preferredLanguage, equals('auto'));
    });

    test(
      'two calls to defaultForCurrentPlatform return equivalent configs',
      () {
        final a = AppConfig.defaultForCurrentPlatform();
        final b = AppConfig.defaultForCurrentPlatform();
        expect(a.preferredLanguage, equals(b.preferredLanguage));
        expect(a.themeMode, equals(b.themeMode));
        expect(a.retentionDays, equals(b.retentionDays));
      },
    );
  });

  group('AppConfig fromJson fallback to platform defaults', () {
    test('fromJson uses platform default for missing preferredLanguage', () {
      final defaults = AppConfig.defaultForCurrentPlatform();
      final config = AppConfig.fromJson({});
      expect(config.preferredLanguage, equals(defaults.preferredLanguage));
    });

    test('fromJson uses platform default for missing themeMode', () {
      final defaults = AppConfig.defaultForCurrentPlatform();
      final config = AppConfig.fromJson({});
      expect(config.themeMode, equals(defaults.themeMode));
    });

    test('fromJson explicit value overrides platform default', () {
      final config = AppConfig.fromJson({'themeMode': 'light'});
      expect(config.themeMode, equals('light'));
    });

    test('fromJson explicit false overrides default true for runOnStartup', () {
      final config = AppConfig.fromJson({'runOnStartup': false});
      expect(config.runOnStartup, isFalse);
    });
  });

  group('AppConfig PR #10 fields (thumbnails / onboarding / image cap)', () {
    test('default values', () {
      const c = AppConfig();
      expect(c.hasCompletedOnboarding, isFalse);
      expect(c.generateImageThumbnails, isTrue);
      expect(c.generateVideoThumbnails, isTrue);
      expect(c.generateAudioThumbnails, isTrue);
      expect(c.maxImageProcessingSizeMB, equals(25));
    });

    test('JSON round-trip preserves new fields', () {
      const c = AppConfig(
        hasCompletedOnboarding: true,
        generateImageThumbnails: false,
        generateVideoThumbnails: false,
        generateAudioThumbnails: false,
        maxImageProcessingSizeMB: 5,
      );
      final restored = AppConfig.fromJson(c.toJson());
      expect(restored.hasCompletedOnboarding, isTrue);
      expect(restored.generateImageThumbnails, isFalse);
      expect(restored.generateVideoThumbnails, isFalse);
      expect(restored.generateAudioThumbnails, isFalse);
      expect(restored.maxImageProcessingSizeMB, equals(5));
    });

    test('copyWith updates each new field independently', () {
      const c = AppConfig();
      final u = c.copyWith(
        hasCompletedOnboarding: true,
        generateImageThumbnails: false,
        maxImageProcessingSizeMB: 10,
      );
      expect(u.hasCompletedOnboarding, isTrue);
      expect(u.generateImageThumbnails, isFalse);
      expect(u.generateVideoThumbnails, isTrue); // unchanged
      expect(u.maxImageProcessingSizeMB, equals(10));
    });

    test(
      'hasCompletedOnboarding migrates from legacy hasSeenWindowsOnboarding',
      () {
        final c = AppConfig.fromJson({'hasSeenWindowsOnboarding': true});
        expect(c.hasCompletedOnboarding, isTrue);
      },
    );

    test('hasSeenOnboarding migrates from legacy hasSeenWindowsOnboarding', () {
      final c = AppConfig.fromJson({'hasSeenWindowsOnboarding': true});
      expect(c.hasSeenOnboarding, isTrue);
    });

    test('hasSeenOnboarding new key takes precedence over legacy', () {
      final c = AppConfig.fromJson({
        'hasSeenWindowsOnboarding': false,
        'hasSeenOnboarding': true,
      });
      expect(c.hasSeenOnboarding, isTrue);
    });

    test(
      'both hasSeenOnboarding and hasCompletedOnboarding populated from legacy',
      () {
        final c = AppConfig.fromJson({'hasSeenWindowsOnboarding': true});
        expect(c.hasSeenOnboarding, isTrue);
        expect(c.hasCompletedOnboarding, isTrue);
      },
    );

    test(
      'hasCompletedOnboarding stays false when neither legacy nor new is set',
      () {
        final c = AppConfig.fromJson({});
        expect(c.hasCompletedOnboarding, isFalse);
      },
    );

    test('explicit hasCompletedOnboarding overrides legacy', () {
      final c = AppConfig.fromJson({
        'hasSeenWindowsOnboarding': true,
        'hasCompletedOnboarding': false,
      });
      expect(c.hasCompletedOnboarding, isFalse);
    });
  });

  group('AppConfig PR #9 field (keepBrokenItemsDays)', () {
    test('default value is 30', () {
      const c = AppConfig();
      expect(c.keepBrokenItemsDays, equals(30));
    });

    test('JSON round-trip preserves keepBrokenItemsDays', () {
      const c = AppConfig(keepBrokenItemsDays: 7);
      final restored = AppConfig.fromJson(c.toJson());
      expect(restored.keepBrokenItemsDays, equals(7));
    });

    test('absent key in JSON falls back to default (30)', () {
      final c = AppConfig.fromJson({});
      expect(c.keepBrokenItemsDays, equals(30));
    });

    test('copyWith updates keepBrokenItemsDays independently', () {
      const c = AppConfig();
      final updated = c.copyWith(keepBrokenItemsDays: 14);
      expect(updated.keepBrokenItemsDays, equals(14));
      // Other fields unaffected
      expect(updated.retentionDays, equals(c.retentionDays));
    });
  });

  group('AppConfig PR #10b field (resetFiltersOnShow)', () {
    test('default value is true', () {
      const c = AppConfig();
      expect(c.resetFiltersOnShow, isTrue);
    });

    test('JSON round-trip preserves resetFiltersOnShow', () {
      const c = AppConfig(resetFiltersOnShow: false);
      final restored = AppConfig.fromJson(c.toJson());
      expect(restored.resetFiltersOnShow, isFalse);
    });

    test('absent key in JSON falls back to default (true)', () {
      final c = AppConfig.fromJson({});
      expect(c.resetFiltersOnShow, isTrue);
    });

    test('copyWith updates resetFiltersOnShow independently', () {
      const c = AppConfig();
      final updated = c.copyWith(resetFiltersOnShow: false);
      expect(updated.resetFiltersOnShow, isFalse);
      expect(updated.resetScrollOnShow, equals(c.resetScrollOnShow));
      expect(updated.resetSearchOnShow, equals(c.resetSearchOnShow));
    });
  });

  group('AppConfig PR #11 field (imagesQuotaMB)', () {
    test('default value is 0 (unlimited)', () {
      const c = AppConfig();
      expect(c.imagesQuotaMB, equals(0));
    });

    test('JSON round-trip preserves imagesQuotaMB', () {
      const c = AppConfig(imagesQuotaMB: 500);
      final restored = AppConfig.fromJson(c.toJson());
      expect(restored.imagesQuotaMB, equals(500));
    });

    test('absent key in JSON falls back to default (0)', () {
      final c = AppConfig.fromJson({});
      expect(c.imagesQuotaMB, equals(0));
    });

    test('copyWith updates imagesQuotaMB independently', () {
      const c = AppConfig();
      final updated = c.copyWith(imagesQuotaMB: 1024);
      expect(updated.imagesQuotaMB, equals(1024));
      // Other multimedia fields unaffected
      expect(
        updated.maxImageProcessingSizeMB,
        equals(c.maxImageProcessingSizeMB),
      );
    });
  });
}
