import 'dart:convert';
import 'dart:io';

import '../services/app_logger.dart';

const _sentinel = Object();

class AppConfig {
  const AppConfig({
    this.preferredLanguage = 'auto',
    this.runOnStartup = true,
    this.hotkeyUseCtrl = true,
    this.hotkeyUseWin = false,
    this.hotkeyUseAlt = true,
    this.hotkeyUseShift = false,
    this.hotkeyVirtualKey = 0x56,
    this.hotkeyKeyName = 'V',
    this.plainPasteHotkeyEnabled = false,
    this.plainPasteHotkeyUseCtrl = true,
    this.plainPasteHotkeyUseWin = false,
    this.plainPasteHotkeyUseAlt = true,
    this.plainPasteHotkeyUseShift = true,
    this.plainPasteHotkeyVirtualKey = 0x56,
    this.plainPasteHotkeyKeyName = 'V',
    this.pageSize = 30,
    this.maxItemsBeforeCleanup = 100,
    this.scrollLoadThreshold = 400,
    this.retentionDays = 30,
    this.keepBrokenItemsDays = 30,
    this.colorLabels = const {},
    this.duplicateIgnoreWindowMs = 450,
    this.delayBeforeFocusMs = 100,
    this.delayBeforePasteMs = 180,
    this.maxFocusVerifyAttempts = 15,
    this.lastBackupDateUtc,
    this.popupWidth = 380,
    this.popupHeight = 500,
    this.cardMinLines = 2,
    this.cardMaxLines = 5,
    this.hideOnDeactivate = true,
    this.resetScrollOnShow = true,
    this.resetSearchOnShow = true,
    this.resetFiltersOnShow = true,
    this.hasSeenHint = false,
    this.themeMode = 'dark',
    this.accessibilityWasGranted = false,
    this.lastRunVersion = '',
    this.hasSeenOnboarding = false,
    this.hasCompletedOnboarding = false,
    this.generateImageThumbnails = true,
    this.generateVideoThumbnails = true,
    this.generateAudioThumbnails = true,
    this.maxImageProcessingSizeMB = 25,
    this.imagesQuotaMB = 0,
    this.linuxAppindicatorWarningDismissed = false,
    this.linuxXtestWarningDismissed = false,
    this.rememberWindowPosition = false,
    this.lastWindowX,
    this.lastWindowY,
  });

