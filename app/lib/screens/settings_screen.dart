// coverage:ignore-file
import 'dart:io';

import 'package:path/path.dart' as p;

import 'package:core/core.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import '../helpers/url_helper.dart';
import '../l10n/app_localizations.dart';

import '../shell/startup_helper.dart';
import '../theme/app_theme_data.dart';
import '../theme/theme_provider.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({
    required this.config,
    required this.configPath,
    required this.clipboardService,
    required this.storage,
    required this.onSave,
    required this.onSoftReset,
    required this.onHardReset,
    super.key,
  });

  final AppConfig config;
  final String configPath;
  final ClipboardService clipboardService;
  final StorageConfig storage;
  final Future<void> Function(AppConfig newConfig, bool hotkeyChanged) onSave;

  /// Resets config + first-run flag, keeps clipboard history, then restarts.
  final Future<void> Function() onSoftReset;

  /// Deletes all data (db, images, config, first-run flag), then restarts.
  final Future<void> Function() onHardReset;

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  int _selectedTab = 0;
  bool _hasChanges = false;

  late String _preferredLanguage;
  late bool _runOnStartup;

  late bool _hotkeyCtrl;
  late bool _hotkeyWin;
  late bool _hotkeyAlt;
  late bool _hotkeyShift;
  late int _hotkeyVirtualKey;
  late String _hotkeyKeyName;

  late Map<String, String> _colorLabels;

  late int _pageSize;
  late int _maxItemsBeforeCleanup;
  late int _scrollLoadThreshold;

  late int _retentionDays;

  late int _duplicateIgnoreWindowMs;
  late int _delayBeforeFocusMs;
  late int _delayBeforePasteMs;
  late int _maxFocusVerifyAttempts;

  late DateTime? _lastBackupDateUtc;

  late int _popupWidth;
  late int _popupHeight;
  late int _cardMinLines;
  late int _cardMaxLines;
  late String _themeMode;

  late bool _hideOnDeactivate;
  late bool _resetScrollOnShow;
  late bool _resetSearchOnShow;
  late bool _showTrayIcon;
  late bool _showInTaskbar;

  bool get _hotkeyChanged =>
      _hotkeyCtrl != widget.config.hotkeyUseCtrl ||
      _hotkeyWin != widget.config.hotkeyUseWin ||
      _hotkeyAlt != widget.config.hotkeyUseAlt ||
      _hotkeyShift != widget.config.hotkeyUseShift ||
      _hotkeyVirtualKey != widget.config.hotkeyVirtualKey;

  @override
  void initState() {
    super.initState();
    _preferredLanguage = widget.config.preferredLanguage;
    _runOnStartup = widget.config.runOnStartup;
    _hotkeyCtrl = widget.config.hotkeyUseCtrl;
    _hotkeyWin = widget.config.hotkeyUseWin;
    _hotkeyAlt = widget.config.hotkeyUseAlt;
    _hotkeyShift = widget.config.hotkeyUseShift;
    _hotkeyVirtualKey = widget.config.hotkeyVirtualKey;
    _hotkeyKeyName = widget.config.hotkeyKeyName;
    _colorLabels = Map.of(widget.config.colorLabels);
    _pageSize = widget.config.pageSize;
    _maxItemsBeforeCleanup = widget.config.maxItemsBeforeCleanup;
    _scrollLoadThreshold = widget.config.scrollLoadThreshold;
    _retentionDays = widget.config.retentionDays;
    _duplicateIgnoreWindowMs = widget.config.duplicateIgnoreWindowMs;
    _delayBeforeFocusMs = widget.config.delayBeforeFocusMs;
    _delayBeforePasteMs = widget.config.delayBeforePasteMs;
    _maxFocusVerifyAttempts = widget.config.maxFocusVerifyAttempts;
    _lastBackupDateUtc = widget.config.lastBackupDateUtc;
    _popupWidth = widget.config.popupWidth;
    _popupHeight = widget.config.popupHeight;
    _cardMinLines = widget.config.cardMinLines;
    _cardMaxLines = widget.config.cardMaxLines;
    _themeMode = widget.config.themeMode;
    _hideOnDeactivate = widget.config.hideOnDeactivate;
    _resetScrollOnShow = widget.config.resetScrollOnShow;
    _resetSearchOnShow = widget.config.resetSearchOnShow;
    _showTrayIcon = widget.config.showTrayIcon;
    _showInTaskbar = widget.config.showInTaskbar;
  }

  void _markChanged() {
    if (!_hasChanges) setState(() => _hasChanges = true);
  }

  AppConfig _buildConfig() => widget.config.copyWith(
    preferredLanguage: _preferredLanguage,
    runOnStartup: _runOnStartup,
    hotkeyUseCtrl: _hotkeyCtrl,
    hotkeyUseWin: _hotkeyWin,
    hotkeyUseAlt: _hotkeyAlt,
    hotkeyUseShift: _hotkeyShift,
    hotkeyVirtualKey: _hotkeyVirtualKey,
    hotkeyKeyName: _hotkeyKeyName,
    colorLabels: _colorLabels,
    pageSize: _pageSize,
    maxItemsBeforeCleanup: _maxItemsBeforeCleanup,
    scrollLoadThreshold: _scrollLoadThreshold,
    retentionDays: _retentionDays,
    duplicateIgnoreWindowMs: _duplicateIgnoreWindowMs,
    delayBeforeFocusMs: _delayBeforeFocusMs,
    delayBeforePasteMs: _delayBeforePasteMs,
    maxFocusVerifyAttempts: _maxFocusVerifyAttempts,
    lastBackupDateUtc: _lastBackupDateUtc,
    popupWidth: _popupWidth,
    popupHeight: _popupHeight,
    cardMinLines: _cardMinLines,
    cardMaxLines: _cardMaxLines,
    themeMode: _themeMode,
    hideOnDeactivate: _hideOnDeactivate,
    resetScrollOnShow: _resetScrollOnShow,
    resetSearchOnShow: _resetSearchOnShow,
    showTrayIcon: _showTrayIcon,
    showInTaskbar: _showInTaskbar,
  );

  Future<void> _save() async {
    final hotkeyChanged = _hotkeyChanged;
    final newConfig = _buildConfig();
    await newConfig.save(widget.configPath);
    await StartupHelper.apply(_runOnStartup);
    await widget.onSave(newConfig, hotkeyChanged);
  }

  void _resetToDefaults() {
    final d = AppConfig.defaultForCurrentPlatform();
    setState(() {
      _preferredLanguage = d.preferredLanguage;
      _runOnStartup = d.runOnStartup;
      _hotkeyCtrl = d.hotkeyUseCtrl;
      _hotkeyWin = d.hotkeyUseWin;
      _hotkeyAlt = d.hotkeyUseAlt;
      _hotkeyShift = d.hotkeyUseShift;
      _hotkeyVirtualKey = d.hotkeyVirtualKey;
      _hotkeyKeyName = d.hotkeyKeyName;
      _colorLabels = {};
      _pageSize = d.pageSize;
      _maxItemsBeforeCleanup = d.maxItemsBeforeCleanup;
      _scrollLoadThreshold = d.scrollLoadThreshold;
      _retentionDays = d.retentionDays;
      _duplicateIgnoreWindowMs = d.duplicateIgnoreWindowMs;
      _delayBeforeFocusMs = d.delayBeforeFocusMs;
      _delayBeforePasteMs = d.delayBeforePasteMs;
      _maxFocusVerifyAttempts = d.maxFocusVerifyAttempts;
      _popupWidth = d.popupWidth;
      _popupHeight = d.popupHeight;
      _cardMinLines = d.cardMinLines;
      _cardMaxLines = d.cardMaxLines;
      _themeMode = d.themeMode;
      _hideOnDeactivate = d.hideOnDeactivate;
      _resetScrollOnShow = d.resetScrollOnShow;
      _resetSearchOnShow = d.resetSearchOnShow;
      _showTrayIcon = d.showTrayIcon;
      _showInTaskbar = d.showInTaskbar;
      _hasChanges = true;
    });
  }

  String _hotkeyString([String separator = '+']) {
    final isMac = Platform.isMacOS;
    final parts = <String>[];
    if (_hotkeyCtrl) parts.add('Ctrl');
    if (_hotkeyWin) parts.add(isMac ? 'Cmd' : 'Win');
    if (_hotkeyAlt) parts.add(isMac ? 'Option' : 'Alt');
    if (_hotkeyShift) parts.add('Shift');
    parts.add(_hotkeyKeyName);
    return parts.join(separator);
  }

  String? get _pastePresetName {
    if (_delayBeforeFocusMs == 50 &&
        _delayBeforePasteMs == 80 &&
        _maxFocusVerifyAttempts == 10 &&
        _duplicateIgnoreWindowMs == 300) {
      return 'Fast';
    }
    if (_delayBeforeFocusMs == 80 &&
        _delayBeforePasteMs == 120 &&
        _maxFocusVerifyAttempts == 12 &&
        _duplicateIgnoreWindowMs == 350) {
      return 'Normal';
    }
    if (_delayBeforeFocusMs == 100 &&
        _delayBeforePasteMs == 180 &&
        _maxFocusVerifyAttempts == 15 &&
        _duplicateIgnoreWindowMs == 450) {
      return 'Safe';
    }
    if (_delayBeforeFocusMs == 150 &&
        _delayBeforePasteMs == 250 &&
        _maxFocusVerifyAttempts == 20 &&
        _duplicateIgnoreWindowMs == 600) {
      return 'Slow';
    }
    return null;
  }

  void _applyPastePreset(String name) {
    setState(() {
      switch (name) {
        case 'Fast':
          _delayBeforeFocusMs = 50;
          _delayBeforePasteMs = 80;
          _maxFocusVerifyAttempts = 10;
          _duplicateIgnoreWindowMs = 300;
        case 'Normal':
          _delayBeforeFocusMs = 80;
          _delayBeforePasteMs = 120;
          _maxFocusVerifyAttempts = 12;
          _duplicateIgnoreWindowMs = 350;
        case 'Safe':
          _delayBeforeFocusMs = 100;
          _delayBeforePasteMs = 180;
          _maxFocusVerifyAttempts = 15;
          _duplicateIgnoreWindowMs = 450;
        case 'Slow':
          _delayBeforeFocusMs = 150;
          _delayBeforePasteMs = 250;
          _maxFocusVerifyAttempts = 20;
          _duplicateIgnoreWindowMs = 600;
      }
    });
    _markChanged();
  }

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);

    return Scaffold(
      backgroundColor: Platform.isWindows
          ? colors.background.withValues(alpha: 0.85)
          : colors.background,
      body: Column(
        children: [
          DragToMoveArea(
            child: Container(
              height: 36,
              color: colors.surface,
              padding: const EdgeInsets.only(right: 8),
              alignment: Alignment.centerRight,
              child: Icon(
                Icons.drag_indicator_rounded,
                size: 14,
                color: colors.onSurfaceMuted.withValues(alpha: 0.4),
              ),
            ),
          ),
          Expanded(
            child: Row(
              children: [
                _buildSidebar(colors),
                VerticalDivider(
                  width: 1,
                  thickness: 0.5,
                  color: colors.divider,
                ),
                Expanded(child: _buildContent(theme, colors)),
              ],
            ),
          ),
          Divider(height: 1, thickness: 0.5, color: colors.divider),
          _buildFooter(colors),
        ],
      ),
    );
  }

  Widget _buildSidebar(AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    return Container(
      width: 220,
      color: colors.surface,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 24, 16, 4),
            child: Row(
              children: [
                Icon(Icons.settings_rounded, size: 22, color: colors.primary),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        l.settingsTitle,
                        style: TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.w600,
                          color: colors.onSurface,
                        ),
                      ),
                      Text(
                        'CopyPaste v${AppConfig.appVersion}',
                        style: TextStyle(
                          fontSize: 10,
                          color: colors.onSurfaceMuted,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
          _NavItem(
            icon: Icons.tune_rounded,
            label: l.tabGeneral,
            selected: _selectedTab == 0,
            colors: colors,
            onTap: () => setState(() => _selectedTab = 0),
          ),
          _NavItem(
            icon: Icons.archive_rounded,
            label: l.tabBackupRestore,
            selected: _selectedTab == 1,
            colors: colors,
            onTap: () => setState(() => _selectedTab = 1),
          ),
          _NavItem(
            icon: Icons.palette_rounded,
            label: l.tabAppearance,
            selected: _selectedTab == 2,
            colors: colors,
            onTap: () => setState(() => _selectedTab = 2),
          ),
          _NavItem(
            icon: Icons.keyboard_rounded,
            label: l.tabShortcuts,
            selected: _selectedTab == 3,
            colors: colors,
            onTap: () => setState(() => _selectedTab = 3),
          ),
          _NavItem(
            icon: Icons.info_outline_rounded,
            label: l.tabAbout,
            selected: _selectedTab == 4,
            colors: colors,
            onTap: () => setState(() => _selectedTab = 4),
          ),
        ],
      ),
    );
  }

  Widget _buildContent(AppThemeData theme, AppThemeColorScheme colors) {
    return switch (_selectedTab) {
      0 => _buildGeneralTab(colors),
      1 => _buildBackupTab(colors),
      2 => _buildAppearanceTab(colors),
      3 => _buildShortcutsTab(colors),
      4 => _buildAboutTab(colors),
      _ => const SizedBox.shrink(),
    };
  }

  Widget _buildGeneralTab(AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        _SectionCard(
          colors: colors,
          icon: Icons.language_rounded,
          title: l.sectionLanguage,
          children: [
            _SettingsRow(
              label: l.settingLanguage,
              colors: colors,
              trailing: SegmentedButton<String>(
                style: _segmentedStyle(colors),
                showSelectedIcon: false,
                segments: const [
                  ButtonSegment(value: 'auto', label: Text('Auto')),
                  ButtonSegment(value: 'en', label: Text('EN')),
                  ButtonSegment(value: 'es', label: Text('ES')),
                ],
                selected: {_preferredLanguage},
                onSelectionChanged: (s) {
                  setState(() => _preferredLanguage = s.first);
                  _markChanged();
                },
              ),
            ),
          ],
        ),

        _SectionCard(
          colors: colors,
          icon: Icons.power_settings_new_rounded,
          title: l.sectionStartup,
          children: [
            _SettingsRow(
              label: l.settingRunOnStartup,
              subtitle: l.subtitleStartupDesc,
              colors: colors,
              trailing: Switch(
                value: _runOnStartup,
                activeThumbColor: colors.primary,
                onChanged: (v) {
                  setState(() => _runOnStartup = v);
                  _markChanged();
                },
              ),
            ),
          ],
        ),

        _SectionCard(
          colors: colors,
          icon: Icons.keyboard_rounded,
          title: l.sectionKeyboardShortcut,
          children: [
            _SettingsRow(
              label: l.settingHotkeyShortcutLabel,
              subtitle: 'Current: ${_hotkeyString(' + ')}',
              colors: colors,
            ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                _ModifierChip(
                  label: 'Ctrl',
                  selected: _hotkeyCtrl,
                  colors: colors,
                  onTap: () {
                    setState(() => _hotkeyCtrl = !_hotkeyCtrl);
                    _markChanged();
                  },
                ),
                _ModifierChip(
                  label: Platform.isMacOS ? 'Cmd' : 'Win',
                  selected: _hotkeyWin,
                  colors: colors,
                  onTap: () {
                    setState(() => _hotkeyWin = !_hotkeyWin);
                    _markChanged();
                  },
                ),
                _ModifierChip(
                  label: Platform.isMacOS ? 'Option' : 'Alt',
                  selected: _hotkeyAlt,
                  colors: colors,
                  onTap: () {
                    setState(() => _hotkeyAlt = !_hotkeyAlt);
                    _markChanged();
                  },
                ),
                _ModifierChip(
                  label: 'Shift',
                  selected: _hotkeyShift,
                  colors: colors,
                  onTap: () {
                    setState(() => _hotkeyShift = !_hotkeyShift);
                    _markChanged();
                  },
                ),
                const SizedBox(width: 4),
                _KeySelector(
                  currentKey: _hotkeyKeyName,
                  colors: colors,
                  onChanged: (k, vk) {
                    setState(() {
                      _hotkeyKeyName = k;
                      _hotkeyVirtualKey = vk;
                    });
                    _markChanged();
                  },
                ),
              ],
            ),
          ],
        ),

        _SectionCard(
          colors: colors,
          icon: Icons.category_rounded,
          title: l.sectionCategories,
          subtitle: l.subtitleCategories,
          children: [
            ..._colorEntries(l).map(
              (e) => Padding(
                padding: const EdgeInsets.symmetric(vertical: 3),
                child: Row(
                  children: [
                    Container(
                      width: 14,
                      height: 14,
                      decoration: BoxDecoration(
                        color: e.color,
                        shape: BoxShape.circle,
                      ),
                    ),
                    const SizedBox(width: 10),
                    Expanded(
                      child: _CompactTextField(
                        initialValue: _colorLabels[e.key] ?? e.defaultName,
                        colors: colors,
                        onChanged: (v) {
                          _colorLabels[e.key] = v;
                          _markChanged();
                        },
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),

        _SectionCard(
          colors: colors,
          icon: Icons.speed_rounded,
          title: l.sectionPerformance,
          children: [
            _NumberRow(
              label: l.settingItemsPerPage,
              value: _pageSize,
              min: 5,
              max: 100,
              colors: colors,
              onChanged: (v) {
                setState(() => _pageSize = v);
                _markChanged();
              },
            ),
            _NumberRow(
              label: l.settingMemoryLimit,
              value: _maxItemsBeforeCleanup,
              min: 20,
              max: 500,
              colors: colors,
              onChanged: (v) {
                setState(() => _maxItemsBeforeCleanup = v);
                _markChanged();
              },
            ),
            _NumberRow(
              label: l.settingScrollThreshold,
              value: _scrollLoadThreshold,
              min: 50,
              max: 500,
              colors: colors,
              onChanged: (v) {
                setState(() => _scrollLoadThreshold = v);
                _markChanged();
              },
            ),
          ],
        ),

        _SectionCard(
          colors: colors,
          icon: Icons.storage_rounded,
          title: l.sectionStorage,
          children: [
            _NumberRow(
              label: l.settingRetentionDaysLabel,
              value: _retentionDays,
              min: 0,
              max: 365,
              colors: colors,
              onChanged: (v) {
                setState(() => _retentionDays = v);
                _markChanged();
              },
            ),
            const SizedBox(height: 8),
            _ActionTile(
              icon: Icons.delete_sweep_outlined,
              label: l.settingClearHistoryLabel,
              colors: colors,
              onTap: _clearHistory,
            ),
          ],
        ),

        _SectionCard(
          colors: colors,
          icon: Icons.content_paste_go_rounded,
          title: l.sectionPaste,
          subtitle: l.subtitlePastePreset,
          children: [
            _SettingsRow(
              label: l.settingPasteSpeed,
              subtitle: l.subtitlePasteSpeed,
              colors: colors,
              trailing: _PresetDropdown(
                value: _pastePresetName,
                items: const ['Fast', 'Normal', 'Safe', 'Slow'],
                colors: colors,
                onChanged: _applyPastePreset,
              ),
            ),
            const SizedBox(height: 6),
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: colors.warning.withValues(alpha: 0.08),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(
                  color: colors.warning.withValues(alpha: 0.3),
                ),
              ),
              child: Text(
                '\u26a0\ufe0f Fast: May cause strange behavior in heavy apps.\n'
                '\u26a0\ufe0f Slow: May feel like a failure on modern computers.',
                style: TextStyle(
                  fontSize: 10.5,
                  color: colors.onSurfaceVariant,
                ),
              ),
            ),
          ],
        ),

        const SizedBox(height: 16),
      ],
    );
  }

  Widget _buildBackupTab(AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        _SectionCard(
          colors: colors,
          icon: Icons.archive_rounded,
          title: l.sectionBackupRestore,
          subtitle: l.subtitleBackup,
          children: [
            if (_lastBackupDateUtc != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text(
                  l.backupLastDate(_formatDate(_lastBackupDateUtc!)),
                  style: TextStyle(fontSize: 11, color: colors.onSurfaceMuted),
                ),
              )
            else
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text(
                  l.backupNone,
                  style: TextStyle(fontSize: 11, color: colors.onSurfaceMuted),
                ),
              ),
            Row(
              children: [
                Expanded(
                  child: _ActionTile(
                    icon: Icons.backup_rounded,
                    label: l.backupCreateLabel,
                    colors: colors,
                    onTap: _createBackup,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: _ActionTile(
                    icon: Icons.restore_rounded,
                    label: l.backupRestoreLabel,
                    colors: colors,
                    onTap: _restoreBackup,
                  ),
                ),
              ],
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildShortcutsTab(AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        _SectionCard(
          colors: colors,
          icon: Icons.keyboard_rounded,
          title: l.sectionShortcuts,
          children: [
            _ShortcutRow(
              keys: _hotkeyString(),
              description: l.shortcutOpenClose,
              colors: colors,
            ),
            _ShortcutRow(
              keys: '\u2191 / \u2193',
              description: l.shortcutArrows,
              colors: colors,
            ),
            _ShortcutRow(
              keys: 'Enter',
              description: l.shortcutEnter,
              colors: colors,
            ),
            _ShortcutRow(
              keys: 'Delete',
              description: l.shortcutDelete,
              colors: colors,
            ),
            _ShortcutRow(keys: 'P', description: l.shortcutPin, colors: colors),
            _ShortcutRow(
              keys: 'E',
              description: l.shortcutEdit,
              colors: colors,
            ),
            _ShortcutRow(
              keys: '\u2192',
              description: l.shortcutExpand,
              colors: colors,
            ),
            _ShortcutRow(
              keys: 'Escape',
              description: l.shortcutEscape,
              colors: colors,
            ),
            _ShortcutRow(
              keys: Platform.isMacOS ? 'Cmd+1' : 'Ctrl+1',
              description: l.shortcutTab1,
              colors: colors,
            ),
            _ShortcutRow(
              keys: Platform.isMacOS ? 'Cmd+2' : 'Ctrl+2',
              description: l.shortcutTab2,
              colors: colors,
            ),
            _ShortcutRow(
              keys: 'Shift+Tab',
              description: l.shortcutFocusSearch,
              colors: colors,
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildAppearanceTab(AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        _SectionCard(
          colors: colors,
          icon: Icons.aspect_ratio_rounded,
          title: l.sectionAppearance,
          children: [
            _ThemeRow(
              label: l.settingTheme,
              value: _themeMode,
              colors: colors,
              options: [
                (value: 'light', label: l.themeLight),
                (value: 'dark', label: l.themeDark),
                (value: 'auto', label: l.themeAuto),
              ],
              onChanged: (v) {
                setState(() => _themeMode = v);
                _markChanged();
              },
            ),
            _NumberRow(
              label: l.settingPanelWidth,
              value: _popupWidth,
              min: 300,
              max: 600,
              colors: colors,
              onChanged: (v) {
                setState(() => _popupWidth = v);
                _markChanged();
              },
            ),
            _NumberRow(
              label: l.settingPanelHeight,
              value: _popupHeight,
              min: 300,
              max: 800,
              colors: colors,
              onChanged: (v) {
                setState(() => _popupHeight = v);
                _markChanged();
              },
            ),
            _NumberRow(
              label: l.settingLinesCollapsed,
              value: _cardMinLines,
              min: 1,
              max: 10,
              colors: colors,
              onChanged: (v) {
                setState(() => _cardMinLines = v);
                _markChanged();
              },
            ),
            _NumberRow(
              label: l.settingLinesExpanded,
              value: _cardMaxLines,
              min: 1,
              max: 20,
              colors: colors,
              onChanged: (v) {
                setState(() => _cardMaxLines = v);
                _markChanged();
              },
            ),
          ],
        ),
        _SectionCard(
          colors: colors,
          icon: Icons.toggle_on_rounded,
          title: l.sectionBehavior,
          children: [
            _ToggleRow(
              label: l.settingHideOnDeactivate,
              subtitle: l.subtitleHideOnDeactivate,
              value: _hideOnDeactivate,
              colors: colors,
              onChanged: (v) {
                setState(() => _hideOnDeactivate = v);
                _markChanged();
              },
            ),
            _ToggleRow(
              label: l.settingScrollToTopOnOpen,
              subtitle: l.subtitleScrollToTopOnOpen,
              value: _resetScrollOnShow,
              colors: colors,
              onChanged: (v) {
                setState(() => _resetScrollOnShow = v);
                _markChanged();
              },
            ),
            _ToggleRow(
              label: l.settingClearSearchOnOpen,
              subtitle: l.subtitleClearSearchOnOpen,
              value: _resetSearchOnShow,
              colors: colors,
              onChanged: (v) {
                setState(() => _resetSearchOnShow = v);
                _markChanged();
              },
            ),
            if (Platform.isWindows)
              _ToggleRow(
                label: l.settingShowInTaskbar,
                subtitle: l.subtitleShowInTaskbar,
                value: _showInTaskbar,
                colors: colors,
                onChanged: (v) {
                  setState(() => _showInTaskbar = v);
                  _markChanged();
                },
              ),
            if (Platform.isWindows)
              _ToggleRow(
                label: l.settingShowTrayIcon,
                subtitle: l.subtitleShowTrayIcon,
                value: _showTrayIcon,
                colors: colors,
                onChanged: (v) {
                  setState(() => _showTrayIcon = v);
                  _markChanged();
                },
              ),
          ],
        ),

        _SectionCard(
          colors: colors,
          icon: Icons.restart_alt_rounded,
          title: l.sectionReset,
          children: [
            _ActionTile(
              icon: Icons.settings_backup_restore_rounded,
              label: l.resetSoftLabel,
              subtitle: l.resetSoftSubtitle,
              colors: colors,
              onTap: _softReset,
            ),
            _ActionTile(
              icon: Icons.delete_forever_rounded,
              label: l.resetHardLabel,
              subtitle: l.resetHardSubtitle,
              colors: colors,
              onTap: _hardReset,
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildAboutTab(AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        _SectionCard(
          colors: colors,
          icon: Icons.info_outline_rounded,
          title: l.sectionAbout,
          children: [
            Text(
              l.aboutDescription,
              style: TextStyle(
                fontSize: 12,
                color: colors.onSurfaceVariant,
                height: 1.5,
              ),
            ),
            const SizedBox(height: 14),
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                _AboutBadge(
                  icon: Icons.new_releases_outlined,
                  label: 'v${AppConfig.appVersion}',
                  colors: colors,
                ),
                _AboutBadge(
                  icon: Icons.lock_outline_rounded,
                  label: l.aboutTagLocal,
                  colors: colors,
                ),
                _AboutBadge(
                  icon: Icons.code_rounded,
                  label: l.aboutTagOpenSource,
                  colors: colors,
                ),
                _AboutBadge(
                  icon: Icons.favorite_border_rounded,
                  label: l.aboutTagFree,
                  colors: colors,
                ),
              ],
            ),
          ],
        ),
        _SectionCard(
          colors: colors,
          icon: Icons.link_rounded,
          title: l.sectionLinks,
          children: [
            _ActionTile(
              icon: Icons.code_rounded,
              label: l.linkGitHub,
              colors: colors,
              onTap: () => _openUrl('https://github.com/rgdevment/CopyPaste'),
            ),
            _ActionTile(
              icon: Icons.coffee_rounded,
              label: l.linkCoffee,
              colors: colors,
              onTap: () => _openUrl('https://buymeacoffee.com/rgdevment'),
            ),
          ],
        ),
        _SectionCard(
          colors: colors,
          icon: Icons.shield_outlined,
          title: l.sectionPrivacy,
          children: [
            Text(
              l.privacyStatement,
              style: TextStyle(
                fontSize: 12,
                color: colors.onSurfaceVariant,
                height: 1.5,
              ),
            ),
            const SizedBox(height: 4),
            _ActionTile(
              icon: Icons.open_in_new_rounded,
              label: l.privacyPolicy,
              colors: colors,
              onTap: () => _openUrl(
                'https://github.com/rgdevment/CopyPaste/blob/main/PRIVACY.md',
              ),
            ),
          ],
        ),
        _SectionCard(
          colors: colors,
          icon: Icons.help_outline_rounded,
          title: l.sectionSupport,
          children: [
            _ActionTile(
              icon: Icons.download_rounded,
              label: l.supportExportLogs,
              subtitle: l.supportExportLogsSubtitle,
              colors: colors,
              onTap: _exportLogs,
            ),
            _ActionTile(
              icon: Icons.folder_open_rounded,
              label: l.supportOpenLogsFolder,
              subtitle: l.supportOpenLogsFolderSubtitle,
              colors: colors,
              onTap: _openLogsFolder,
            ),
            _ActionTile(
              icon: Icons.bug_report_outlined,
              label: l.supportGitHub,
              colors: colors,
              onTap: () =>
                  _openUrl('https://github.com/rgdevment/CopyPaste/issues'),
            ),
          ],
        ),
        _SectionCard(
          colors: colors,
          icon: Icons.apps_rounded,
          title: l.sectionOtherTools,
          children: [
            _ActionTile(
              icon: Icons.open_in_new_rounded,
              label: l.otherToolLinkUnbound,
              subtitle: l.otherToolLinkUnboundDesc,
              colors: colors,
              leading: Padding(
                padding: const EdgeInsets.only(top: 1),
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(4),
                  child: Image.asset(
                    'assets/icons/icon_linkunbound.png',
                    width: 28,
                    height: 28,
                  ),
                ),
              ),
              onTap: () => _openUrl('https://github.com/rgdevment/LinkUnbound'),
            ),
          ],
        ),
        Padding(
          padding: const EdgeInsets.only(top: 12, left: 4),
          child: Text(
            l.aboutLicense,
            style: TextStyle(fontSize: 10.5, color: colors.onSurfaceMuted),
          ),
        ),
      ],
    );
  }

  Widget _buildFooter(AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    return Container(
      height: 52,
      padding: const EdgeInsets.symmetric(horizontal: 20),
      color: colors.surface,
      child: Row(
        children: [
          _SmallButton(
            label: l.buttonReset,
            colors: colors,
            onTap: _resetToDefaults,
          ),
          const Spacer(),
          if (_hotkeyChanged)
            Padding(
              padding: const EdgeInsets.only(right: 12),
              child: Text(
                l.hotkeyWillApply,
                style: TextStyle(fontSize: 10, color: colors.onSurfaceMuted),
              ),
            ),
          _SmallButton(
            label: l.buttonCancel,
            colors: colors,
            onTap: () => Navigator.of(context).pop(),
          ),
          const SizedBox(width: 8),
          _SmallButton(
            label: l.buttonSave,
            colors: colors,
            primary: true,
            onTap: _hasChanges
                ? () async {
                    try {
                      await _save();
                      if (mounted) Navigator.of(context).pop();
                    } catch (e) {
                      if (mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(content: Text('Save failed: $e')),
                        );
                      }
                    }
                  }
                : null,
          ),
        ],
      ),
    );
  }

  Future<void> _exportLogs() async {
    final l = AppLocalizations.of(context);
    try {
      final ts = DateTime.now()
          .toIso8601String()
          .replaceAll(':', '-')
          .split('.')
          .first;
      final fileName = 'CopyPaste_logs_$ts.zip';
      final savePath = _resolveDownloadsPath(fileName);

      final count = await SupportService.exportLogs(
        widget.storage,
        AppConfig.appVersion,
        savePath,
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            count > 0 ? l.supportExportSuccess : l.supportExportEmpty,
          ),
          action: SnackBarAction(
            label: l.supportShowInFiles,
            onPressed: () => SupportService.revealFile(savePath),
          ),
          duration: const Duration(seconds: 5),
        ),
      );
    } catch (e, s) {
      AppLogger.exception(e, s, '_exportLogs');
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(l.supportExportError),
          duration: const Duration(seconds: 3),
        ),
      );
    }
  }

  String _resolveDownloadsPath(String fileName) {
    final String base;
    if (Platform.isWindows) {
      base = p.join(Platform.environment['USERPROFILE'] ?? '', 'Downloads');
    } else {
      base = p.join(Platform.environment['HOME'] ?? '', 'Downloads');
    }
    final dir = Directory(base);
    if (dir.existsSync()) return p.join(base, fileName);
    return p.join(widget.storage.logsPath, fileName);
  }

  Future<void> _openLogsFolder() async {
    try {
      await SupportService.openLogsFolder(widget.storage);
    } catch (e, s) {
      AppLogger.exception(e, s, '_openLogsFolder');
    }
  }

  Future<void> _softReset() async {
    final l = AppLocalizations.of(context);
    final confirmed = await _showConfirmDialog(
      l.resetSoftConfirmTitle,
      l.resetSoftConfirmMessage,
      l.resetConfirmButton,
    );
    if (confirmed == true) await widget.onSoftReset();
  }

  Future<void> _hardReset() async {
    final l = AppLocalizations.of(context);
    final confirmed = await _showConfirmDialog(
      l.resetHardConfirmTitle,
      l.resetHardConfirmMessage,
      l.resetConfirmButton,
    );
    if (confirmed == true) await widget.onHardReset();
  }

  Future<void> _clearHistory() async {
    final l = AppLocalizations.of(context);
    final confirmed = await _showConfirmDialog(
      l.clearHistoryConfirmTitle,
      l.clearHistoryConfirmMessage,
      l.clearHistoryConfirmButton,
    );
    if (confirmed == true) {
      await widget.clipboardService.clearUnpinnedHistory();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(l.historyCleared),
            duration: const Duration(seconds: 2),
          ),
        );
      }
    }
  }

  Future<void> _createBackup() async {
    try {
      final ts = DateTime.now()
          .toIso8601String()
          .replaceAll(':', '-')
          .split('.')
          .first;
      final suggestedName = 'CopyPaste_Backup_$ts';

      final path = await FilePicker.platform.saveFile(
        dialogTitle: 'Save Backup',
        fileName: '$suggestedName.zip',
        type: FileType.custom,
        allowedExtensions: ['zip'],
      );
      if (path == null) return;

      final count = await widget.clipboardService.getItemCount();
      await BackupService.createBackup(
        path,
        widget.storage,
        AppConfig.appVersion,
        itemCount: count,
        walCheckpoint: widget.clipboardService.walCheckpoint,
      );
      setState(() => _lastBackupDateUtc = DateTime.now().toUtc());
      _markChanged();
      if (mounted) {
        final l = AppLocalizations.of(context);
        final filename = path.split(Platform.pathSeparator).last;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(l.backupSavedFile(filename)),
            duration: const Duration(seconds: 3),
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        final l = AppLocalizations.of(context);
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(l.backupError)));
      }
    }
  }

  Future<void> _restoreBackup() async {
    final l = AppLocalizations.of(context);

    final result = await FilePicker.platform.pickFiles(
      dialogTitle: l.restoreDialogTitle,
      type: FileType.custom,
      allowedExtensions: ['zip'],
    );
    if (result == null || result.files.isEmpty) return;

    final path = result.files.single.path;
    if (path == null || !File(path).existsSync()) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(l.restoreFileNotFound)));
      }
      return;
    }

    if (!mounted) return;
    final colors = CopyPasteTheme.colorsOf(context);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: colors.cardBackground,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        title: Text(
          l.restoreDialogTitle,
          style: TextStyle(fontSize: 14, color: colors.onSurface),
        ),
        content: Text(
          l.restoreDialogWarning,
          style: TextStyle(fontSize: 12, color: colors.onSurfaceVariant),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: Text(
              l.buttonCancel,
              style: TextStyle(color: colors.onSurfaceMuted),
            ),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: Text(
              l.buttonRestore,
              style: TextStyle(color: colors.danger),
            ),
          ),
        ],
      ),
    );
    if (confirmed != true) return;

    try {
      final manifest = await BackupService.restoreBackup(path, widget.storage);
      if (manifest != null && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(l.restoreRestartRequired),
            duration: const Duration(seconds: 2),
          ),
        );
        await Future<void>.delayed(const Duration(seconds: 2));
        exit(0);
      } else if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(l.restoreCompleted)));
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(l.restoreError)));
      }
    }
  }

  void _openUrl(String url) {
    UrlHelper.open(url);
  }

  String _formatDate(DateTime dt) =>
      '${dt.day.toString().padLeft(2, '0')}/'
      '${dt.month.toString().padLeft(2, '0')}/${dt.year}';

  Future<bool?> _showConfirmDialog(
    String title,
    String message,
    String confirmLabel,
  ) {
    final colors = CopyPasteTheme.colorsOf(context);
    final l = AppLocalizations.of(context);
    return showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: colors.cardBackground,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        title: Text(
          title,
          style: TextStyle(fontSize: 14, color: colors.onSurface),
        ),
        content: Text(
          message,
          style: TextStyle(fontSize: 12.5, color: colors.onSurfaceVariant),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: Text(
              l.buttonCancel,
              style: TextStyle(color: colors.onSurfaceMuted),
            ),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: Text(confirmLabel, style: TextStyle(color: colors.danger)),
          ),
        ],
      ),
    );
  }

  ButtonStyle _segmentedStyle(AppThemeColorScheme colors) =>
      SegmentedButton.styleFrom(
        foregroundColor: colors.onSurface,
        selectedForegroundColor: colors.primary,
        selectedBackgroundColor: colors.primary.withValues(alpha: 0.12),
        side: BorderSide(color: colors.divider),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
        textStyle: const TextStyle(fontSize: 12),
      );

  static List<({String key, String defaultName, Color color})> _colorEntries(
    AppLocalizations l,
  ) => [
    (key: 'Red', defaultName: l.colorRed, color: const Color(0xFFE53935)),
    (key: 'Green', defaultName: l.colorGreen, color: const Color(0xFF43A047)),
    (key: 'Purple', defaultName: l.colorPurple, color: const Color(0xFF8E24AA)),
    (key: 'Yellow', defaultName: l.colorYellow, color: const Color(0xFFFDD835)),
    (key: 'Blue', defaultName: l.colorBlue, color: const Color(0xFF1E88E5)),
    (key: 'Orange', defaultName: l.colorOrange, color: const Color(0xFFFB8C00)),
  ];
}

