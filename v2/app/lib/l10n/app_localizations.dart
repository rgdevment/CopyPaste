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

  /// Application name
  ///
  /// In en, this message translates to:
  /// **'CopyPaste'**
  String get appTitle;

  /// Recent tab label
  ///
  /// In en, this message translates to:
  /// **'Recent'**
  String get tabRecent;

  /// Pinned tab label
  ///
  /// In en, this message translates to:
  /// **'Pinned'**
  String get tabPinned;

  /// Search box placeholder
  ///
  /// In en, this message translates to:
  /// **'Search...'**
  String get searchPlaceholder;

  /// Empty list message
  ///
  /// In en, this message translates to:
  /// **'No items in this section'**
  String get emptyState;

  /// Update available banner text
  ///
  /// In en, this message translates to:
  /// **'v{version} available — click to download'**
  String updateBannerLabel(String version);

  /// First-run hint banner text
  ///
  /// In en, this message translates to:
  /// **'Customize your experience in'**
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

  /// General section header
  ///
  /// In en, this message translates to:
  /// **'GENERAL'**
  String get sectionGeneral;

  /// Hotkey section header
  ///
  /// In en, this message translates to:
  /// **'HOTKEY'**
  String get sectionHotkey;

  /// Storage section header
  ///
  /// In en, this message translates to:
  /// **'STORAGE'**
  String get sectionStorage;

  /// Backup section header
  ///
  /// In en, this message translates to:
  /// **'BACKUP'**
  String get sectionBackup;

  /// Shortcuts section header
  ///
  /// In en, this message translates to:
  /// **'KEYBOARD SHORTCUTS'**
  String get sectionShortcuts;

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

  /// Auto language option
  ///
  /// In en, this message translates to:
  /// **'Auto'**
  String get languageAuto;

  /// English language option
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get languageEnglish;

  /// Spanish language option
  ///
  /// In en, this message translates to:
  /// **'Español'**
  String get languageSpanish;

  /// Hotkey description
  ///
  /// In en, this message translates to:
  /// **'Global shortcut to open CopyPaste'**
  String get settingHotkeyLabel;

  /// Hint when hotkey changes
  ///
  /// In en, this message translates to:
  /// **'Hotkey will apply immediately'**
  String get hotkeyWillApply;

  /// Retention days setting label
  ///
  /// In en, this message translates to:
  /// **'History retention'**
  String get settingRetentionDays;

  /// Retention days description
  ///
  /// In en, this message translates to:
  /// **'Days to keep history (0 = unlimited)'**
  String get settingRetentionDaysDesc;

  /// Clear history button label
  ///
  /// In en, this message translates to:
  /// **'Clear all history'**
  String get settingClearHistory;

  /// Clear history description
  ///
  /// In en, this message translates to:
  /// **'Permanently delete all clipboard items'**
  String get settingClearHistoryDesc;

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

  /// Create backup description
  ///
  /// In en, this message translates to:
  /// **'Export all data to a ZIP file'**
  String get backupCreateDesc;

  /// Restore backup label
  ///
  /// In en, this message translates to:
  /// **'Restore backup'**
  String get backupRestoreLabel;

  /// Restore backup description
  ///
  /// In en, this message translates to:
  /// **'Import data from a backup file'**
  String get backupRestoreDesc;

  /// Creating backup snackbar
  ///
  /// In en, this message translates to:
  /// **'Creating backup...'**
  String get backupCreating;

  /// Backup created success message
  ///
  /// In en, this message translates to:
  /// **'Backup created: {count} items, {images} images.'**
  String backupSuccess(int count, int images);

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

  /// Restore path input hint
  ///
  /// In en, this message translates to:
  /// **'Path to .zip backup file'**
  String get restoreDialogHint;

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

  /// Invalid backup file error
  ///
  /// In en, this message translates to:
  /// **'Invalid backup file. Select a valid CopyPaste backup (.zip).'**
  String get restoreInvalidFile;

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
  /// **'Reset'**
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

  /// Edit card dialog title
  ///
  /// In en, this message translates to:
  /// **'Edit card'**
  String get editCardTitle;

  /// Label input placeholder
  ///
  /// In en, this message translates to:
  /// **'Label (optional)'**
  String get editLabelPlaceholder;

  /// Label input hint
  ///
  /// In en, this message translates to:
  /// **'40 characters max'**
  String get editLabelHint;

  /// Color picker label in edit dialog
  ///
  /// In en, this message translates to:
  /// **'Color'**
  String get editColorLabel;

  /// No description provided for @colorNone.
  ///
  /// In en, this message translates to:
  /// **'None'**
  String get colorNone;

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

  /// File not available warning on card
  ///
  /// In en, this message translates to:
  /// **'File not available'**
  String get fileNotAvailable;

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

  /// Keyboard shortcuts section title
  ///
  /// In en, this message translates to:
  /// **'Keyboard Shortcuts'**
  String get shortcutsTitle;

  /// Keyboard shortcuts subtitle
  ///
  /// In en, this message translates to:
  /// **'Power your workflow'**
  String get shortcutsSubtitle;

  /// No description provided for @shortcutsGroupGeneral.
  ///
  /// In en, this message translates to:
  /// **'GENERAL'**
  String get shortcutsGroupGeneral;

  /// No description provided for @shortcutsGroupNavigation.
  ///
  /// In en, this message translates to:
  /// **'NAVIGATION'**
  String get shortcutsGroupNavigation;

  /// No description provided for @shortcutsGroupActions.
  ///
  /// In en, this message translates to:
  /// **'ACTIONS'**
  String get shortcutsGroupActions;

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

  /// Thumbnails section title
  ///
  /// In en, this message translates to:
  /// **'THUMBNAILS (ADVANCED)'**
  String get sectionThumbnails;

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

  /// Keep window visible label
  ///
  /// In en, this message translates to:
  /// **'Keep window visible'**
  String get settingKeepWindowVisible;

  /// Scroll to top after paste label
  ///
  /// In en, this message translates to:
  /// **'Scroll to top after paste'**
  String get settingScrollToTopOnPaste;

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

  /// Thumbnail quality label
  ///
  /// In en, this message translates to:
  /// **'Thumbnail quality'**
  String get settingThumbnailQuality;

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

  /// Keep window visible subtitle
  ///
  /// In en, this message translates to:
  /// **'Window stays open when you click on other apps'**
  String get subtitleKeepWindowVisible;

  /// Scroll to top on paste subtitle
  ///
  /// In en, this message translates to:
  /// **'Only when window is pinned'**
  String get subtitleScrollToTopOnPaste;

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

  /// Paste speed subtitle
  ///
  /// In en, this message translates to:
  /// **'Adjust restoration and paste timings'**
  String get subtitlePasteSpeed;

  /// Thumbnails section subtitle
  ///
  /// In en, this message translates to:
  /// **'Modifying can affect RAM usage and disk space.'**
  String get subtitleThumbnails;

  /// Thumbnail quality subtitle
  ///
  /// In en, this message translates to:
  /// **'Only affects future thumbnails'**
  String get subtitleThumbnailQuality;

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