  factory AppConfig.fromJson(Map<String, dynamic> json) {
    final defaults = defaultForCurrentPlatform();
    final hotkeyUseCtrl =
        json['hotkeyUseCtrl'] as bool? ?? defaults.hotkeyUseCtrl;
    final hotkeyUseWin = json['hotkeyUseWin'] as bool? ?? defaults.hotkeyUseWin;
    final hotkeyUseAlt = json['hotkeyUseAlt'] as bool? ?? defaults.hotkeyUseAlt;
    final hotkeyUseShift =
        json['hotkeyUseShift'] as bool? ?? defaults.hotkeyUseShift;
    var hotkeyVirtualKey =
        json['hotkeyVirtualKey'] as int? ?? defaults.hotkeyVirtualKey;
    var hotkeyKeyName =
        json['hotkeyKeyName'] as String? ?? defaults.hotkeyKeyName;
    var plainPasteHotkeyEnabled =
        // Missing means this is a pre-feature config. Do not unexpectedly
        // claim a new global shortcut for an existing user.
        json['plainPasteHotkeyEnabled'] as bool? ?? false;
    var plainPasteHotkeyUseCtrl =
        json['plainPasteHotkeyUseCtrl'] as bool? ??
        defaults.plainPasteHotkeyUseCtrl;
    var plainPasteHotkeyUseWin =
        json['plainPasteHotkeyUseWin'] as bool? ??
        defaults.plainPasteHotkeyUseWin;
    var plainPasteHotkeyUseAlt =
        json['plainPasteHotkeyUseAlt'] as bool? ??
        defaults.plainPasteHotkeyUseAlt;
    var plainPasteHotkeyUseShift =
        json['plainPasteHotkeyUseShift'] as bool? ??
        defaults.plainPasteHotkeyUseShift;
    var plainPasteHotkeyVirtualKey =
        json['plainPasteHotkeyVirtualKey'] as int? ??
        defaults.plainPasteHotkeyVirtualKey;
    var plainPasteHotkeyKeyName =
        json['plainPasteHotkeyKeyName'] as String? ??
        defaults.plainPasteHotkeyKeyName;

    final shortcutDefaultsVersion =
        json['shortcutDefaultsVersion'] as int? ?? 1;
    if (shortcutDefaultsVersion < 2) {
      final legacyPlainBinding =
          plainPasteHotkeyEnabled &&
          plainPasteHotkeyVirtualKey == 0x56 &&
          ((Platform.isMacOS &&
                  !plainPasteHotkeyUseCtrl &&
                  plainPasteHotkeyUseWin &&
                  plainPasteHotkeyUseAlt &&
                  plainPasteHotkeyUseShift) ||
              (!Platform.isMacOS &&
                  plainPasteHotkeyUseCtrl &&
                  !plainPasteHotkeyUseWin &&
                  !plainPasteHotkeyUseAlt &&
                  plainPasteHotkeyUseShift));
      if (legacyPlainBinding) {
        plainPasteHotkeyEnabled = false;
        plainPasteHotkeyUseCtrl = defaults.plainPasteHotkeyUseCtrl;
        plainPasteHotkeyUseWin = defaults.plainPasteHotkeyUseWin;
        plainPasteHotkeyUseAlt = defaults.plainPasteHotkeyUseAlt;
        plainPasteHotkeyUseShift = defaults.plainPasteHotkeyUseShift;
        plainPasteHotkeyVirtualKey = defaults.plainPasteHotkeyVirtualKey;
        plainPasteHotkeyKeyName = defaults.plainPasteHotkeyKeyName;
      }
      if (legacyPlainBinding) {
        AppLogger.info('Migrated the legacy plain-paste shortcut default');
      }
    }

    // Version 2 briefly changed the Windows opening default from Ctrl+Alt+C
    // to Ctrl+Alt+V. Revert only that exact automatic binding; version 1
    // custom bindings and every other versioned combination remain untouched.
    final versionTwoWindowsOpen =
        Platform.isWindows &&
        shortcutDefaultsVersion == 2 &&
        hotkeyUseCtrl &&
        !hotkeyUseWin &&
        hotkeyUseAlt &&
        !hotkeyUseShift &&
        hotkeyVirtualKey == 0x56;
    if (versionTwoWindowsOpen) {
      hotkeyVirtualKey = 0x43;
      hotkeyKeyName = 'C';
      AppLogger.info('Restored the Windows opening shortcut to Ctrl+Alt+C');
    }

    // Ctrl+Alt+Shift+V proved uncomfortable. Version 4 briefly tried the
    // application-level Ctrl+Shift+V convention, but a global registration
    // would shadow it in VS Code, terminals, and other apps. Migrate only
    // those two exact automatic Windows bindings to Ctrl+Alt+V; bindings from
    // versions where they could have been user-defined remain untouched.
    final legacyWindowsPlainPaste =
        Platform.isWindows &&
        plainPasteHotkeyUseCtrl &&
        !plainPasteHotkeyUseWin &&
        plainPasteHotkeyVirtualKey == 0x56 &&
        ((shortcutDefaultsVersion == 3 &&
                plainPasteHotkeyUseAlt &&
                plainPasteHotkeyUseShift) ||
            (shortcutDefaultsVersion == 4 &&
                !plainPasteHotkeyUseAlt &&
                plainPasteHotkeyUseShift));
    if (legacyWindowsPlainPaste) {
      plainPasteHotkeyUseAlt = true;
      plainPasteHotkeyUseShift = false;
      AppLogger.info('Updated the Windows plain-paste shortcut to Ctrl+Alt+V');
    }

    var duplicateIgnoreWindowMs =
        json['duplicateIgnoreWindowMs'] as int? ??
        defaults.duplicateIgnoreWindowMs;
    var delayBeforeFocusMs =
        json['delayBeforeFocusMs'] as int? ?? defaults.delayBeforeFocusMs;
    var delayBeforePasteMs =
        json['delayBeforePasteMs'] as int? ?? defaults.delayBeforePasteMs;
    var maxFocusVerifyAttempts =
        json['maxFocusVerifyAttempts'] as int? ??
        defaults.maxFocusVerifyAttempts;
    final storedPasteDefaultsVersion =
        json['pasteDefaultsVersion'] as int? ?? 1;

    // v2 moved Windows onto the Instant preset, assuming native focus
    // verification made fixed delays unnecessary. It does not: the check only
    // proves the destination is the active top-level window, so the paste can
    // still outrun apps that route keyboard focus internally. Undo it for
    // anyone left on those exact values; tuned tuples are preserved.
    final untouchedInstantPaste =
        Platform.isWindows &&
        storedPasteDefaultsVersion < pasteDefaultsVersion &&
        duplicateIgnoreWindowMs == 300 &&
        delayBeforeFocusMs == 0 &&
        delayBeforePasteMs == 20 &&
        maxFocusVerifyAttempts == 15;
    if (untouchedInstantPaste) {
      duplicateIgnoreWindowMs = 350;
      delayBeforeFocusMs = 80;
      delayBeforePasteMs = 120;
      maxFocusVerifyAttempts = 12;
      AppLogger.info('Updated Windows paste timing to the Normal preset');
    }

    return AppConfig(
      preferredLanguage:
          json['preferredLanguage'] as String? ?? defaults.preferredLanguage,
      runOnStartup: json['runOnStartup'] as bool? ?? defaults.runOnStartup,
      hotkeyUseCtrl: hotkeyUseCtrl,
      hotkeyUseWin: hotkeyUseWin,
      hotkeyUseAlt: hotkeyUseAlt,
      hotkeyUseShift: hotkeyUseShift,
      hotkeyVirtualKey: hotkeyVirtualKey,
      hotkeyKeyName: hotkeyKeyName,
      plainPasteHotkeyEnabled: plainPasteHotkeyEnabled,
      plainPasteHotkeyUseCtrl: plainPasteHotkeyUseCtrl,
      plainPasteHotkeyUseWin: plainPasteHotkeyUseWin,
      plainPasteHotkeyUseAlt: plainPasteHotkeyUseAlt,
      plainPasteHotkeyUseShift: plainPasteHotkeyUseShift,
      plainPasteHotkeyVirtualKey: plainPasteHotkeyVirtualKey,
      plainPasteHotkeyKeyName: plainPasteHotkeyKeyName,
      pageSize: json['pageSize'] as int? ?? defaults.pageSize,
      maxItemsBeforeCleanup:
          json['maxItemsBeforeCleanup'] as int? ??
          defaults.maxItemsBeforeCleanup,
      scrollLoadThreshold:
          json['scrollLoadThreshold'] as int? ?? defaults.scrollLoadThreshold,
      retentionDays: json['retentionDays'] as int? ?? defaults.retentionDays,
      keepBrokenItemsDays:
          json['keepBrokenItemsDays'] as int? ?? defaults.keepBrokenItemsDays,
      colorLabels:
          (json['colorLabels'] as Map<String, dynamic>?)?.map(
            (k, v) => MapEntry(k, v as String),
          ) ??
          const {},
      duplicateIgnoreWindowMs: duplicateIgnoreWindowMs,
      delayBeforeFocusMs: delayBeforeFocusMs,
      delayBeforePasteMs: delayBeforePasteMs,
      maxFocusVerifyAttempts: maxFocusVerifyAttempts,
      lastBackupDateUtc: json['lastBackupDateUtc'] != null
          ? DateTime.tryParse(json['lastBackupDateUtc'] as String)
          : null,
      popupWidth: json['popupWidth'] as int? ?? defaults.popupWidth,
      popupHeight: json['popupHeight'] as int? ?? defaults.popupHeight,
      cardMinLines: json['cardMinLines'] as int? ?? defaults.cardMinLines,
      cardMaxLines: json['cardMaxLines'] as int? ?? defaults.cardMaxLines,
      hideOnDeactivate:
          json['hideOnDeactivate'] as bool? ?? defaults.hideOnDeactivate,
      resetScrollOnShow:
          json['resetScrollOnShow'] as bool? ?? defaults.resetScrollOnShow,
      resetSearchOnShow:
          json['resetSearchOnShow'] as bool? ?? defaults.resetSearchOnShow,
      resetFiltersOnShow:
          json['resetFiltersOnShow'] as bool? ?? defaults.resetFiltersOnShow,
      hasSeenHint: json['hasSeenHint'] as bool? ?? defaults.hasSeenHint,
      themeMode: json['themeMode'] as String? ?? defaults.themeMode,
      accessibilityWasGranted:
          json['accessibilityWasGranted'] as bool? ??
          defaults.accessibilityWasGranted,
      lastRunVersion:
          json['lastRunVersion'] as String? ?? defaults.lastRunVersion,
      hasSeenOnboarding:
          json['hasSeenOnboarding'] as bool? ??
          json['hasSeenWindowsOnboarding'] as bool? ??
          defaults.hasSeenOnboarding,
      hasCompletedOnboarding:
          json['hasCompletedOnboarding'] as bool? ??
          (json['hasSeenOnboarding'] as bool? ??
              json['hasSeenWindowsOnboarding'] as bool? ??
              defaults.hasCompletedOnboarding),
      generateImageThumbnails:
          json['generateImageThumbnails'] as bool? ??
          defaults.generateImageThumbnails,
      generateVideoThumbnails:
          json['generateVideoThumbnails'] as bool? ??
          defaults.generateVideoThumbnails,
      generateAudioThumbnails:
          json['generateAudioThumbnails'] as bool? ??
          defaults.generateAudioThumbnails,
      maxImageProcessingSizeMB:
          json['maxImageProcessingSizeMB'] as int? ??
          defaults.maxImageProcessingSizeMB,
      imagesQuotaMB: json['imagesQuotaMB'] as int? ?? defaults.imagesQuotaMB,
      linuxAppindicatorWarningDismissed:
          json['linuxAppindicatorWarningDismissed'] as bool? ??
          defaults.linuxAppindicatorWarningDismissed,
      linuxXtestWarningDismissed:
          json['linuxXtestWarningDismissed'] as bool? ??
          defaults.linuxXtestWarningDismissed,
      rememberWindowPosition:
          json['rememberWindowPosition'] as bool? ??
          defaults.rememberWindowPosition,
      lastWindowX: (json['lastWindowX'] as num?)?.toDouble(),
      lastWindowY: (json['lastWindowY'] as num?)?.toDouble(),
    );
  }

