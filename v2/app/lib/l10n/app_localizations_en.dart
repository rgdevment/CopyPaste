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
  String get hintBannerText => 'Customize your experience in';

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
  String get buttonReset => 'Reset';

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
      'A lightweight, fast, and open-source clipboard manager for Windows, macOS, and Linux.\nCopy once, paste anywhere — your clipboard history, always at hand.';

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
}
