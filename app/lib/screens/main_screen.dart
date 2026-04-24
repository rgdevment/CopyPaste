import 'dart:async';
import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../helpers/url_helper.dart';
import '../l10n/app_localizations.dart';
import '../services/auto_update_service.dart';
import '../services/release_manifest_service.dart';
import '../theme/app_theme_data.dart';
import '../theme/theme_provider.dart';
import '../widgets/clipboard_card.dart';
import '../widgets/empty_state.dart';
import '../widgets/filter_bar.dart';
import '../widgets/filter_tab_bar.dart';
import '../widgets/label_color_dialog.dart';
import '../widgets/title_bar.dart';

enum ClipboardTab { recent, pinned }

class MainScreen extends StatefulWidget {
  const MainScreen({
    required this.clipboardService,
    required this.onPaste,
    required this.onPastePlain,
    required this.onExit,
    required this.onSettings,
    this.resetScrollOnShow = true,
    this.resetSearchOnShow = true,
    this.resetFiltersOnShow = true,
    this.cardMinLines = 2,
    this.cardMaxLines = 5,
    this.colorLabels = const {},
    this.showHint = false,
    this.onDismissHint,
    this.updateVersion,
    this.updateSeverity,
    super.key,
  });

  final ClipboardService clipboardService;
  final void Function(ClipboardItem item) onPaste;
  final void Function(ClipboardItem item) onPastePlain;
  final VoidCallback onExit;
  final VoidCallback onSettings;
  final bool resetScrollOnShow;
  final bool resetSearchOnShow;
  final bool resetFiltersOnShow;
  final int cardMinLines;
  final int cardMaxLines;
  final Map<String, String> colorLabels;
  final bool showHint;
  final VoidCallback? onDismissHint;
  final String? updateVersion;
  final ManifestSeverity? updateSeverity;

  @override
  State<MainScreen> createState() => MainScreenState();
}

class MainScreenState extends State<MainScreen> {
  final _scrollController = ScrollController();
  final _searchController = TextEditingController();
  final _focusNode = FocusNode();
  final _searchFocusNode = FocusNode();
  final _filterBarKey = GlobalKey<FilterBarState>();
  final _cardKeys = <String, GlobalKey>{};

  ClipboardTab _currentTab = ClipboardTab.recent;
  List<ClipboardItem> _items = [];
  bool _loading = false;
  bool _pendingReload = false;
  int _selectedIndex = -1;
  int _expandedIndex = -1;
  Timer? _reloadDebounce;

  String _searchQuery = '';
  List<ClipboardContentType> _typeFilters = [];
  List<CardColor> _colorFilters = [];

  StreamSubscription<ClipboardItem>? _addedSub;
  StreamSubscription<ClipboardItem>? _reactivatedSub;

  static const int _pageSize = 30;
  int _currentPage = 0;
  bool _hasMore = true;

  bool _isFirstRender = true;

  @override
  void initState() {
    super.initState();
    _addedSub = widget.clipboardService.onItemAdded.listen((_) => _reload());
    _reactivatedSub = widget.clipboardService.onItemReactivated.listen(
      (_) => _reload(),
    );
    _searchFocusNode.onKeyEvent = _onSearchKeyEvent;
    _scrollController.addListener(_onScroll);
    _loadItems();
  }

  @override
  void dispose() {
    _addedSub?.cancel();
    _reactivatedSub?.cancel();
    _reloadDebounce?.cancel();
    _scrollController.dispose();
    _searchController.dispose();
    _focusNode.dispose();
    _searchFocusNode.dispose();
    super.dispose();
  }

  void onWindowShow() {
    if (widget.resetFiltersOnShow) {
      _typeFilters = [];
      _colorFilters = [];
      _currentTab = ClipboardTab.recent;
    }
    _reload();
    if (widget.resetScrollOnShow && _scrollController.hasClients) {
      _scrollController.jumpTo(0);
    }
    if (widget.resetSearchOnShow) {
      _searchController.clear();
      _searchQuery = '';
    }
    _searchFocusNode.requestFocus();
  }

  void onWindowHide() {
    _selectedIndex = -1;
    _expandedIndex = -1;
    if (_items.length > _pageSize) {
      _items = _items.sublist(0, _pageSize);
      _currentPage = 0;
      _hasMore = true;
    }
    setState(() {});
  }