  static const int shortcutDefaultsVersion = 5;
  static const int pasteDefaultsVersion = 3;

  static AppConfig defaultForCurrentPlatform() =>
      defaultForPlatform(Platform.operatingSystem);

  // Kept for tests that pass a platform string explicitly.
  static AppConfig defaultForPlatform(String platform) => switch (platform) {
    // Ctrl+Alt+C keeps the established CopyPaste opening gesture. The optional
    // system-wide plain-paste shortcut shares the modifiers for muscle memory
    // without shadowing the common application-level Ctrl+Shift+V gesture.
    'windows' => const AppConfig(
      hotkeyUseCtrl: true,
      hotkeyUseAlt: true,
      hotkeyUseShift: false,
      hotkeyVirtualKey: 0x43,
      hotkeyKeyName: 'C',
      plainPasteHotkeyEnabled: false,
      plainPasteHotkeyUseCtrl: true,
      plainPasteHotkeyUseAlt: true,
      plainPasteHotkeyUseShift: false,
      duplicateIgnoreWindowMs: 350,
      delayBeforeFocusMs: 80,
      delayBeforePasteMs: 120,
      maxFocusVerifyAttempts: 12,
    ),
    // Control+Shift+V opens the panel. The optional global plain-paste binding
    // includes every modifier and stays disabled until explicitly enabled.
    'macos' => const AppConfig(
      hotkeyUseCtrl: true,
      hotkeyUseAlt: false,
      hotkeyUseShift: true,
      plainPasteHotkeyEnabled: false,
      plainPasteHotkeyUseCtrl: true,
      plainPasteHotkeyUseWin: true,
      plainPasteHotkeyUseAlt: true,
      plainPasteHotkeyUseShift: true,
    ),
    // Super+V is the desktop-oriented history gesture. Ctrl+Shift+V remains
    // available to terminals because the optional global binding is disabled.
    'linux' => const AppConfig(
      hotkeyUseCtrl: false,
      hotkeyUseWin: true,
      hotkeyUseAlt: false,
      hotkeyUseShift: false,
      plainPasteHotkeyEnabled: false,
      plainPasteHotkeyUseCtrl: true,
      plainPasteHotkeyUseShift: true,
      plainPasteHotkeyUseAlt: false,
    ),
    _ => const AppConfig(),
  };