class _NavItem extends StatefulWidget {
  const _NavItem({
    required this.icon,
    required this.label,
    required this.selected,
    required this.colors,
    required this.onTap,
  });

  final IconData icon;
  final String label;
  final bool selected;
  final AppThemeColorScheme colors;
  final VoidCallback onTap;

  @override
  State<_NavItem> createState() => _NavItemState();
}

class _NavItemState extends State<_NavItem> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final bg = widget.selected
        ? widget.colors.primary.withValues(alpha: 0.12)
        : (_hovering
              ? widget.colors.onSurface.withValues(alpha: 0.05)
              : Colors.transparent);
    final fg = widget.selected
        ? widget.colors.primary
        : widget.colors.onSurface;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: widget.onTap,
        child: Container(
          margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 2),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: bg,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Row(
            children: [
              Icon(widget.icon, size: 18, color: fg),
              const SizedBox(width: 12),
              Flexible(
                child: Text(
                  widget.label,
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: widget.selected
                        ? FontWeight.w600
                        : FontWeight.w400,
                    color: fg,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SectionCard extends StatelessWidget {
  const _SectionCard({
    required this.colors,
    required this.icon,
    required this.title,
    required this.children,
    this.subtitle,
  });

  final AppThemeColorScheme colors;
  final IconData icon;
  final String title;
  final String? subtitle;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: colors.cardBackground,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: colors.cardBorder),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, size: 14, color: colors.onSurfaceMuted),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  title,
                  style: TextStyle(
                    fontSize: 10,
                    fontWeight: FontWeight.w600,
                    color: colors.onSurfaceMuted,
                    letterSpacing: 1.0,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
          if (subtitle != null)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                subtitle!,
                style: TextStyle(fontSize: 11, color: colors.onSurfaceMuted),
              ),
            ),
          const SizedBox(height: 12),
          ...children,
        ],
      ),
    );
  }
}

