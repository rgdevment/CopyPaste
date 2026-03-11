import 'dart:convert';
import 'dart:io';

import '../services/app_logger.dart';

const _sentinel = Object();

class AppConfig {
  const AppConfig({
    this.preferredLanguage = 'auto',
    this.runOnStartup = true,
    this.hotkeyUseCtrl = false,
    this.hotkeyUseWin = true,
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
    this.popupWidth = 368,
    this.popupHeight = 500,
    this.cardMinLines = 2,
    this.cardMaxLines = 5,
    this.hideOnDeactivate = true,
    this.resetScrollOnShow = true,
    this.resetSearchOnShow = true,
    this.hasSeenHint = false,
    this.themeMode = 'dark',
    this.showTrayIcon = true,
    this.accessibilityWasGranted = false,
  });

  factory AppConfig.fromJson(Map<String, dynamic> json) => AppConfig(
    preferredLanguage: json['preferredLanguage'] as String? ?? 'auto',
    runOnStartup: json['runOnStartup'] as bool? ?? true,
    hotkeyUseCtrl: json['hotkeyUseCtrl'] as bool? ?? false,
    hotkeyUseWin: json['hotkeyUseWin'] as bool? ?? true,
    hotkeyUseAlt: json['hotkeyUseAlt'] as bool? ?? true,
    hotkeyUseShift: json['hotkeyUseShift'] as bool? ?? false,
    hotkeyVirtualKey: json['hotkeyVirtualKey'] as int? ?? 0x56,
    hotkeyKeyName: json['hotkeyKeyName'] as String? ?? 'V',
    pageSize: json['pageSize'] as int? ?? 30,
    maxItemsBeforeCleanup: json['maxItemsBeforeCleanup'] as int? ?? 100,
    scrollLoadThreshold: json['scrollLoadThreshold'] as int? ?? 400,
    retentionDays: json['retentionDays'] as int? ?? 30,
    colorLabels:
        (json['colorLabels'] as Map<String, dynamic>?)?.map(
          (k, v) => MapEntry(k, v as String),
        ) ??
        const {},
    duplicateIgnoreWindowMs: json['duplicateIgnoreWindowMs'] as int? ?? 450,
    delayBeforeFocusMs: json['delayBeforeFocusMs'] as int? ?? 100,
    delayBeforePasteMs: json['delayBeforePasteMs'] as int? ?? 180,
    maxFocusVerifyAttempts: json['maxFocusVerifyAttempts'] as int? ?? 15,
    lastBackupDateUtc: json['lastBackupDateUtc'] != null
        ? DateTime.tryParse(json['lastBackupDateUtc'] as String)
        : null,
    popupWidth: json['popupWidth'] as int? ?? 368,
    popupHeight: json['popupHeight'] as int? ?? 500,
    cardMinLines: json['cardMinLines'] as int? ?? 2,
    cardMaxLines: json['cardMaxLines'] as int? ?? 5,
    hideOnDeactivate: json['hideOnDeactivate'] as bool? ?? true,
    resetScrollOnShow: json['resetScrollOnShow'] as bool? ?? true,
    resetSearchOnShow: json['resetSearchOnShow'] as bool? ?? true,
    hasSeenHint: json['hasSeenHint'] as bool? ?? false,
    themeMode: json['themeMode'] as String? ?? 'dark',
    showTrayIcon: json['showTrayIcon'] as bool? ?? true,
    accessibilityWasGranted: json['accessibilityWasGranted'] as bool? ?? false,
  );

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
  final bool accessibilityWasGranted;

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
    bool? accessibilityWasGranted,
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
    accessibilityWasGranted:
        accessibilityWasGranted ?? this.accessibilityWasGranted,
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
    'accessibilityWasGranted': accessibilityWasGranted,
  };

  static Future<AppConfig> load(String configPath) async {
    final file = File(configPath);
    if (!file.existsSync()) return const AppConfig();
    try {
      final json = jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
      return AppConfig.fromJson(json);
    } catch (e) {
      AppLogger.error('Failed to load config: $e');
      return const AppConfig();
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