  static const String fileName = 'config.json';
  static const String appVersion = String.fromEnvironment(
    'APP_VERSION',
    defaultValue: '2.0.0',
  );

  // Language & Startup
  final String preferredLanguage;
  final bool runOnStartup;

  // Hotkey
  final bool hotkeyUseCtrl;
  final bool hotkeyUseWin;
  final bool hotkeyUseAlt;
  final bool hotkeyUseShift;
  final int hotkeyVirtualKey;
  final String hotkeyKeyName;
  final bool plainPasteHotkeyEnabled;
  final bool plainPasteHotkeyUseCtrl;
  final bool plainPasteHotkeyUseWin;
  final bool plainPasteHotkeyUseAlt;
  final bool plainPasteHotkeyUseShift;
  final int plainPasteHotkeyVirtualKey;
  final String plainPasteHotkeyKeyName;

  // Performance
  final int pageSize;
  final int maxItemsBeforeCleanup;
  final int scrollLoadThreshold;

  // Storage
  final int retentionDays;
  final int keepBrokenItemsDays;
  final Map<String, String> colorLabels;

  // Paste behavior
  final int duplicateIgnoreWindowMs;
  final int delayBeforeFocusMs;
  final int delayBeforePasteMs;
  final int maxFocusVerifyAttempts;