class _SettingsRow extends StatelessWidget {
  const _SettingsRow({
    required this.label,
    required this.colors,
    this.subtitle,
    this.trailing,
  });

  final String label;
  final String? subtitle;
  final AppThemeColorScheme colors;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  label,
                  style: TextStyle(fontSize: 12.5, color: colors.onSurface),
                ),
                if (subtitle != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Text(
                      subtitle!,
                      style: TextStyle(
                        fontSize: 10.5,
                        color: colors.onSurfaceMuted,
                      ),
                    ),
                  ),
              ],
            ),
          ),
          ?trailing,
        ],
      ),
    );
  }
}

class _ToggleRow extends StatelessWidget {
  const _ToggleRow({
    required this.label,
    required this.value,
    required this.colors,
    required this.onChanged,
    this.subtitle,
  });

  final String label;
  final String? subtitle;
  final bool value;
  final AppThemeColorScheme colors;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  label,
                  style: TextStyle(fontSize: 12.5, color: colors.onSurface),
                ),
                if (subtitle != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Text(
                      subtitle!,
                      style: TextStyle(
                        fontSize: 10.5,
                        color: colors.onSurfaceMuted,
                      ),
                    ),
                  ),
              ],
            ),
          ),
          Switch(
            value: value,
            activeThumbColor: colors.primary,
            onChanged: onChanged,
          ),
        ],
      ),
    );
  }
}