  Future<void> _loadItems() async {
    if (_loading) return;
    _pendingReload = false;
    setState(() => _loading = true);

    try {
      final items = await widget.clipboardService.getHistoryAdvanced(
        query: _searchQuery.isEmpty ? null : _searchQuery,
        types: _typeFilters.isEmpty ? null : _typeFilters,
        colors: _colorFilters.isEmpty ? null : _colorFilters,
        isPinned: _currentTab == ClipboardTab.pinned ? true : null,
        limit: _pageSize,
        skip: _currentPage * _pageSize,
      );

      setState(() {
        if (_currentPage == 0) {
          _items = items;
          final activeIds = items.map((e) => e.id).toSet();
          _cardKeys.removeWhere((id, _) => !activeIds.contains(id));
        } else {
          _items.addAll(items);
        }
        _hasMore = items.length >= _pageSize;
        _loading = false;
      });
    } catch (e) {
      AppLogger.error('Failed to load items: $e');
      setState(() => _loading = false);
    }

    if (_pendingReload) {
      _currentPage = 0;
      _hasMore = true;
      _pendingReload = false;
      setState(() {});
      await _loadItems();
    }
  }

  void _reload() {
    _reloadDebounce?.cancel();
    _reloadDebounce = Timer(const Duration(milliseconds: 80), () {
      if (_loading) {
        _pendingReload = true;
        return;
      }
      _currentPage = 0;
      _hasMore = true;
      _loadItems();
    });
  }

  void _onScroll() {
    if (!_hasMore || _loading) return;
    final max = _scrollController.position.maxScrollExtent;
    if (_scrollController.offset >= max - 100) {
      _currentPage++;
      _loadItems();
    }
  }

  void _onSearchChanged(String query) {
    _searchQuery = query;
    _selectedIndex = -1;
    _reload();
  }

  void _onTabChanged(ClipboardTab tab) {
    if (_currentTab == tab) return;
    setState(() {
      _currentTab = tab;
      _selectedIndex = -1;
    });
    _reload();
  }

  void _onTypeFilterChanged(List<ClipboardContentType> types) {
    _typeFilters = types;
    _selectedIndex = -1;
    _reload();
  }

  void _onColorFilterChanged(List<CardColor> colors) {
    _colorFilters = colors;
    _selectedIndex = -1;
    _reload();
  }

  void _clearFilters() {
    _typeFilters = [];
    _colorFilters = [];
    _searchController.clear();
    _searchQuery = '';
    _selectedIndex = -1;
    _reload();
  }

  Future<void> _onItemTap(ClipboardItem item) async {
    widget.onPaste(item);
  }

  Future<void> _onItemPin(ClipboardItem item) async {
    await widget.clipboardService.updatePin(item.id, !item.isPinned);
    _reload();
  }

  Future<void> _onItemDelete(ClipboardItem item) async {
    await widget.clipboardService.removeItem(item.id);
    _reload();
  }

  Future<void> _onItemOpen(ClipboardItem item) async {
    bool opened = false;
    try {
      switch (item.type) {
        case ClipboardContentType.image:
          opened = await _openImageInTemp(item);
        case ClipboardContentType.file:
        case ClipboardContentType.folder:
        case ClipboardContentType.audio:
        case ClipboardContentType.video:
          await UrlHelper.open(item.content.split('\n').first.trim());
          opened = true;
        case ClipboardContentType.link:
          await UrlHelper.open(item.content.trim());
          opened = true;
        case ClipboardContentType.email:
          await UrlHelper.open('mailto:${item.content.trim()}');
          opened = true;
        case ClipboardContentType.phone:
          await UrlHelper.open('tel:${item.content.trim()}');
          opened = true;
        default:
          break;
      }
    } catch (_) {}
    if (opened) {
      await widget.clipboardService.recordPaste(item.id);
      _reload();
    }
  }

  Future<bool> _openImageInTemp(ClipboardItem item) async {
    final src = File(item.content);
    if (!src.existsSync()) return false;
    final name = item.content.split(Platform.pathSeparator).last;
    final tmp = await Directory.systemTemp.createTemp('copypaste_');
    final dest = File('${tmp.path}${Platform.pathSeparator}$name');
    await src.copy(dest.path);
    await UrlHelper.open(dest.path);
    return true;
  }

  Future<void> _onItemLabelColor(
    ClipboardItem item,
    String? label,
    CardColor color,
  ) async {
    await widget.clipboardService.updateLabelAndColor(item.id, label, color);
    _reload();
  }

