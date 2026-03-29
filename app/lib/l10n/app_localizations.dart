import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_es.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('es'),
  ];

  /// Search box placeholder
  ///
  /// In en, this message translates to:
  /// **'Search clipboard…'**
  String get searchPlaceholder;

  /// Empty list message
  ///
  /// In en, this message translates to:
  /// **'No items in this section'**
  String get emptyState;

  /// Empty state subtitle
  ///
  /// In en, this message translates to:
  /// **'Copy something to get started'**
  String get emptyStateSubtitle;

  /// First-run hint banner text
  ///
  /// In en, this message translates to:
  /// **'CopyPaste is active and running in the background. Look for it in the system tray or just use your shortcut. Feel free to customize your experience in'**
  String get hintBannerText;

  /// First-run hint banner action
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get hintBannerAction;

  /// Settings screen title
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settingsTitle;

  /// Shortcuts section header
  ///
  /// In en, this message translates to:
  /// **'KEYBOARD SHORTCUTS'**
  String get sectionShortcuts;

  /// Storage section header
  ///
  /// In en, this message translates to:
  /// **'STORAGE'**
  String get sectionStorage;

  /// Run on startup toggle label
  ///
  /// In en, this message translates to:
  /// **'Run on startup'**
  String get settingRunOnStartup;

  /// Language picker label
  ///
  /// In en, this message translates to:
  /// **'Interface language'**
  String get settingLanguage;

  /// Hint when hotkey changes
  ///
  /// In en, this message translates to:
  /// **'Hotkey will apply immediately'**
  String get hotkeyWillApply;

  /// Support section header in About tab
  ///
  /// In en, this message translates to:
  /// **'SUPPORT'**
  String get sectionSupport;

  /// Export logs action label
  ///
  /// In en, this message translates to:
  /// **'Export logs'**
  String get supportExportLogs;

  /// Export logs subtitle
  ///
  /// In en, this message translates to:
  /// **'Save a zip with app logs for a bug report. Your clipboard content is never included.'**
  String get supportExportLogsSubtitle;

  /// Open logs folder label
  ///
  /// In en, this message translates to:
  /// **'Open logs folder'**
  String get supportOpenLogsFolder;

  /// Open logs folder subtitle
  ///
  /// In en, this message translates to:
  /// **'Browse the raw log files in your file manager.'**
  String get supportOpenLogsFolderSubtitle;

  /// GitHub issue link label
  ///
  /// In en, this message translates to:
  /// **'Report a bug on GitHub'**
  String get supportGitHub;

  /// Snackbar after successful log export
  ///
  /// In en, this message translates to:
  /// **'Logs saved to Downloads.'**
  String get supportExportSuccess;

  /// Snackbar action to reveal the exported file in Finder/Explorer
  ///
  /// In en, this message translates to:
  /// **'Show'**
  String get supportShowInFiles;

  /// Snackbar when no logs exist
  ///
  /// In en, this message translates to:
  /// **'No log files found.'**
  String get supportExportEmpty;

  /// Snackbar on export error
  ///
  /// In en, this message translates to:
  /// **'Failed to export logs.'**
  String get supportExportError;

  /// Reset section header in About tab
  ///
  /// In en, this message translates to:
  /// **'RESET & CLEAN INSTALL'**
  String get sectionReset;

  /// Soft reset action label
  ///
  /// In en, this message translates to:
  /// **'Soft Reset'**
  String get resetSoftLabel;

  /// Soft reset subtitle
  ///
  /// In en, this message translates to:
  /// **'Resets all settings to defaults and marks app as fresh install. Clipboard history is preserved.'**
  String get resetSoftSubtitle;

  /// Hard reset action label
  ///
  /// In en, this message translates to:
  /// **'Hard Reset'**
  String get resetHardLabel;

  /// Hard reset subtitle
  ///
  /// In en, this message translates to:
  /// **'Deletes all clipboard history, images, and settings. This cannot be undone.'**
  String get resetHardSubtitle;

  /// Soft reset confirm dialog title
  ///
  /// In en, this message translates to:
  /// **'Soft reset?'**
  String get resetSoftConfirmTitle;

  /// Soft reset confirm dialog message
  ///
  /// In en, this message translates to:
  /// **'All settings will return to defaults and the app will restart as if freshly installed. Your clipboard history will not be deleted.'**
  String get resetSoftConfirmMessage;

  /// Hard reset confirm dialog title
  ///
  /// In en, this message translates to:
  /// **'Hard reset?'**
  String get resetHardConfirmTitle;

  /// Hard reset confirm dialog message
  ///
  /// In en, this message translates to:
  /// **'This will permanently delete all clipboard history, images, and settings, then restart the app. This cannot be undone.'**
  String get resetHardConfirmMessage;

  /// Reset confirm button label
  ///
  /// In en, this message translates to:
  /// **'Reset & Restart'**
  String get resetConfirmButton;

  /// Clear history dialog title
  ///
  /// In en, this message translates to:
  /// **'Clear history?'**
  String get clearHistoryConfirmTitle;

  /// Clear history dialog message
  ///
  /// In en, this message translates to:
  /// **'This will permanently delete all non-pinned clipboard items. This action cannot be undone.'**
  String get clearHistoryConfirmMessage;

  /// Clear history confirm button
  ///
  /// In en, this message translates to:
  /// **'Clear'**
  String get clearHistoryConfirmButton;

  /// Last backup date
  ///
  /// In en, this message translates to:
  /// **'Last backup: {date}'**
  String backupLastDate(String date);

  /// No backup yet message
  ///
  /// In en, this message translates to:
  /// **'No backup created yet.'**
  String get backupNone;

  /// Create backup label
  ///
  /// In en, this message translates to:
  /// **'Create backup'**
  String get backupCreateLabel;

  /// Restore backup label
  ///
  /// In en, this message translates to:
  /// **'Restore backup'**
  String get backupRestoreLabel;

  /// Backup error message
  ///
  /// In en, this message translates to:
  /// **'Failed to create backup. Check permissions.'**
  String get backupError;

  /// Restore dialog title
  ///
  /// In en, this message translates to:
  /// **'Restore backup'**
  String get restoreDialogTitle;

  /// Restore confirmation warning
  ///
  /// In en, this message translates to:
  /// **'This will replace all current data with the backup contents. Continue?'**
  String get restoreDialogWarning;

  /// File not found error
  ///
  /// In en, this message translates to:
  /// **'File not found.'**
  String get restoreFileNotFound;

  /// Restore success message
  ///
  /// In en, this message translates to:
  /// **'Restored {count} items.'**
  String restoreSuccess(int count);

  /// Restore error message
  ///
  /// In en, this message translates to:
  /// **'Restore failed. Your previous data has been preserved.'**
  String get restoreError;

  /// Save button
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get buttonSave;

  /// Cancel button
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get buttonCancel;

  /// Reset button
  ///
  /// In en, this message translates to:
  /// **'Restore defaults'**
  String get buttonReset;

  /// Context menu paste
  ///
  /// In en, this message translates to:
  /// **'Paste'**
  String get menuPaste;

  /// Context menu paste plain
  ///
  /// In en, this message translates to:
  /// **'Paste plain'**
  String get menuPastePlain;

  /// Context menu pin
  ///
  /// In en, this message translates to:
  /// **'Pin'**
  String get menuPin;

  /// Context menu unpin
  ///
  /// In en, this message translates to:
  /// **'Unpin'**
  String get menuUnpin;

  /// Context menu edit
  ///
  /// In en, this message translates to:
  /// **'Edit card'**
  String get menuEdit;

  /// Context menu delete
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get menuDelete;

  /// Color picker label in edit dialog
  ///
  /// In en, this message translates to:
  /// **'Color'**
  String get editColorLabel;

  /// No description provided for @colorRed.
  ///
  /// In en, this message translates to:
  /// **'Red'**
  String get colorRed;

  /// No description provided for @colorGreen.
  ///
  /// In en, this message translates to:
  /// **'Green'**
  String get colorGreen;

  /// No description provided for @colorPurple.
  ///
  /// In en, this message translates to:
  /// **'Purple'**
  String get colorPurple;

  /// No description provided for @colorYellow.
  ///
  /// In en, this message translates to:
  /// **'Yellow'**
  String get colorYellow;

  /// No description provided for @colorBlue.
  ///
  /// In en, this message translates to:
  /// **'Blue'**
  String get colorBlue;

  /// No description provided for @colorOrange.
  ///
  /// In en, this message translates to:
  /// **'Orange'**
  String get colorOrange;

  /// No description provided for @typeText.
  ///
  /// In en, this message translates to:
  /// **'Text'**
  String get typeText;

  /// No description provided for @typeImage.
  ///
  /// In en, this message translates to:
  /// **'Image'**
  String get typeImage;

  /// No description provided for @typeFile.
  ///
  /// In en, this message translates to:
  /// **'File'**
  String get typeFile;

  /// No description provided for @typeFolder.
  ///
  /// In en, this message translates to:
  /// **'Folder'**
  String get typeFolder;

  /// No description provided for @typeLink.
  ///
  /// In en, this message translates to:
  /// **'Link'**
  String get typeLink;

  /// No description provided for @typeAudio.
  ///
  /// In en, this message translates to:
  /// **'Audio'**
  String get typeAudio;

  /// No description provided for @typeVideo.
  ///
  /// In en, this message translates to:
  /// **'Video'**
  String get typeVideo;

  /// No description provided for @typeEmail.
  ///
  /// In en, this message translates to:
  /// **'Email'**
  String get typeEmail;

  /// No description provided for @typePhone.
  ///
  /// In en, this message translates to:
  /// **'Phone'**
  String get typePhone;

  /// No description provided for @typeColor.
  ///
  /// In en, this message translates to:
  /// **'Color'**
  String get typeColor;

  /// No description provided for @typeIp.
  ///
  /// In en, this message translates to:
  /// **'IP'**
  String get typeIp;

  /// No description provided for @typeUuid.
  ///
  /// In en, this message translates to:
  /// **'UUID'**
  String get typeUuid;

  /// No description provided for @typeJson.
  ///
  /// In en, this message translates to:
  /// **'JSON'**
  String get typeJson;

  /// No description provided for @filterAll.
  ///
  /// In en, this message translates to:
  /// **'All'**
  String get filterAll;

  /// No description provided for @filterPinned.
  ///
  /// In en, this message translates to:
  /// **'Pinned'**
  String get filterPinned;

  /// System tray tooltip
  ///
  /// In en, this message translates to:
  /// **'CopyPaste'**
  String get trayTooltip;

  /// Tray menu exit item
  ///
  /// In en, this message translates to:
  /// **'Exit'**
  String get trayExit;

  /// No description provided for @shortcutOpenClose.
  ///
  /// In en, this message translates to:
  /// **'Open / close CopyPaste'**
  String get shortcutOpenClose;

  /// No description provided for @shortcutEscape.
  ///
  /// In en, this message translates to:
  /// **'Clear search or close window'**
  String get shortcutEscape;

  /// No description provided for @shortcutTab1.
  ///
  /// In en, this message translates to:
  /// **'Switch to Recent tab'**
  String get shortcutTab1;

  /// No description provided for @shortcutTab2.
  ///
  /// In en, this message translates to:
  /// **'Switch to Pinned tab'**
  String get shortcutTab2;

  /// No description provided for @shortcutArrows.
  ///
  /// In en, this message translates to:
  /// **'Navigate between items'**
  String get shortcutArrows;

  /// No description provided for @shortcutEnter.
  ///
  /// In en, this message translates to:
  /// **'Paste selected item'**
  String get shortcutEnter;

  /// No description provided for @shortcutDelete.
  ///
  /// In en, this message translates to:
  /// **'Delete selected item'**
  String get shortcutDelete;

  /// No description provided for @shortcutPin.
  ///
  /// In en, this message translates to:
  /// **'Pin / Unpin selected item'**
  String get shortcutPin;

  /// No description provided for @shortcutEdit.
  ///
  /// In en, this message translates to:
  /// **'Edit card (label and color)'**
  String get shortcutEdit;

  /// General nav tab
  ///
  /// In en, this message translates to:
  /// **'General'**
  String get tabGeneral;

  /// Backup nav tab
  ///
  /// In en, this message translates to:
  /// **'Backup'**
  String get tabBackupRestore;

  /// Appearance nav tab
  ///
  /// In en, this message translates to:
  /// **'Appearance'**
  String get tabAppearance;

  /// Shortcuts nav tab
  ///
  /// In en, this message translates to:
  /// **'Shortcuts'**
  String get tabShortcuts;

  /// About nav tab
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get tabAbout;

  /// Language section title
  ///
  /// In en, this message translates to:
  /// **'LANGUAGE'**
  String get sectionLanguage;

  /// Startup section title
  ///
  /// In en, this message translates to:
  /// **'STARTUP'**
  String get sectionStartup;

  /// Keyboard shortcut section title
  ///
  /// In en, this message translates to:
  /// **'KEYBOARD SHORTCUT'**
  String get sectionKeyboardShortcut;

  /// Categories section title
  ///
  /// In en, this message translates to:
  /// **'CATEGORIES'**
  String get sectionCategories;

  /// Performance section title
  ///
  /// In en, this message translates to:
  /// **'PERFORMANCE'**
  String get sectionPerformance;

  /// Paste section title
  ///
  /// In en, this message translates to:
  /// **'PASTE'**
  String get sectionPaste;

  /// Backup and restore section title
  ///
  /// In en, this message translates to:
  /// **'BACKUP & RESTORE'**
  String get sectionBackupRestore;

  /// Appearance section title
  ///
  /// In en, this message translates to:
  /// **'APPEARANCE'**
  String get sectionAppearance;

  /// Theme selector label
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get settingTheme;

  /// Light theme option
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themeLight;

  /// Dark theme option
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themeDark;

  /// Auto theme option
  ///
  /// In en, this message translates to:
  /// **'Auto'**
  String get themeAuto;

  /// Behavior section title
  ///
  /// In en, this message translates to:
  /// **'BEHAVIOR'**
  String get sectionBehavior;

  /// About section title
  ///
  /// In en, this message translates to:
  /// **'COPYPASTE'**
  String get sectionAbout;

  /// Links section title
  ///
  /// In en, this message translates to:
  /// **'LINKS'**
  String get sectionLinks;

  /// Items per page label
  ///
  /// In en, this message translates to:
  /// **'Items per page'**
  String get settingItemsPerPage;

  /// Memory limit label
  ///
  /// In en, this message translates to:
  /// **'Memory limit'**
  String get settingMemoryLimit;

  /// Scroll threshold label
  ///
  /// In en, this message translates to:
  /// **'Scroll threshold (px)'**
  String get settingScrollThreshold;

  /// Paste speed label
  ///
  /// In en, this message translates to:
  /// **'Paste speed'**
  String get settingPasteSpeed;

  /// Panel width label
  ///
  /// In en, this message translates to:
  /// **'Panel width (px)'**
  String get settingPanelWidth;

  /// Panel height label
  ///
  /// In en, this message translates to:
  /// **'Panel height (px)'**
  String get settingPanelHeight;

  /// Lines collapsed label
  ///
  /// In en, this message translates to:
  /// **'Lines collapsed'**
  String get settingLinesCollapsed;

  /// Lines expanded label
  ///
  /// In en, this message translates to:
  /// **'Lines expanded'**
  String get settingLinesExpanded;

  /// Hide on deactivate label
  ///
  /// In en, this message translates to:
  /// **'Hide on deactivate'**
  String get settingHideOnDeactivate;

  /// Scroll to top on open label
  ///
  /// In en, this message translates to:
  /// **'Scroll to top on open'**
  String get settingScrollToTopOnOpen;

  /// Clear search on open label
  ///
  /// In en, this message translates to:
  /// **'Clear search on open'**
  String get settingClearSearchOnOpen;

  /// Show tray icon label (macOS only)
  ///
  /// In en, this message translates to:
  /// **'Show tray icon'**
  String get settingShowTrayIcon;

  /// Retention days label
  ///
  /// In en, this message translates to:
  /// **'Retention days (0 = unlimited)'**
  String get settingRetentionDaysLabel;

  /// Clear clipboard history label
  ///
  /// In en, this message translates to:
  /// **'Clear clipboard history'**
  String get settingClearHistoryLabel;

  /// Hotkey shortcut label
  ///
  /// In en, this message translates to:
  /// **'Shortcut to open/close CopyPaste'**
  String get settingHotkeyShortcutLabel;

  /// Startup subtitle
  ///
  /// In en, this message translates to:
  /// **'Launches in background when you sign in'**
  String get subtitleStartupDesc;

  /// Hide on deactivate subtitle
  ///
  /// In en, this message translates to:
  /// **'Close window when clicking outside'**
  String get subtitleHideOnDeactivate;

  /// Scroll to top on open subtitle
  ///
  /// In en, this message translates to:
  /// **'Resets scroll and selects latest item'**
  String get subtitleScrollToTopOnOpen;

  /// Clear search on open subtitle
  ///
  /// In en, this message translates to:
  /// **'Clears the search text each time'**
  String get subtitleClearSearchOnOpen;

  /// Show tray icon subtitle (macOS only)
  ///
  /// In en, this message translates to:
  /// **'Show icon in the menu bar. Use hotkey if hidden'**
  String get subtitleShowTrayIcon;

  /// Paste speed subtitle
  ///
  /// In en, this message translates to:
  /// **'Adjust restoration and paste timings'**
  String get subtitlePasteSpeed;

  /// Categories subtitle
  ///
  /// In en, this message translates to:
  /// **'Customize the names of color categories.'**
  String get subtitleCategories;

  /// GitHub link label
  ///
  /// In en, this message translates to:
  /// **'Support & Source code — GitHub'**
  String get linkGitHub;

  /// Buy me a coffee link label
  ///
  /// In en, this message translates to:
  /// **'Buy me a coffee'**
  String get linkCoffee;

  /// Edit card dialog title
  ///
  /// In en, this message translates to:
  /// **'Label & Color'**
  String get editDialogTitle;

  /// Label input hint in edit dialog
  ///
  /// In en, this message translates to:
  /// **'Add a label...'**
  String get editDialogHint;

  /// Snackbar after clearing history
  ///
  /// In en, this message translates to:
  /// **'History cleared'**
  String get historyCleared;

  /// Backup saved snackbar
  ///
  /// In en, this message translates to:
  /// **'Backup saved: {filename}'**
  String backupSavedFile(String filename);

  /// Restore action button
  ///
  /// In en, this message translates to:
  /// **'Restore'**
  String get buttonRestore;

  /// Restore completed snackbar
  ///
  /// In en, this message translates to:
  /// **'Restore completed'**
  String get restoreCompleted;

  /// Restore requires restart message
  ///
  /// In en, this message translates to:
  /// **'Restore completed. The app will restart to apply changes.'**
  String get restoreRestartRequired;

  /// Expand collapse shortcut
  ///
  /// In en, this message translates to:
  /// **'Expand / collapse card'**
  String get shortcutExpand;

  /// Focus search shortcut
  ///
  /// In en, this message translates to:
  /// **'Focus search box'**
  String get shortcutFocusSearch;

  /// Tray menu show/hide item
  ///
  /// In en, this message translates to:
  /// **'Show/Hide'**
  String get trayShowHide;

  /// Badge when file is missing
  ///
  /// In en, this message translates to:
  /// **'Not found'**
  String get fileNotFound;

  /// Fallback name for audio items
  ///
  /// In en, this message translates to:
  /// **'Audio file'**
  String get audioFile;

  /// Fallback name for video items
  ///
  /// In en, this message translates to:
  /// **'Video file'**
  String get videoFile;

  /// Timestamp for less than 1 minute ago
  ///
  /// In en, this message translates to:
  /// **'now'**
  String get timeNow;

  /// Filter menu clear action
  ///
  /// In en, this message translates to:
  /// **'Clear all filters'**
  String get clearAllFilters;

  /// Filter menu color section header
  ///
  /// In en, this message translates to:
  /// **'COLOR'**
  String get colorSectionLabel;

  /// No color option
  ///
  /// In en, this message translates to:
  /// **'None'**
  String get colorNone;

  /// Paste preset subtitle
  ///
  /// In en, this message translates to:
  /// **'Automatic paste speed. Normal/Safe recommended for most computers.'**
  String get subtitlePastePreset;

  /// Backup section subtitle
  ///
  /// In en, this message translates to:
  /// **'Create a backup of your clipboard history, images, and settings. Restore at any time on this or another device.'**
  String get subtitleBackup;

  /// About section description
  ///
  /// In en, this message translates to:
  /// **'A modern clipboard manager built to feel native on Windows, macOS, and Linux.\nLocal-first — your history, always at hand. No accounts, no telemetry, no subscriptions.'**
  String get aboutDescription;

  /// Privacy section title in About tab
  ///
  /// In en, this message translates to:
  /// **'PRIVACY'**
  String get sectionPrivacy;

  /// Short privacy philosophy statement shown in About tab
  ///
  /// In en, this message translates to:
  /// **'Everything local. Nothing leaves your PC — no telemetry, no sync, no accounts.'**
  String get privacyStatement;

  /// Link label to open the full privacy policy
  ///
  /// In en, this message translates to:
  /// **'Privacy Policy'**
  String get privacyPolicy;

  /// Badge label: everything is stored locally
  ///
  /// In en, this message translates to:
  /// **'Local-only'**
  String get aboutTagLocal;

  /// Badge label: the app is open source
  ///
  /// In en, this message translates to:
  /// **'Open source'**
  String get aboutTagOpenSource;

  /// Badge label: the app is free
  ///
  /// In en, this message translates to:
  /// **'Free'**
  String get aboutTagFree;

  /// License footer text
  ///
  /// In en, this message translates to:
  /// **'GPL v3 License — Free and open source.'**
  String get aboutLicense;

  /// Title for the macOS accessibility permissions dialog
  ///
  /// In en, this message translates to:
  /// **'Accessibility Permission Required'**
  String get permissionsTitle;

  /// Body text explaining why accessibility permission is needed
  ///
  /// In en, this message translates to:
  /// **'CopyPaste needs Accessibility permission to paste content into other apps.\n\nGo to System Settings → Privacy & Security → Accessibility and enable CopyPaste.'**
  String get permissionsMessage;

  /// Button to open macOS System Settings
  ///
  /// In en, this message translates to:
  /// **'Open Settings'**
  String get permissionsOpenSettings;

  /// Dismiss button for permissions dialog
  ///
  /// In en, this message translates to:
  /// **'Later'**
  String get permissionsDismiss;

  /// Snackbar message when permission is confirmed
  ///
  /// In en, this message translates to:
  /// **'Permission granted'**
  String get permissionsGranted;

  /// Title shown when permission was previously granted but is no longer recognised (Gatekeeper identity change)
  ///
  /// In en, this message translates to:
  /// **'Accessibility Permission Lost'**
  String get permissionsResetTitle;

  /// Instructions for fixing stale TCC entries after Gatekeeper re-authorisation
  ///
  /// In en, this message translates to:
  /// **'macOS no longer recognises CopyPaste\'s permission because the app was re-authorised through Gatekeeper.\n\nTo fix this:\n1. Open Accessibility settings below\n2. Remove CopyPaste from the list (−)\n3. Re-add it or toggle it back on'**
  String get permissionsResetMessage;

  /// Shown after polling times out without detecting the permission grant
  ///
  /// In en, this message translates to:
  /// **'Make sure CopyPaste is enabled in Privacy & Security > Accessibility.\n\nThe app will continue automatically when the permission is detected.'**
  String get permissionsRestartMessage;

  /// Button to manually re-check accessibility permission
  ///
  /// In en, this message translates to:
  /// **'Check Again'**
  String get permissionsCheckAgain;

  /// Button to restart the app when permission detection is stuck
  ///
  /// In en, this message translates to:
  /// **'Restart App'**
  String get permissionsRestartApp;

  /// Label shown while polling for the accessibility permission grant
  ///
  /// In en, this message translates to:
  /// **'Waiting for permission…'**
  String get permissionsWaiting;

  /// Short text shown in the footer when an update is available
  ///
  /// In en, this message translates to:
  /// **'v{version} is available, please update'**
  String updateBadge(String version);

  /// Update dialog message for macOS
  ///
  /// In en, this message translates to:
  /// **'Version {version} is available.\n\nUpdate via Homebrew:\nbrew upgrade copypaste\n\nOr download the latest release from GitHub.'**
  String updateAvailableMac(String version);

  /// Update dialog message for Linux
  ///
  /// In en, this message translates to:
  /// **'Version {version} is available.\n\nDownload the latest release from GitHub.'**
  String updateAvailableLinux(String version);

  /// Update dialog message for MS Store builds
  ///
  /// In en, this message translates to:
  /// **'Version {version} is available.\n\nUpdate CopyPaste from the Microsoft Store to get the latest version.'**
  String updateAvailableStore(String version);

  /// Title of the update available dialog
  ///
  /// In en, this message translates to:
  /// **'Update Available'**
  String get updateDialogTitle;

  /// Button to open the GitHub release page
  ///
  /// In en, this message translates to:
  /// **'View release'**
  String get updateViewRelease;

  /// Button to dismiss the update notification
  ///
  /// In en, this message translates to:
  /// **'Later'**
  String get updateDismiss;

  /// Title for the Wayland-unsupported gate screen
  ///
  /// In en, this message translates to:
  /// **'Wayland is not supported yet'**
  String get waylandUnsupportedTitle;

  /// Badge chip on the Wayland-unsupported gate screen
  ///
  /// In en, this message translates to:
  /// **'Open source · X11 only'**
  String get waylandUnsupportedBadge;

  /// Body text on the Wayland-unsupported gate screen
  ///
  /// In en, this message translates to:
  /// **'Linux support is still limited — we\'re working on it. For now, only X11 is supported. Please start your session in X11 to use CopyPaste.'**
  String get waylandUnsupportedBody;

  /// Button to open the repo from the Wayland gate
  ///
  /// In en, this message translates to:
  /// **'Contribute on GitHub'**
  String get waylandUnsupportedGitHub;

  /// Button to exit the app from the Wayland gate
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get waylandUnsupportedClose;

  /// Shown when the preferred Linux hotkey is unavailable and a temporary fallback is active
  ///
  /// In en, this message translates to:
  /// **'The shortcut {requested} is unavailable on this X11 desktop. CopyPaste is temporarily using {fallback}. You can change it in Settings.'**
  String linuxHotkeyFallbackWarning(String requested, String fallback);

  /// Shown when both the requested Linux hotkey and the temporary fallback fail
  ///
  /// In en, this message translates to:
  /// **'The shortcut {requested} is unavailable on this X11 desktop, and the temporary fallback {fallback} also failed. Open Settings to choose another shortcut.'**
  String linuxHotkeyConflictWarning(String requested, String fallback);

  /// Label for the Windows taskbar visibility toggle in Settings
  ///
  /// In en, this message translates to:
  /// **'Keep in taskbar'**
  String get settingShowInTaskbar;

  /// Subtitle for the Windows taskbar visibility toggle in Settings
  ///
  /// In en, this message translates to:
  /// **'The app stays visible in the taskbar when closed. Turn off to hide it to the system tray only.'**
  String get subtitleShowInTaskbar;

  /// In-app snackbar shown inside the window when it is raised by a second launch attempt
  ///
  /// In en, this message translates to:
  /// **'CopyPaste runs in the background — press {hotkey} or click the tray icon to open it anytime.'**
  String wakeupHint(String hotkey);

  /// Hint shown when user opens CopyPaste from the taskbar in taskbar mode
  ///
  /// In en, this message translates to:
  /// **'Tip: press {hotkey} to open and paste automatically — no focus lost.'**
  String taskbarOpenHint(String hotkey);

  /// Windows balloon shown at startup when window starts hidden
  ///
  /// In en, this message translates to:
  /// **'Running in the background. Press {hotkey} or click the tray icon.'**
  String balloonStartupBody(String hotkey);

  /// Windows balloon title when a second instance is launched
  ///
  /// In en, this message translates to:
  /// **'CopyPaste is already open'**
  String get balloonWakeupTitle;

  /// Windows balloon body when a second instance is launched
  ///
  /// In en, this message translates to:
  /// **'Press {hotkey} or click the tray icon to bring it up.'**
  String balloonWakeupBody(String hotkey);

  /// Onboarding screen title
  ///
  /// In en, this message translates to:
  /// **'Welcome to CopyPaste'**
  String get onboardingTitle;

  /// Onboarding screen subtitle
  ///
  /// In en, this message translates to:
  /// **'Everything you copy, saved.'**
  String get onboardingSubtitle;

  /// Onboarding privacy badge chip
  ///
  /// In en, this message translates to:
  /// **'No cloud · No tracking · 100% local'**
  String get onboardingPrivacyBadge;

  /// Onboarding main description
  ///
  /// In en, this message translates to:
  /// **'Runs silently in the background. Press {hotkey} anytime to open your clipboard history.'**
  String onboardingDescription(String hotkey);

  /// Onboarding tray location hint
  ///
  /// In en, this message translates to:
  /// **'Look for the CP icon next to your clock.'**
  String get onboardingTrayHint;

  /// Onboarding settings button
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get onboardingSettingsButton;

  /// Onboarding dismiss button
  ///
  /// In en, this message translates to:
  /// **'Get started'**
  String get onboardingDismissButton;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'es'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'es':
      return AppLocalizationsEs();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