class _ThemeRow extends StatelessWidget {
  const _ThemeRow({
    required this.label,
    required this.value,
    required this.colors,
    required this.options,
    required this.onChanged,
  });

  final String label;
  final String value;
  final AppThemeColorScheme colors;
  final List<({String value, String label})> options;
  final void Function(String) onChanged;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: TextStyle(fontSize: 13, color: colors.onSurface),
            ),
          ),
          Container(
            decoration: BoxDecoration(
              color: colors.surfaceVariant,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(
                color: colors.onSurface.withValues(alpha: 0.08),
              ),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                for (final opt in options)
                  GestureDetector(
                    onTap: () => onChanged(opt.value),
                    child: AnimatedContainer(
                      duration: const Duration(milliseconds: 150),
                      padding: const EdgeInsets.symmetric(
                        horizontal: 14,
                        vertical: 6,
                      ),
                      decoration: BoxDecoration(
                        color: value == opt.value
                            ? colors.primary.withValues(alpha: 0.12)
                            : Colors.transparent,
                        borderRadius: BorderRadius.circular(7),
                        border: value == opt.value
                            ? Border.all(
                                color: colors.primary.withValues(alpha: 0.3),
                              )
                            : null,
                      ),
                      child: Text(
                        opt.label,
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: value == opt.value
                              ? FontWeight.w600
                              : FontWeight.w400,
                          color: value == opt.value
                              ? colors.primary
                              : colors.onSurfaceMuted,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _NumberRow extends StatelessWidget {
  const _NumberRow({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.colors,
    required this.onChanged,
  });

  final String label;
  final int value;
  final int min;
  final int max;
  final AppThemeColorScheme colors;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: TextStyle(fontSize: 12.5, color: colors.onSurface),
            ),
          ),
          SizedBox(
            width: 130,
            height: 30,
            child: Row(
              children: [
                _StepButton(
                  icon: Icons.remove,
                  colors: colors,
                  isLeft: true,
                  onTap: value > min ? () => onChanged(value - 1) : null,
                ),
                Expanded(
                  child: Container(
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      border: Border.symmetric(
                        horizontal: BorderSide(color: colors.divider),
                      ),
                      color: colors.surface,
                    ),
                    child: Text(
                      '$value',
                      style: TextStyle(
                        fontSize: 12,
                        color: colors.onSurface,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                  ),
                ),
                _StepButton(
                  icon: Icons.add,
                  colors: colors,
                  isLeft: false,
                  onTap: value < max ? () => onChanged(value + 1) : null,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _StepButton extends StatelessWidget {
  const _StepButton({
    required this.icon,
    required this.colors,
    required this.isLeft,
    this.onTap,
  });

  final IconData icon;
  final AppThemeColorScheme colors;
  final bool isLeft;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        width: 30,
        height: 30,
        decoration: BoxDecoration(
          color: colors.surface,
          border: Border.all(color: colors.divider),
          borderRadius: isLeft
              ? const BorderRadius.horizontal(left: Radius.circular(6))
              : const BorderRadius.horizontal(right: Radius.circular(6)),
        ),
        child: Icon(
          icon,
          size: 14,
          color: onTap != null ? colors.onSurface : colors.onSurfaceMuted,
        ),
      ),
    );
  }
}

class _CompactTextField extends StatefulWidget {
  const _CompactTextField({
    required this.initialValue,
    required this.colors,
    required this.onChanged,
  });

  final String initialValue;
  final AppThemeColorScheme colors;
  final ValueChanged<String> onChanged;

  @override
  State<_CompactTextField> createState() => _CompactTextFieldState();
}

class _CompactTextFieldState extends State<_CompactTextField> {
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialValue);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: _controller,
      maxLength: 20,
      style: TextStyle(fontSize: 12, color: widget.colors.onSurface),
      decoration: InputDecoration(
        isDense: true,
        counterText: '',
        contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: BorderSide(color: widget.colors.divider),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: BorderSide(color: widget.colors.divider),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: BorderSide(color: widget.colors.primary),
        ),
      ),
      onChanged: widget.onChanged,
    );
  }
}

class _PresetDropdown extends StatelessWidget {
  const _PresetDropdown({
    required this.value,
    required this.items,
    required this.colors,
    required this.onChanged,
  });

  final String? value;
  final List<String> items;
  final AppThemeColorScheme colors;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: BoxDecoration(
        color: colors.surface,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: colors.divider),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<String>(
          value: value,
          hint: Text(
            'Custom',
            style: TextStyle(fontSize: 12, color: colors.onSurfaceMuted),
          ),
          isDense: true,
          dropdownColor: colors.cardBackground,
          style: TextStyle(fontSize: 12, color: colors.onSurface),
          icon: Icon(
            Icons.arrow_drop_down,
            size: 16,
            color: colors.onSurfaceMuted,
          ),
          items: items
              .map((i) => DropdownMenuItem(value: i, child: Text(i)))
              .toList(),
          onChanged: (v) {
            if (v != null) onChanged(v);
          },
        ),
      ),
    );
  }
}