  KeyEventResult _onSearchKeyEvent(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowDown && _items.isNotEmpty) {
      setState(() => _selectedIndex = 0);
      _focusNode.requestFocus();
      _ensureVisible(0);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  KeyEventResult _onKeyEvent(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }

    final key = event.logicalKey;
    final ctrl =
        HardwareKeyboard.instance.isControlPressed ||
        (Platform.isMacOS && HardwareKeyboard.instance.isMetaPressed);
    final alt = HardwareKeyboard.instance.isAltPressed;

    if (key == LogicalKeyboardKey.escape) {
      if (_searchQuery.isNotEmpty ||
          _typeFilters.isNotEmpty ||
          _colorFilters.isNotEmpty) {
        _searchController.clear();
        _onSearchChanged('');
        _clearFilters();
        return KeyEventResult.handled;
      }
      widget.onExit();
      return KeyEventResult.handled;
    }

    if (alt && key == LogicalKeyboardKey.keyC) {
      _searchFocusNode.requestFocus();
      setState(() => _selectedIndex = -1);
      return KeyEventResult.handled;
    }

    if (alt &&
        (key == LogicalKeyboardKey.keyG || key == LogicalKeyboardKey.keyT)) {
      _filterBarKey.currentState?.openMenu();
      return KeyEventResult.handled;
    }

    if (ctrl && key == LogicalKeyboardKey.digit1) {
      _onTabChanged(ClipboardTab.recent);
      return KeyEventResult.handled;
    }

    if (ctrl && key == LogicalKeyboardKey.digit2) {
      _onTabChanged(ClipboardTab.pinned);
      return KeyEventResult.handled;
    }

    if (key == LogicalKeyboardKey.tab &&
        HardwareKeyboard.instance.isShiftPressed) {
      _searchFocusNode.requestFocus();
      setState(() => _selectedIndex = -1);
      return KeyEventResult.handled;
    }

    if (key == LogicalKeyboardKey.arrowDown) {
      if (_selectedIndex < _items.length - 1) {
        setState(() => _selectedIndex++);
        _ensureVisible(_selectedIndex);
      }
      return KeyEventResult.handled;
    }

    if (key == LogicalKeyboardKey.arrowUp) {
      if (_selectedIndex > 0) {
        setState(() => _selectedIndex--);
        _ensureVisible(_selectedIndex);
      } else if (_selectedIndex == 0) {
        setState(() => _selectedIndex = -1);
        _searchFocusNode.requestFocus();
      }
      return KeyEventResult.handled;
    }

    if (key == LogicalKeyboardKey.enter && _selectedIndex >= 0) {
      _onItemTap(_items[_selectedIndex]);
      return KeyEventResult.handled;
    }

    if (key == LogicalKeyboardKey.delete && _selectedIndex >= 0) {
      _onItemDelete(_items[_selectedIndex]);
      return KeyEventResult.handled;
    }

    if (key == LogicalKeyboardKey.keyP && _selectedIndex >= 0) {
      _onItemPin(_items[_selectedIndex]);
      return KeyEventResult.handled;
    }

    if (key == LogicalKeyboardKey.keyE && _selectedIndex >= 0) {
      _editSelectedItem();
      return KeyEventResult.handled;
    }

    if (key == LogicalKeyboardKey.arrowRight && _selectedIndex >= 0) {
      setState(() {
        _expandedIndex = _expandedIndex == _selectedIndex ? -1 : _selectedIndex;
      });
      return KeyEventResult.handled;
    }

    return KeyEventResult.ignored;
  }

  void _editSelectedItem() {
    if (_selectedIndex < 0 || _selectedIndex >= _items.length) return;
    final item = _items[_selectedIndex];
    _showEditDialog(item);
  }

  Future<void> _showEditDialog(ClipboardItem item) async {
    if (!mounted) return;
    final result = await LabelColorDialog.show(
      context,
      currentLabel: item.label,
      currentColor: item.cardColor,
    );
    if (result != null) {
      await _onItemLabelColor(item, result.label, result.color);
    }
  }

