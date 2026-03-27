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
    this.pageSize = 30,
    this.maxItemsBeforeCleanup = 100,
    this.scrollLoadThreshold = 400,
    this.retentionDays = 30,
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
    this.hasSeenHint = false,
    this.themeMode = 'auto',
    this.showTrayIcon = true,
    this.showInTaskbar = false,
    this.accessibilityWasGranted = false,
    this.lastRunVersion = '',
  });

  factory AppConfig.fromJson(Map<String, dynamic> json) {
    final defaults = defaultForCurrentPlatform();
    return AppConfig(
      preferredLanguage:
          json['preferredLanguage'] as String? ?? defaults.preferredLanguage,
      runOnStartup: json['runOnStartup'] as bool? ?? defaults.runOnStartup,
      hotkeyUseCtrl: json['hotkeyUseCtrl'] as bool? ?? defaults.hotkeyUseCtrl,
      hotkeyUseWin: json['hotkeyUseWin'] as bool? ?? defaults.hotkeyUseWin,
      hotkeyUseAlt: json['hotkeyUseAlt'] as bool? ?? defaults.hotkeyUseAlt,
      hotkeyUseShift:
          json['hotkeyUseShift'] as bool? ?? defaults.hotkeyUseShift,
      hotkeyVirtualKey:
          json['hotkeyVirtualKey'] as int? ?? defaults.hotkeyVirtualKey,
      hotkeyKeyName: json['hotkeyKeyName'] as String? ?? defaults.hotkeyKeyName,
      pageSize: json['pageSize'] as int? ?? defaults.pageSize,
      maxItemsBeforeCleanup:
          json['maxItemsBeforeCleanup'] as int? ??
          defaults.maxItemsBeforeCleanup,
      scrollLoadThreshold:
          json['scrollLoadThreshold'] as int? ?? defaults.scrollLoadThreshold,
      retentionDays: json['retentionDays'] as int? ?? defaults.retentionDays,
      colorLabels:
          (json['colorLabels'] as Map<String, dynamic>?)?.map(
            (k, v) => MapEntry(k, v as String),
          ) ??
          const {},
      duplicateIgnoreWindowMs:
          json['duplicateIgnoreWindowMs'] as int? ??
          defaults.duplicateIgnoreWindowMs,
      delayBeforeFocusMs:
          json['delayBeforeFocusMs'] as int? ?? defaults.delayBeforeFocusMs,
      delayBeforePasteMs:
          json['delayBeforePasteMs'] as int? ?? defaults.delayBeforePasteMs,
      maxFocusVerifyAttempts:
          json['maxFocusVerifyAttempts'] as int? ??
          defaults.maxFocusVerifyAttempts,
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
      hasSeenHint: json['hasSeenHint'] as bool? ?? defaults.hasSeenHint,
      themeMode: json['themeMode'] as String? ?? defaults.themeMode,
      showTrayIcon: json['showTrayIcon'] as bool? ?? defaults.showTrayIcon,
      showInTaskbar: json['showInTaskbar'] as bool? ?? defaults.showInTaskbar,
      accessibilityWasGranted:
          json['accessibilityWasGranted'] as bool? ??
          defaults.accessibilityWasGranted,
      lastRunVersion:
          json['lastRunVersion'] as String? ?? defaults.lastRunVersion,
    );
  }

  static AppConfig defaultForCurrentPlatform() => defaultForPlatform('default');

  // Kept for tests that pass a platform string explicitly.
  static AppConfig defaultForPlatform(String platform) => const AppConfig();

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

  // Performance
  final int pageSize;
  final int maxItemsBeforeCleanup;
  final int scrollLoadThreshold;

  // Storage
  final int retentionDays;
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
  final bool hasSeenHint;
  final String themeMode;
  final bool showTrayIcon;
  final bool showInTaskbar;
  final bool accessibilityWasGranted;
  final String lastRunVersion;

  AppConfig copyWith({
    String? preferredLanguage,
    bool? runOnStartup,
    bool? hotkeyUseCtrl,
    bool? hotkeyUseWin,
    bool? hotkeyUseAlt,
    bool? hotkeyUseShift,
    int? hotkeyVirtualKey,
    String? hotkeyKeyName,
    int? pageSize,
    int? maxItemsBeforeCleanup,
    int? scrollLoadThreshold,
    int? retentionDays,
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
    bool? hasSeenHint,
    String? themeMode,
    bool? showTrayIcon,
    bool? showInTaskbar,
    bool? accessibilityWasGranted,
    String? lastRunVersion,
  }) => AppConfig(
    preferredLanguage: preferredLanguage ?? this.preferredLanguage,
    runOnStartup: runOnStartup ?? this.runOnStartup,
    hotkeyUseCtrl: hotkeyUseCtrl ?? this.hotkeyUseCtrl,
    hotkeyUseWin: hotkeyUseWin ?? this.hotkeyUseWin,
    hotkeyUseAlt: hotkeyUseAlt ?? this.hotkeyUseAlt,
    hotkeyUseShift: hotkeyUseShift ?? this.hotkeyUseShift,
    hotkeyVirtualKey: hotkeyVirtualKey ?? this.hotkeyVirtualKey,
    hotkeyKeyName: hotkeyKeyName ?? this.hotkeyKeyName,
    pageSize: pageSize ?? this.pageSize,
    maxItemsBeforeCleanup: maxItemsBeforeCleanup ?? this.maxItemsBeforeCleanup,
    scrollLoadThreshold: scrollLoadThreshold ?? this.scrollLoadThreshold,
    retentionDays: retentionDays ?? this.retentionDays,
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
    hasSeenHint: hasSeenHint ?? this.hasSeenHint,
    themeMode: themeMode ?? this.themeMode,
    showTrayIcon: showTrayIcon ?? this.showTrayIcon,
    showInTaskbar: showInTaskbar ?? this.showInTaskbar,
    accessibilityWasGranted:
        accessibilityWasGranted ?? this.accessibilityWasGranted,
    lastRunVersion: lastRunVersion ?? this.lastRunVersion,
  );

  Map<String, dynamic> toJson() => {
    'preferredLanguage': preferredLanguage,
    'runOnStartup': runOnStartup,
    'hotkeyUseCtrl': hotkeyUseCtrl,
    'hotkeyUseWin': hotkeyUseWin,
    'hotkeyUseAlt': hotkeyUseAlt,
    'hotkeyUseShift': hotkeyUseShift,
    'hotkeyVirtualKey': hotkeyVirtualKey,
    'hotkeyKeyName': hotkeyKeyName,
    'pageSize': pageSize,
    'maxItemsBeforeCleanup': maxItemsBeforeCleanup,
    'scrollLoadThreshold': scrollLoadThreshold,
    'retentionDays': retentionDays,
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
    'hasSeenHint': hasSeenHint,
    'themeMode': themeMode,
    'showTrayIcon': showTrayIcon,
    'showInTaskbar': showInTaskbar,
    'accessibilityWasGranted': accessibilityWasGranted,
    'lastRunVersion': lastRunVersion,
  };

  static Future<AppConfig> load(String configPath) async {
    final file = File(configPath);
    if (!file.existsSync()) return AppConfig.defaultForCurrentPlatform();
    try {
      final json = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
      return AppConfig.fromJson(json);
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