class _ActionTile extends StatefulWidget {
  const _ActionTile({
    required this.icon,
    required this.label,
    required this.colors,
    required this.onTap,
    this.subtitle,
    this.leading,
  });

  final IconData icon;
  final String label;
  final String? subtitle;
  final Widget? leading;
  final AppThemeColorScheme colors;
  final VoidCallback onTap;

  @override
  State<_ActionTile> createState() => _ActionTileState();
}

class _ActionTileState extends State<_ActionTile> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: widget.onTap,
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 10),
          decoration: BoxDecoration(
            color: _hovering
                ? widget.colors.onSurface.withValues(alpha: 0.05)
                : Colors.transparent,
            borderRadius: BorderRadius.circular(6),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              widget.leading ??
                  Padding(
                    padding: const EdgeInsets.only(top: 1),
                    child: Icon(
                      widget.icon,
                      size: 16,
                      color: widget.colors.onSurfaceMuted,
                    ),
                  ),
              const SizedBox(width: 10),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      widget.label,
                      style: TextStyle(
                        fontSize: 12.5,
                        color: widget.colors.onSurface,
                      ),
                    ),
                    if (widget.subtitle != null) ...[
                      const SizedBox(height: 2),
                      Text(
                        widget.subtitle!,
                        style: TextStyle(
                          fontSize: 11,
                          color: widget.colors.onSurfaceMuted,
                          height: 1.4,
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _AboutBadge extends StatelessWidget {
  const _AboutBadge({
    required this.icon,
    required this.label,
    required this.colors,
  });

  final IconData icon;
  final String label;
  final AppThemeColorScheme colors;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        border: Border.all(color: colors.cardBorder),
        borderRadius: BorderRadius.circular(20),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 11, color: colors.onSurfaceMuted),
          const SizedBox(width: 5),
          Text(
            label,
            style: TextStyle(fontSize: 10.5, color: colors.onSurfaceVariant),
          ),
        ],
      ),
    );
  }
}

