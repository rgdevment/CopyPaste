import 'dart:convert';
import 'dart:io';

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
    this.retentionDays = 30,
    this.colorLabels = const {},
    this.duplicateIgnoreWindowMs = 450,
    this.delayBeforeFocusMs = 100,
    this.delayBeforePasteMs = 180,
    this.maxFocusVerifyAttempts = 15,
    this.thumbnailWidth = 170,
    this.thumbnailQuality = 80,
    this.lastBackupDateUtc,
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
        retentionDays: json['retentionDays'] as int? ?? 30,
        colorLabels:
            (json['colorLabels'] as Map<String, dynamic>?)
                    ?.map((k, v) => MapEntry(k, v as String)) ??
                const {},
        duplicateIgnoreWindowMs:
            json['duplicateIgnoreWindowMs'] as int? ?? 450,
        delayBeforeFocusMs: json['delayBeforeFocusMs'] as int? ?? 100,
        delayBeforePasteMs: json['delayBeforePasteMs'] as int? ?? 180,
        maxFocusVerifyAttempts:
            json['maxFocusVerifyAttempts'] as int? ?? 15,
        thumbnailWidth: json['thumbnailWidth'] as int? ?? 170,
        thumbnailQuality: json['thumbnailQuality'] as int? ?? 80,
        lastBackupDateUtc: json['lastBackupDateUtc'] != null
            ? DateTime.tryParse(json['lastBackupDateUtc'] as String)
            : null,
      );

  static const String fileName = 'config.json';

  final String preferredLanguage;
  final bool runOnStartup;
  final bool hotkeyUseCtrl;
  final bool hotkeyUseWin;
  final bool hotkeyUseAlt;
  final bool hotkeyUseShift;
  final int hotkeyVirtualKey;
  final String hotkeyKeyName;
  final int pageSize;
  final int retentionDays;
  final Map<String, String> colorLabels;
  final int duplicateIgnoreWindowMs;
  final int delayBeforeFocusMs;
  final int delayBeforePasteMs;
  final int maxFocusVerifyAttempts;
  final int thumbnailWidth;
  final int thumbnailQuality;
  final DateTime? lastBackupDateUtc;

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
    int? retentionDays,
    Map<String, String>? colorLabels,
    int? duplicateIgnoreWindowMs,
    int? delayBeforeFocusMs,
    int? delayBeforePasteMs,
    int? maxFocusVerifyAttempts,
    int? thumbnailWidth,
    int? thumbnailQuality,
    DateTime? lastBackupDateUtc,
  }) =>
      AppConfig(
        preferredLanguage: preferredLanguage ?? this.preferredLanguage,
        runOnStartup: runOnStartup ?? this.runOnStartup,
        hotkeyUseCtrl: hotkeyUseCtrl ?? this.hotkeyUseCtrl,
        hotkeyUseWin: hotkeyUseWin ?? this.hotkeyUseWin,
        hotkeyUseAlt: hotkeyUseAlt ?? this.hotkeyUseAlt,
        hotkeyUseShift: hotkeyUseShift ?? this.hotkeyUseShift,
        hotkeyVirtualKey: hotkeyVirtualKey ?? this.hotkeyVirtualKey,
        hotkeyKeyName: hotkeyKeyName ?? this.hotkeyKeyName,
        pageSize: pageSize ?? this.pageSize,
        retentionDays: retentionDays ?? this.retentionDays,
        colorLabels: colorLabels ?? this.colorLabels,
        duplicateIgnoreWindowMs:
            duplicateIgnoreWindowMs ?? this.duplicateIgnoreWindowMs,
        delayBeforeFocusMs: delayBeforeFocusMs ?? this.delayBeforeFocusMs,
        delayBeforePasteMs: delayBeforePasteMs ?? this.delayBeforePasteMs,
        maxFocusVerifyAttempts:
            maxFocusVerifyAttempts ?? this.maxFocusVerifyAttempts,
        thumbnailWidth: thumbnailWidth ?? this.thumbnailWidth,
        thumbnailQuality: thumbnailQuality ?? this.thumbnailQuality,
        lastBackupDateUtc: lastBackupDateUtc ?? this.lastBackupDateUtc,
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
        'retentionDays': retentionDays,
        'colorLabels': colorLabels,
        'duplicateIgnoreWindowMs': duplicateIgnoreWindowMs,
        'delayBeforeFocusMs': delayBeforeFocusMs,
        'delayBeforePasteMs': delayBeforePasteMs,
        'maxFocusVerifyAttempts': maxFocusVerifyAttempts,
        'thumbnailWidth': thumbnailWidth,
        'thumbnailQuality': thumbnailQuality,
        if (lastBackupDateUtc != null)
          'lastBackupDateUtc': lastBackupDateUtc!.toIso8601String(),
      };

  static Future<AppConfig> load(String configPath) async {
    final file = File(configPath);
    if (!file.existsSync()) return const AppConfig();
    try {
      final json =
          jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
      return AppConfig.fromJson(json);
    } catch (_) {
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