  // Backup
  final DateTime? lastBackupDateUtc;

  // Appearance
  final int popupWidth;
  final int popupHeight;
  final int cardMinLines;
  final int cardMaxLines;
  final bool hideOnDeactivate;
  final bool resetScrollOnShow;
  final bool resetSearchOnShow;
  final bool resetFiltersOnShow;
  final bool hasSeenHint;
  final String themeMode;
  final bool accessibilityWasGranted;
  final String lastRunVersion;
  final bool hasSeenOnboarding;
  final bool hasCompletedOnboarding;

  // Multimedia & thumbnails
  final bool generateImageThumbnails;
  final bool generateVideoThumbnails;
  final bool generateAudioThumbnails;
  final int maxImageProcessingSizeMB;

  // Storage quota (total bytes allowed under images/). 0 disables the cap;
  // anything > 0 triggers an LRU purge during the periodic cleanup until the
  // owned bytes drop back below the limit. Pinned items are never purged.
  final int imagesQuotaMB;

  // Linux capability warning banners (dismissible).
  final bool linuxAppindicatorWarningDismissed;
  final bool linuxXtestWarningDismissed;

  final bool rememberWindowPosition;
  final double? lastWindowX;
  final double? lastWindowY;

  AppConfig copyWith({
    String? preferredLanguage,
    bool? runOnStartup,
    bool? hotkeyUseCtrl,
    bool? hotkeyUseWin,
    bool? hotkeyUseAlt,
    bool? hotkeyUseShift,
    int? hotkeyVirtualKey,
    String? hotkeyKeyName,
    bool? plainPasteHotkeyEnabled,
    bool? plainPasteHotkeyUseCtrl,
    bool? plainPasteHotkeyUseWin,
    bool? plainPasteHotkeyUseAlt,
    bool? plainPasteHotkeyUseShift,
    int? plainPasteHotkeyVirtualKey,
    String? plainPasteHotkeyKeyName,
    int? pageSize,
    int? maxItemsBeforeCleanup,
    int? scrollLoadThreshold,
    int? retentionDays,
    int? keepBrokenItemsDays,
    Map<String, String>? colorLabels,
    int? duplicateIgnoreWindowMs,
    int? delayBeforeFocusMs,
    int? delayBeforePasteMs,
    int? maxFocusVerifyAttempts,
    Object? lastBackupDateUtc = _sentinel,
    int? popupWidth,
    int? popupHeight,
    int? cardMinLines,
    int? cardMaxLines,
    bool? hideOnDeactivate,
    bool? resetScrollOnShow,
    bool? resetSearchOnShow,
    bool? resetFiltersOnShow,
    bool? hasSeenHint,
    String? themeMode,
    bool? accessibilityWasGranted,
    String? lastRunVersion,
    bool? hasSeenOnboarding,
    bool? hasCompletedOnboarding,
    bool? generateImageThumbnails,
    bool? generateVideoThumbnails,
    bool? generateAudioThumbnails,
    int? maxImageProcessingSizeMB,
    int? imagesQuotaMB,
    bool? linuxAppindicatorWarningDismissed,
    bool? linuxXtestWarningDismissed,
    bool? rememberWindowPosition,
    Object? lastWindowX = _sentinel,
    Object? lastWindowY = _sentinel,
  }) => AppConfig(
    preferredLanguage: preferredLanguage ?? this.preferredLanguage,
    runOnStartup: runOnStartup ?? this.runOnStartup,
    hotkeyUseCtrl: hotkeyUseCtrl ?? this.hotkeyUseCtrl,
    hotkeyUseWin: hotkeyUseWin ?? this.hotkeyUseWin,
    hotkeyUseAlt: hotkeyUseAlt ?? this.hotkeyUseAlt,
    hotkeyUseShift: hotkeyUseShift ?? this.hotkeyUseShift,
    hotkeyVirtualKey: hotkeyVirtualKey ?? this.hotkeyVirtualKey,
    hotkeyKeyName: hotkeyKeyName ?? this.hotkeyKeyName,
    plainPasteHotkeyEnabled:
        plainPasteHotkeyEnabled ?? this.plainPasteHotkeyEnabled,
    plainPasteHotkeyUseCtrl:
        plainPasteHotkeyUseCtrl ?? this.plainPasteHotkeyUseCtrl,
    plainPasteHotkeyUseWin:
        plainPasteHotkeyUseWin ?? this.plainPasteHotkeyUseWin,
    plainPasteHotkeyUseAlt:
        plainPasteHotkeyUseAlt ?? this.plainPasteHotkeyUseAlt,
    plainPasteHotkeyUseShift:
        plainPasteHotkeyUseShift ?? this.plainPasteHotkeyUseShift,
    plainPasteHotkeyVirtualKey:
        plainPasteHotkeyVirtualKey ?? this.plainPasteHotkeyVirtualKey,
    plainPasteHotkeyKeyName:
        plainPasteHotkeyKeyName ?? this.plainPasteHotkeyKeyName,
    pageSize: pageSize ?? this.pageSize,
    maxItemsBeforeCleanup: maxItemsBeforeCleanup ?? this.maxItemsBeforeCleanup,
    scrollLoadThreshold: scrollLoadThreshold ?? this.scrollLoadThreshold,
    retentionDays: retentionDays ?? this.retentionDays,
    keepBrokenItemsDays: keepBrokenItemsDays ?? this.keepBrokenItemsDays,
    colorLabels: colorLabels ?? this.colorLabels,
    duplicateIgnoreWindowMs:
        duplicateIgnoreWindowMs ?? this.duplicateIgnoreWindowMs,
    delayBeforeFocusMs: delayBeforeFocusMs ?? this.delayBeforeFocusMs,
    delayBeforePasteMs: delayBeforePasteMs ?? this.delayBeforePasteMs,
    maxFocusVerifyAttempts:
        maxFocusVerifyAttempts ?? this.maxFocusVerifyAttempts,
    lastBackupDateUtc: lastBackupDateUtc == _sentinel
        ? this.lastBackupDateUtc
        : lastBackupDateUtc as DateTime?,
    popupWidth: popupWidth ?? this.popupWidth,
    popupHeight: popupHeight ?? this.popupHeight,
    cardMinLines: cardMinLines ?? this.cardMinLines,
    cardMaxLines: cardMaxLines ?? this.cardMaxLines,
    hideOnDeactivate: hideOnDeactivate ?? this.hideOnDeactivate,
    resetScrollOnShow: resetScrollOnShow ?? this.resetScrollOnShow,
    resetSearchOnShow: resetSearchOnShow ?? this.resetSearchOnShow,
    resetFiltersOnShow: resetFiltersOnShow ?? this.resetFiltersOnShow,
    hasSeenHint: hasSeenHint ?? this.hasSeenHint,
    themeMode: themeMode ?? this.themeMode,
    accessibilityWasGranted:
        accessibilityWasGranted ?? this.accessibilityWasGranted,
    lastRunVersion: lastRunVersion ?? this.lastRunVersion,
    hasSeenOnboarding: hasSeenOnboarding ?? this.hasSeenOnboarding,
    hasCompletedOnboarding:
        hasCompletedOnboarding ?? this.hasCompletedOnboarding,
    generateImageThumbnails:
        generateImageThumbnails ?? this.generateImageThumbnails,
    generateVideoThumbnails:
        generateVideoThumbnails ?? this.generateVideoThumbnails,
    generateAudioThumbnails:
        generateAudioThumbnails ?? this.generateAudioThumbnails,
    maxImageProcessingSizeMB:
        maxImageProcessingSizeMB ?? this.maxImageProcessingSizeMB,
    imagesQuotaMB: imagesQuotaMB ?? this.imagesQuotaMB,
    linuxAppindicatorWarningDismissed:
        linuxAppindicatorWarningDismissed ??
        this.linuxAppindicatorWarningDismissed,
    linuxXtestWarningDismissed:
        linuxXtestWarningDismissed ?? this.linuxXtestWarningDismissed,
    rememberWindowPosition:
        rememberWindowPosition ?? this.rememberWindowPosition,
    lastWindowX: lastWindowX == _sentinel
        ? this.lastWindowX
        : lastWindowX as double?,
    lastWindowY: lastWindowY == _sentinel
        ? this.lastWindowY
        : lastWindowY as double?,
  );