class _ModifierChip extends StatelessWidget {
  const _ModifierChip({
    required this.label,
    required this.selected,
    required this.colors,
    required this.onTap,
  });

  final String label;
  final bool selected;
  final AppThemeColorScheme colors;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: selected
                ? colors.primary.withValues(alpha: 0.15)
                : colors.onSurface.withValues(alpha: 0.05),
            borderRadius: BorderRadius.circular(6),
            border: Border.all(
              color: selected
                  ? colors.primary.withValues(alpha: 0.4)
                  : colors.divider,
            ),
          ),
          child: Text(
            label,
            style: TextStyle(
              fontSize: 11,
              fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
              color: selected ? colors.primary : colors.onSurfaceMuted,
            ),
          ),
        ),
      ),
    );
  }
}

class _KeySelector extends StatelessWidget {
  const _KeySelector({
    required this.currentKey,
    required this.colors,
    required this.onChanged,
  });

  final String currentKey;
  final AppThemeColorScheme colors;
  final void Function(String key, int virtualKey) onChanged;

  static const _keys = [
    ('A', 0x41),
    ('B', 0x42),
    ('C', 0x43),
    ('D', 0x44),
    ('E', 0x45),
    ('F', 0x46),
    ('G', 0x47),
    ('H', 0x48),
    ('I', 0x49),
    ('J', 0x4A),
    ('K', 0x4B),
    ('L', 0x4C),
    ('M', 0x4D),
    ('N', 0x4E),
    ('O', 0x4F),
    ('P', 0x50),
    ('Q', 0x51),
    ('R', 0x52),
    ('S', 0x53),
    ('T', 0x54),
    ('U', 0x55),
    ('V', 0x56),
    ('W', 0x57),
    ('X', 0x58),
    ('Y', 0x59),
    ('Z', 0x5A),
  ];

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: BoxDecoration(
        color: colors.primary.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: colors.primary.withValues(alpha: 0.3)),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<String>(
          value: currentKey,
          isDense: true,
          dropdownColor: colors.cardBackground,
          style: TextStyle(
            fontSize: 12,
            fontWeight: FontWeight.w600,
            color: colors.primary,
          ),
          icon: Icon(Icons.arrow_drop_down, size: 16, color: colors.primary),
          items: _keys
              .map((k) => DropdownMenuItem(value: k.$1, child: Text(k.$1)))
              .toList(),
          onChanged: (value) {
            if (value == null) return;
            final entry = _keys.firstWhere((k) => k.$1 == value);
            onChanged(entry.$1, entry.$2);
          },
        ),
      ),
    );
  }
}

