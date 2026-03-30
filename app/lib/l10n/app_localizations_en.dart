// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get searchPlaceholder => 'Search clipboard…';

  @override
  String get emptyState => 'No items in this section';

  @override
  String get emptyStateSubtitle => 'Copy something to get started';

  @override
  String get hintBannerText =>
      'CopyPaste is active and running in the background. Look for it in the system tray or just use your shortcut. Feel free to customize your experience in';

  @override
  String get hintBannerAction => 'Settings';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get sectionShortcuts => 'KEYBOARD SHORTCUTS';

  @override
  String get sectionStorage => 'STORAGE';

  @override
  String get settingRunOnStartup => 'Run on startup';

  @override
  String get settingLanguage => 'Interface language';

  @override
  String get hotkeyWillApply => 'Hotkey will apply immediately';

  @override
  String get sectionSupport => 'SUPPORT';

  @override
  String get supportExportLogs => 'Export logs';

  @override
  String get supportExportLogsSubtitle =>
      'Save a zip with app logs for a bug report. Your clipboard content is never included.';

  @override
  String get supportOpenLogsFolder => 'Open logs folder';

  @override
  String get supportOpenLogsFolderSubtitle =>
      'Browse the raw log files in your file manager.';

  @override
  String get supportGitHub => 'Report a bug on GitHub';

  @override
  String get supportExportSuccess => 'Logs saved to Downloads.';

  @override
  String get supportShowInFiles => 'Show';

  @override
  String get supportExportEmpty => 'No log files found.';

  @override
  String get supportExportError => 'Failed to export logs.';

  @override
  String get sectionReset => 'RESET & CLEAN INSTALL';

  @override
  String get resetSoftLabel => 'Soft Reset';

  @override
  String get resetSoftSubtitle =>
      'Resets all settings to defaults and marks app as fresh install. Clipboard history is preserved.';

  @override
  String get resetHardLabel => 'Hard Reset';

  @override
  String get resetHardSubtitle =>
      'Deletes all clipboard history, images, and settings. This cannot be undone.';

  @override
  String get resetSoftConfirmTitle => 'Soft reset?';

  @override
  String get resetSoftConfirmMessage =>
      'All settings will return to defaults and the app will restart as if freshly installed. Your clipboard history will not be deleted.';

  @override
  String get resetHardConfirmTitle => 'Hard reset?';

  @override
  String get resetHardConfirmMessage =>
      'This will permanently delete all clipboard history, images, and settings, then restart the app. This cannot be undone.';

  @override
  String get resetConfirmButton => 'Reset & Restart';

  @override
  String get clearHistoryConfirmTitle => 'Clear history?';

  @override
  String get clearHistoryConfirmMessage =>
      'This will permanently delete all non-pinned clipboard items. This action cannot be undone.';

  @override
  String get clearHistoryConfirmButton => 'Clear';

  @override
  String backupLastDate(String date) {
    return 'Last backup: $date';
  }

  @override
  String get backupNone => 'No backup created yet.';

  @override
  String get backupCreateLabel => 'Create backup';

  @override
  String get backupRestoreLabel => 'Restore backup';

  @override
  String get backupError => 'Failed to create backup. Check permissions.';

  @override
  String get restoreDialogTitle => 'Restore backup';

  @override
  String get restoreDialogWarning =>
      'This will replace all current data with the backup contents. Continue?';

  @override
  String get restoreFileNotFound => 'File not found.';

  @override
  String restoreSuccess(int count) {
    return 'Restored $count items.';
  }

  @override
  String get restoreError =>
      'Restore failed. Your previous data has been preserved.';

  @override
  String get buttonSave => 'Save';

  @override
  String get buttonCancel => 'Cancel';

  @override
  String get buttonReset => 'Restore defaults';

  @override
  String get menuPaste => 'Paste';

  @override
  String get menuPastePlain => 'Paste plain';

  @override
  String get menuPin => 'Pin';

  @override
  String get menuUnpin => 'Unpin';

  @override
  String get menuEdit => 'Edit card';

  @override
  String get menuDelete => 'Delete';

  @override
  String get editColorLabel => 'Color';

  @override
  String get colorRed => 'Red';

  @override
  String get colorGreen => 'Green';

  @override
  String get colorPurple => 'Purple';

  @override
  String get colorYellow => 'Yellow';

  @override
  String get colorBlue => 'Blue';

  @override
  String get colorOrange => 'Orange';

  @override
  String get typeText => 'Text';

  @override
  String get typeImage => 'Image';

  @override
  String get typeFile => 'File';

  @override
  String get typeFolder => 'Folder';

  @override
  String get typeLink => 'Link';

  @override
  String get typeAudio => 'Audio';

  @override
  String get typeVideo => 'Video';

  @override
  String get typeEmail => 'Email';

  @override
  String get typePhone => 'Phone';

  @override
  String get typeColor => 'Color';

  @override
  String get typeIp => 'IP';

  @override
  String get typeUuid => 'UUID';

  @override
  String get typeJson => 'JSON';

  @override
  String get filterAll => 'All';

  @override
  String get filterPinned => 'Pinned';

  @override
  String get trayTooltip => 'CopyPaste';

  @override
  String get trayExit => 'Exit';

  @override
  String get shortcutOpenClose => 'Open / close CopyPaste';

  @override
  String get shortcutEscape => 'Clear search or close window';

  @override
  String get shortcutTab1 => 'Switch to Recent tab';

  @override
  String get shortcutTab2 => 'Switch to Pinned tab';

  @override
  String get shortcutArrows => 'Navigate between items';

  @override
  String get shortcutEnter => 'Paste selected item';

  @override
  String get shortcutDelete => 'Delete selected item';

  @override
  String get shortcutPin => 'Pin / Unpin selected item';

  @override
  String get shortcutEdit => 'Edit card (label and color)';

  @override
  String get tabGeneral => 'General';

  @override
  String get tabBackupRestore => 'Backup';

  @override
  String get tabAppearance => 'Appearance';

  @override
  String get tabShortcuts => 'Shortcuts';

  @override
  String get tabAbout => 'About';

  @override
  String get sectionLanguage => 'LANGUAGE';

  @override
  String get sectionStartup => 'STARTUP';

  @override
  String get sectionKeyboardShortcut => 'KEYBOARD SHORTCUT';

  @override
  String get sectionCategories => 'CATEGORIES';

  @override
  String get sectionPerformance => 'PERFORMANCE';

  @override
  String get sectionPaste => 'PASTE';

  @override
  String get sectionBackupRestore => 'BACKUP & RESTORE';

  @override
  String get sectionAppearance => 'APPEARANCE';

  @override
  String get settingTheme => 'Theme';

  @override
  String get themeLight => 'Light';

  @override
  String get themeDark => 'Dark';

  @override
  String get themeAuto => 'Auto';

  @override
  String get sectionBehavior => 'BEHAVIOR';

  @override
  String get sectionAbout => 'COPYPASTE';

  @override
  String get sectionLinks => 'LINKS';

  @override
  String get settingItemsPerPage => 'Items per page';

  @override
  String get settingMemoryLimit => 'Memory limit';

  @override
  String get settingScrollThreshold => 'Scroll threshold (px)';

  @override
  String get settingPasteSpeed => 'Paste speed';

  @override
  String get settingPanelWidth => 'Panel width (px)';

  @override
  String get settingPanelHeight => 'Panel height (px)';

  @override
  String get settingLinesCollapsed => 'Lines collapsed';

  @override
  String get settingLinesExpanded => 'Lines expanded';

  @override
  String get settingHideOnDeactivate => 'Hide on deactivate';

  @override
  String get settingScrollToTopOnOpen => 'Scroll to top on open';

  @override
  String get settingClearSearchOnOpen => 'Clear search on open';

  @override
  String get settingShowTrayIcon => 'Show tray icon';

  @override
  String get settingRetentionDaysLabel => 'Retention days (0 = unlimited)';

  @override
  String get settingClearHistoryLabel => 'Clear clipboard history';

  @override
  String get settingHotkeyShortcutLabel => 'Shortcut to open/close CopyPaste';

  @override
  String get subtitleStartupDesc => 'Launches in background when you sign in';

  @override
  String get subtitleHideOnDeactivate => 'Close window when clicking outside';

  @override
  String get subtitleScrollToTopOnOpen =>
      'Resets scroll and selects latest item';

  @override
  String get subtitleClearSearchOnOpen => 'Clears the search text each time';

  @override
  String get subtitleShowTrayIcon =>
      'Show icon in the menu bar. Use hotkey if hidden';

  @override
  String get subtitlePasteSpeed => 'Adjust restoration and paste timings';

  @override
  String get subtitleCategories => 'Customize the names of color categories.';

  @override
  String get linkGitHub => 'Support & Source code — GitHub';

  @override
  String get linkCoffee => 'Buy me a coffee';

  @override
  String get editDialogTitle => 'Label & Color';

  @override
  String get editDialogHint => 'Add a label...';

  @override
  String get historyCleared => 'History cleared';

  @override
  String backupSavedFile(String filename) {
    return 'Backup saved: $filename';
  }

  @override
  String get buttonRestore => 'Restore';

  @override
  String get restoreCompleted => 'Restore completed';

  @override
  String get restoreRestartRequired =>
      'Restore completed. The app will restart to apply changes.';

  @override
  String get shortcutExpand => 'Expand / collapse card';

  @override
  String get shortcutFocusSearch => 'Focus search box';

  @override
  String get trayShowHide => 'Show/Hide';

  @override
  String get fileNotFound => 'Not found';

  @override
  String get audioFile => 'Audio file';

  @override
  String get videoFile => 'Video file';

  @override
  String get timeNow => 'now';

  @override
  String get clearAllFilters => 'Clear all filters';

  @override
  String get colorSectionLabel => 'COLOR';

  @override
  String get colorNone => 'None';

  @override
  String get subtitlePastePreset =>
      'Automatic paste speed. Normal/Safe recommended for most computers.';

  @override
  String get subtitleBackup =>
      'Create a backup of your clipboard history, images, and settings. Restore at any time on this or another device.';

  @override
  String get aboutDescription =>
      'A modern clipboard manager built to feel native on Windows, macOS, and Linux.\nLocal-first — your history, always at hand. No accounts, no telemetry, no subscriptions.';

  @override
  String get sectionPrivacy => 'PRIVACY';

  @override
  String get privacyStatement =>
      'Everything local. Nothing leaves your PC — no telemetry, no sync, no accounts.';

  @override
  String get privacyPolicy => 'Privacy Policy';

  @override
  String get aboutTagLocal => 'Local-only';

  @override
  String get aboutTagOpenSource => 'Open source';

  @override
  String get aboutTagFree => 'Free';

  @override
  String get aboutLicense => 'GPL v3 License — Free and open source.';

  @override
  String get permissionsTitle => 'Accessibility Permission Required';

  @override
  String get permissionsMessage =>
      'CopyPaste needs Accessibility permission to paste content into other apps.\n\nGo to System Settings → Privacy & Security → Accessibility and enable CopyPaste.';

  @override
  String get permissionsOpenSettings => 'Open Settings';

  @override
  String get permissionsDismiss => 'Later';

  @override
  String get permissionsGranted => 'Permission granted';

  @override
  String get permissionsResetTitle => 'Accessibility Permission Lost';

  @override
  String get permissionsResetMessage =>
      'macOS no longer recognises CopyPaste\'s permission because the app was re-authorised through Gatekeeper.\n\nTo fix this:\n1. Open Accessibility settings below\n2. Remove CopyPaste from the list (−)\n3. Re-add it or toggle it back on';

  @override
  String get permissionsRestartMessage =>
      'Make sure CopyPaste is enabled in Privacy & Security > Accessibility.\n\nThe app will continue automatically when the permission is detected.';

  @override
  String get permissionsCheckAgain => 'Check Again';

  @override
  String get permissionsRestartApp => 'Restart App';

  @override
  String get permissionsWaiting => 'Waiting for permission…';

  @override
  String updateBadge(String version) {
    return 'v$version is available, please update';
  }

  @override
  String updateAvailableMac(String version) {
    return 'Version $version is available.\n\nUpdate via Homebrew:\nbrew upgrade copypaste\n\nOr download the latest release from GitHub.';
  }

  @override
  String updateAvailableLinux(String version) {
    return 'Version $version is available.\n\nDownload the latest release from GitHub.';
  }

  @override
  String updateAvailableStore(String version) {
    return 'Version $version is available.\n\nUpdate CopyPaste from the Microsoft Store to get the latest version.';
  }

  @override
  String get updateDialogTitle => 'Update Available';

  @override
  String get updateViewRelease => 'View release';

  @override
  String get updateDismiss => 'Later';

  @override
  String get waylandUnsupportedTitle => 'Wayland is not supported';

  @override
  String get waylandUnsupportedBadge => 'Open source · X11 only';

  @override
  String get waylandUnsupportedBody =>
      'Linux support is still a work in progress. This project is maintained by a single person and we need more testers to move forward.\n\nCopyPaste works fully on X11 — to use it, log in with an X11 session. Sorry for the inconvenience.';

  @override
  String get waylandUnsupportedGitHub => 'View on GitHub';

  @override
  String get waylandUnsupportedClose => 'Close';

  @override
  String linuxHotkeyFallbackWarning(String requested, String fallback) {
    return 'The shortcut $requested is unavailable on this X11 desktop. CopyPaste is temporarily using $fallback. You can change it in Settings.';
  }

  @override
  String linuxHotkeyConflictWarning(String requested, String fallback) {
    return 'The shortcut $requested is unavailable on this X11 desktop, and the temporary fallback $fallback also failed. Open Settings to choose another shortcut.';
  }

  @override
  String get settingShowInTaskbar => 'Keep in taskbar';

  @override
  String get subtitleShowInTaskbar =>
      'The app stays visible in the taskbar when closed. Turn off to hide it to the system tray only.';

  @override
  String wakeupHint(String hotkey) {
    return 'CopyPaste runs in the background — press $hotkey or click the tray icon to open it anytime.';
  }

  @override
  String taskbarOpenHint(String hotkey) {
    return 'Tip: press $hotkey to open and paste automatically — no focus lost.';
  }

  @override
  String balloonStartupBody(String hotkey) {
    return 'Running in the background. Press $hotkey or click the tray icon.';
  }

  @override
  String get balloonWakeupTitle => 'CopyPaste is already open';

  @override
  String balloonWakeupBody(String hotkey) {
    return 'Press $hotkey or click the tray icon to bring it up.';
  }

  @override
  String get onboardingTitle => 'Welcome to CopyPaste';

  @override
  String get onboardingSubtitle => 'Everything you copy, saved.';

  @override
  String get onboardingPrivacyBadge => 'No cloud · No tracking · 100% local';

  @override
  String onboardingDescription(String hotkey) {
    return 'Runs silently in the background. Press $hotkey anytime to open your clipboard history.';
  }

  @override
  String get onboardingTrayHint => 'Look for the CP icon next to your clock.';

  @override
  String get onboardingSettingsButton => 'Settings';

  @override
  String get onboardingDismissButton => 'Get started';
}