  Map<String, dynamic> toJson() => {
    'shortcutDefaultsVersion': shortcutDefaultsVersion,
    'pasteDefaultsVersion': pasteDefaultsVersion,
    'preferredLanguage': preferredLanguage,
    'runOnStartup': runOnStartup,
    'hotkeyUseCtrl': hotkeyUseCtrl,
    'hotkeyUseWin': hotkeyUseWin,
    'hotkeyUseAlt': hotkeyUseAlt,
    'hotkeyUseShift': hotkeyUseShift,
    'hotkeyVirtualKey': hotkeyVirtualKey,
    'hotkeyKeyName': hotkeyKeyName,
    'plainPasteHotkeyEnabled': plainPasteHotkeyEnabled,
    'plainPasteHotkeyUseCtrl': plainPasteHotkeyUseCtrl,
    'plainPasteHotkeyUseWin': plainPasteHotkeyUseWin,
    'plainPasteHotkeyUseAlt': plainPasteHotkeyUseAlt,
    'plainPasteHotkeyUseShift': plainPasteHotkeyUseShift,
    'plainPasteHotkeyVirtualKey': plainPasteHotkeyVirtualKey,
    'plainPasteHotkeyKeyName': plainPasteHotkeyKeyName,
    'pageSize': pageSize,
    'maxItemsBeforeCleanup': maxItemsBeforeCleanup,
    'scrollLoadThreshold': scrollLoadThreshold,
    'retentionDays': retentionDays,
    'keepBrokenItemsDays': keepBrokenItemsDays,
    'colorLabels': colorLabels,
    'duplicateIgnoreWindowMs': duplicateIgnoreWindowMs,
    'delayBeforeFocusMs': delayBeforeFocusMs,
    'delayBeforePasteMs': delayBeforePasteMs,
    'maxFocusVerifyAttempts': maxFocusVerifyAttempts,
    if (lastBackupDateUtc != null)
      'lastBackupDateUtc': lastBackupDateUtc!.toIso8601String(),
    'popupWidth': popupWidth,
    'popupHeight': popupHeight,
    'cardMinLines': cardMinLines,
    'cardMaxLines': cardMaxLines,
    'hideOnDeactivate': hideOnDeactivate,
    'resetScrollOnShow': resetScrollOnShow,
    'resetSearchOnShow': resetSearchOnShow,
    'resetFiltersOnShow': resetFiltersOnShow,
    'hasSeenHint': hasSeenHint,
    'themeMode': themeMode,
    'accessibilityWasGranted': accessibilityWasGranted,
    'lastRunVersion': lastRunVersion,
    'hasSeenOnboarding': hasSeenOnboarding,
    'hasCompletedOnboarding': hasCompletedOnboarding,
    'generateImageThumbnails': generateImageThumbnails,
    'generateVideoThumbnails': generateVideoThumbnails,
    'generateAudioThumbnails': generateAudioThumbnails,
    'maxImageProcessingSizeMB': maxImageProcessingSizeMB,
    'imagesQuotaMB': imagesQuotaMB,
    'linuxAppindicatorWarningDismissed': linuxAppindicatorWarningDismissed,
    'linuxXtestWarningDismissed': linuxXtestWarningDismissed,
    'rememberWindowPosition': rememberWindowPosition,
    if (lastWindowX != null) 'lastWindowX': lastWindowX,
    if (lastWindowY != null) 'lastWindowY': lastWindowY,
  };

  static Future<AppConfig> load(String configPath) async {
    final file = File(configPath);
    if (!file.existsSync()) return AppConfig.defaultForCurrentPlatform();
    try {
      final json = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
      final config = AppConfig.fromJson(json);
      final storedShortcutVersion =
          json['shortcutDefaultsVersion'] as int? ?? 1;
      final storedPasteVersion = json['pasteDefaultsVersion'] as int? ?? 1;
      if (storedShortcutVersion < shortcutDefaultsVersion ||
          storedPasteVersion < pasteDefaultsVersion) {
        try {
          await config.save(configPath);
        } catch (e) {
          AppLogger.warn('Failed to persist config migration: $e');
        }
      }
      return config;
    } catch (e) {
      AppLogger.error('Failed to load config: $e');
      return AppConfig.defaultForCurrentPlatform();
    }
  }

  Future<void> save(String configPath) async {
    final file = File(configPath);
    await file.create(recursive: true);
    await file.writeAsString(
      const JsonEncoder.withIndent('  ').convert(toJson()),
    );
  }
}