class _ShortcutRow extends StatelessWidget {
  const _ShortcutRow({
    required this.keys,
    required this.description,
    required this.colors,
  });

  final String keys;
  final String description;
  final AppThemeColorScheme colors;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        children: [
          SizedBox(
            width: 100,
            child: Text(
              keys,
              style: TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.w500,
                color: colors.onSurfaceVariant,
                fontFamily: 'Consolas',
              ),
            ),
          ),
          Expanded(
            child: Text(
              description,
              style: TextStyle(fontSize: 11, color: colors.onSurfaceMuted),
            ),
          ),
        ],
      ),
    );
  }
}

class _SmallButton extends StatefulWidget {
  const _SmallButton({
    required this.label,
    required this.colors,
    this.onTap,
    this.primary = false,
  });

  final String label;
  final AppThemeColorScheme colors;
  final VoidCallback? onTap;
  final bool primary;

  @override
  State<_SmallButton> createState() => _SmallButtonState();
}

class _SmallButtonState extends State<_SmallButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onTap != null;
    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      cursor: enabled ? SystemMouseCursors.click : SystemMouseCursors.basic,
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 100),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: BoxDecoration(
            color: widget.primary
                ? (enabled
                      ? (_hovering
                            ? widget.colors.primary.withValues(alpha: 0.9)
                            : widget.colors.primary)
                      : widget.colors.primary.withValues(alpha: 0.4))
                : (_hovering
                      ? widget.colors.onSurface.withValues(alpha: 0.08)
                      : Colors.transparent),
            borderRadius: BorderRadius.circular(6),
          ),
          child: Text(
            widget.label,
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w500,
              color: widget.primary
                  ? widget.colors.onPrimary
                  : widget.colors.onSurface,
            ),
          ),
        ),
      ),
    );
  }
}