  void _ensureVisible(int index) {
    if (index < 0 || index >= _items.length) return;
    final item = _items[index];
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final ctx = _cardKeys[item.id]?.currentContext;
      if (ctx != null) {
        Scrollable.ensureVisible(
          ctx,
          duration: const Duration(milliseconds: 120),
          curve: Curves.easeOut,
        );
      }
    });
  }

  bool get _isEmpty => _items.isEmpty && !_loading;

  @override
  Widget build(BuildContext context) {
    final colors = CopyPasteTheme.colorsOf(context);
    final theme = CopyPasteTheme.of(context);
    final hasColorFilters = _colorFilters.isNotEmpty;

    return Focus(
      focusNode: _focusNode,
      onKeyEvent: _onKeyEvent,
      descendantsAreTraversable: false,
      child: Column(
        children: [
          TitleBar(
            searchController: _searchController,
            searchFocusNode: _searchFocusNode,
            onSearchChanged: _onSearchChanged,
            trailing: FilterBar(
              key: _filterBarKey,
              selectedTypes: _typeFilters,
              selectedColors: _colorFilters,
              colorLabels: widget.colorLabels,
              onTypesChanged: _onTypeFilterChanged,
              onColorsChanged: _onColorFilterChanged,
              onClear: hasColorFilters ? _clearFilters : null,
            ),
          ),
          FilterTabBar(
            selectedTypes: _typeFilters,
            onTypesChanged: _onTypeFilterChanged,
            isPinnedMode: _currentTab == ClipboardTab.pinned,
            onPinnedModeChanged: (pinned) {
              _onTabChanged(pinned ? ClipboardTab.pinned : ClipboardTab.recent);
            },
          ),
          if (widget.showHint) _buildHintBanner(colors),
          Expanded(
            child: _isEmpty
                ? const EmptyState()
                : _buildRealList(theme, _items),
          ),
          Divider(height: 1, thickness: 0.5, color: colors.divider),
          _buildBottomBar(theme, colors),
        ],
      ),
    );
  }

  Widget _buildHintBanner(AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
      color: colors.primary.withValues(alpha: 0.06),
      child: Row(
        children: [
          Icon(
            Icons.lightbulb_outline_rounded,
            size: 14,
            color: colors.primary,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text.rich(
              TextSpan(
                children: [
                  TextSpan(
                    text: l.hintBannerText,
                    style: TextStyle(fontSize: 11, color: colors.onSurface),
                  ),
                  const TextSpan(text: ' '),
                  WidgetSpan(
                    alignment: PlaceholderAlignment.baseline,
                    baseline: TextBaseline.alphabetic,
                    child: GestureDetector(
                      onTap: () {
                        widget.onDismissHint?.call();
                        widget.onSettings();
                      },
                      child: MouseRegion(
                        cursor: SystemMouseCursors.click,
                        child: Text(
                          l.hintBannerAction,
                          style: TextStyle(
                            fontSize: 11,
                            color: colors.primary,
                            decoration: TextDecoration.underline,
                            decorationColor: colors.primary.withValues(
                              alpha: 0.5,
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
          GestureDetector(
            onTap: widget.onDismissHint,
            child: Icon(
              Icons.close_rounded,
              size: 14,
              color: colors.onSurfaceMuted,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRealList(AppThemeData theme, List<ClipboardItem> items) {
    final animate = _isFirstRender;
    if (_isFirstRender) {
      _isFirstRender = false;
    }
    return ListView.builder(
      controller: _scrollController,
      padding: theme.spacing.listPadding.copyWith(top: 6, bottom: 8),
      itemCount: items.length,
      itemBuilder: (context, index) {
        final item = items[index];
        final cardKey = _cardKeys.putIfAbsent(item.id, GlobalKey.new);
        final card = Padding(
          key: cardKey,
          padding: EdgeInsets.only(bottom: theme.spacing.cardGap),
          child: ClipboardCard(
            item: item,
            isSelected: index == _selectedIndex,
            isExpanded: index == _expandedIndex,
            cardMinLines: widget.cardMinLines,
            cardMaxLines: widget.cardMaxLines,
            onTap: () => _onItemTap(item),
            onPin: () => _onItemPin(item),
            onDelete: () => _onItemDelete(item),
            onLabelColor: (label, color) =>
                _onItemLabelColor(item, label, color),
            onPastePlain: () => widget.onPastePlain(item),
            onOpen: () => _onItemOpen(item),
            onRequestThumbnailRefresh:
                widget.clipboardService.requestThumbnailIfStale,
            onSelect: () {
              setState(() => _selectedIndex = index);
              _focusNode.requestFocus();
            },
            onExpandToggle: () {
              setState(() {
                _expandedIndex = _expandedIndex == index ? -1 : index;
              });
            },
          ),
        );

        if (animate && index < _pageSize) {
          return _StaggeredFadeIn(index: index, child: card);
        }
        return card;
      },
    );
  }

  Widget _buildBottomBar(AppThemeData theme, AppThemeColorScheme colors) {
    final l = AppLocalizations.of(context);
    final updateVersion = widget.updateVersion;
    final severity = widget.updateSeverity;
    final isImportant = severity != null && severity != ManifestSeverity.patch;
    final badgeColor = isImportant ? colors.accentRed : colors.primary;
    final badgeText = isImportant
        ? l.updateBadgeImportant(updateVersion ?? '')
        : l.updateBadge(updateVersion ?? '');

    return Container(
      height: theme.spacing.bottomBarHeight,
      padding: const EdgeInsets.symmetric(horizontal: 14),
      child: Row(
        children: [
          if (updateVersion != null)
            Tooltip(
              message: AutoUpdateService.isStoreBuild
                  ? l.updateTooltipStore(updateVersion)
                  : l.updateTooltipGeneric(updateVersion),
              child: InkWell(
                borderRadius: BorderRadius.circular(4),
                onTap: () => _showUpdateDialog(context, updateVersion),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      Icons.system_update_outlined,
                      size: 13,
                      color: badgeColor,
                    ),
                    const SizedBox(width: 5),
                    Text(
                      badgeText,
                      style: theme.typography.branding.copyWith(
                        color: badgeColor,
                        letterSpacing: 0.3,
                      ),
                    ),
                  ],
                ),
              ),
            )
          else
            Opacity(
              opacity: 0.35,
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Image.asset(
                    'assets/icons/icon_notification.png',
                    width: 12,
                    height: 12,
                    color: colors.onSurface,
                    colorBlendMode: BlendMode.srcIn,
                  ),
                  const SizedBox(width: 5),
                  Text(
                    'CopyPaste',
                    style: theme.typography.branding.copyWith(
                      color: colors.onSurface,
                      letterSpacing: 0.3,
                    ),
                  ),
                ],
              ),
            ),
          const Spacer(),
          _BottomBarAction(
            icon: Icons.bug_report_outlined,
            iconSize: 14,
            opacity: 0.4,
            onTap: () =>
                UrlHelper.open('https://github.com/rgdevment/CopyPaste/issues'),
          ),
          const SizedBox(width: 2),
          _BottomBarAction(
            icon: theme.icons.settings,
            iconSize: 14,
            opacity: 0.4,
            onTap: widget.onSettings,
          ),
        ],
      ),
    );
  }

  void _showUpdateDialog(BuildContext context, String version) {
    final l = AppLocalizations.of(context);
    showDialog<void>(
      context: context,
      builder: (dialogCtx) => AlertDialog(
        title: Text(l.updateDialogTitle),
        content: SizedBox(
          width: double.maxFinite,
          child: Text(
            AutoUpdateService.isStoreBuild
                ? l.updateAvailableStore(version)
                : Platform.isMacOS
                ? l.updateAvailableMac(version)
                : Platform.isLinux
                ? l.updateAvailableLinux(version)
                : l.updateAvailableWindows(version),
          ),
        ),
        actionsOverflowButtonSpacing: 8,
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogCtx).pop(),
            child: Text(l.updateDismiss),
          ),
          if (!AutoUpdateService.isStoreBuild)
            FilledButton(
              onPressed: () {
                Navigator.of(dialogCtx).pop();
                UrlHelper.open(
                  'https://github.com/rgdevment/CopyPaste/releases/latest',
                );
              },
              child: Text(l.updateViewRelease),
            ),
        ],
      ),
    );
  }
}

class _StaggeredFadeIn extends StatefulWidget {
  const _StaggeredFadeIn({required this.index, required this.child});

  final int index;
  final Widget child;

  @override
  State<_StaggeredFadeIn> createState() => _StaggeredFadeInState();
}

class _StaggeredFadeInState extends State<_StaggeredFadeIn>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  late final Animation<double> _opacity;
  late final Animation<Offset> _offset;
  Timer? _delayTimer;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 80),
    );
    _opacity = CurvedAnimation(parent: _controller, curve: Curves.easeOut);
    _offset = Tween<Offset>(
      begin: const Offset(0, -4),
      end: Offset.zero,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOut));

    _delayTimer = Timer(Duration(milliseconds: 20 * widget.index), () {
      if (mounted) _controller.forward();
    });
  }

  @override
  void dispose() {
    _delayTimer?.cancel();
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return Transform.translate(
          offset: _offset.value,
          child: Opacity(opacity: _opacity.value, child: child),
        );
      },
      child: widget.child,
    );
  }
}

class _BottomBarAction extends StatefulWidget {
  const _BottomBarAction({
    required this.icon,
    required this.iconSize,
    required this.opacity,
    required this.onTap,
  });

  final IconData icon;
  final double iconSize;
  final double opacity;
  final VoidCallback onTap;

  @override
  State<_BottomBarAction> createState() => _BottomBarActionState();
}

class _BottomBarActionState extends State<_BottomBarAction> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final colors = CopyPasteTheme.colorsOf(context);

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
          child: Icon(
            widget.icon,
            size: widget.iconSize,
            color: colors.onSurface.withValues(
              alpha: _hovering ? widget.opacity + 0.25 : widget.opacity,
            ),
          ),
        ),
      ),
    );
  }
}
