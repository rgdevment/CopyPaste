import 'package:core/core.dart';
import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../theme/theme_provider.dart';

class FilterTabBar extends StatefulWidget {
  const FilterTabBar({
    required this.selectedTypes,
    required this.onTypesChanged,
    required this.isPinnedMode,
    required this.onPinnedModeChanged,
    super.key,
  });

  final List<ClipboardContentType> selectedTypes;
  final void Function(List<ClipboardContentType>) onTypesChanged;
  final bool isPinnedMode;
  final void Function(bool) onPinnedModeChanged;

  @override
  State<FilterTabBar> createState() => _FilterTabBarState();
}

class _FilterTabBarState extends State<FilterTabBar> {
  final ScrollController _scrollController = ScrollController();
  bool _isDragging = false;
  double _dragStartX = 0;
  double _scrollStartOffset = 0;

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final l = AppLocalizations.of(context);

    final isAllSelected = widget.selectedTypes.isEmpty && !widget.isPinnedMode;

    final tabs = <_TabDef>[
      _TabDef(label: l.filterAll, type: null, isPinned: false),
      _TabDef(label: l.filterPinned, type: null, isPinned: true),
      _TabDef(
        label: l.typeText,
        type: ClipboardContentType.text,
        isPinned: false,
      ),
      _TabDef(
        label: l.typeImage,
        type: ClipboardContentType.image,
        isPinned: false,
      ),
      _TabDef(
        label: l.typeFile,
        type: ClipboardContentType.file,
        isPinned: false,
      ),
      _TabDef(
        label: l.typeFolder,
        type: ClipboardContentType.folder,
        isPinned: false,
      ),
      _TabDef(
        label: l.typeLink,
        type: ClipboardContentType.link,
        isPinned: false,
      ),
      _TabDef(
        label: l.typeAudio,
        type: ClipboardContentType.audio,
        isPinned: false,
      ),
      _TabDef(
        label: l.typeVideo,
        type: ClipboardContentType.video,
        isPinned: false,
      ),
    ];

    return SizedBox(
      height: theme.spacing.filterTabBarHeight,
      child: ShaderMask(
        shaderCallback: (bounds) => const LinearGradient(
          begin: Alignment.centerLeft,
          end: Alignment.centerRight,
          colors: [Colors.white, Colors.white, Colors.transparent],
          stops: [0.0, 0.92, 1.0],
        ).createShader(bounds),
        blendMode: BlendMode.dstIn,
        child: Padding(
          padding: theme.spacing.filterTabBarPadding.copyWith(right: 0),
          child: MouseRegion(
            cursor: _isDragging
                ? SystemMouseCursors.grabbing
                : SystemMouseCursors.grab,
            child: Listener(
              onPointerDown: (e) {
                setState(() {
                  _isDragging = true;
                  _dragStartX = e.position.dx;
                  _scrollStartOffset = _scrollController.offset;
                });
              },
              onPointerMove: (e) {
                if (!_isDragging) return;
                final delta = _dragStartX - e.position.dx;
                _scrollController.jumpTo(
                  (_scrollStartOffset + delta).clamp(
                    0.0,
                    _scrollController.position.maxScrollExtent,
                  ),
                );
              },
              onPointerUp: (_) => setState(() => _isDragging = false),
              onPointerCancel: (_) => setState(() => _isDragging = false),
              child: ListView.separated(
                controller: _scrollController,
                scrollDirection: Axis.horizontal,
                physics: const BouncingScrollPhysics(),
                padding: const EdgeInsets.only(right: 24),
                itemCount: tabs.length,
                separatorBuilder: (context, i) => const SizedBox(width: 4),
                itemBuilder: (context, index) {
                  final tab = tabs[index];
                  final isActive = tab.isPinned
                      ? widget.isPinnedMode
                      : tab.type == null
                      ? isAllSelected
                      : !widget.isPinnedMode &&
                            widget.selectedTypes.length == 1 &&
                            widget.selectedTypes.contains(tab.type);

                  return _FilterTab(
                    label: tab.label,
                    isActive: isActive,
                    onTap: () {
                      if (tab.isPinned) {
                        widget.onPinnedModeChanged(!widget.isPinnedMode);
                        if (!widget.isPinnedMode) widget.onTypesChanged([]);
                      } else if (tab.type == null) {
                        widget.onPinnedModeChanged(false);
                        widget.onTypesChanged([]);
                      } else {
                        widget.onPinnedModeChanged(false);
                        final type = tab.type!;
                        final wasActive =
                            !widget.isPinnedMode &&
                            widget.selectedTypes.length == 1 &&
                            widget.selectedTypes.contains(type);
                        widget.onTypesChanged(wasActive ? [] : [type]);
                      }
                    },
                  );
                },
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _TabDef {
  const _TabDef({
    required this.label,
    required this.type,
    required this.isPinned,
  });
  final String label;
  final ClipboardContentType? type;
  final bool isPinned;
}

class _FilterTab extends StatefulWidget {
  const _FilterTab({
    required this.label,
    required this.isActive,
    required this.onTap,
  });

  final String label;
  final bool isActive;
  final VoidCallback onTap;

  @override
  State<_FilterTab> createState() => _FilterTabState();
}

class _FilterTabState extends State<_FilterTab> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);

    final Color bg;
    final Color borderColor;
    final Color textColor;
    final FontWeight weight;

    if (widget.isActive) {
      bg = colors.primary.withValues(alpha: 0.13);
      borderColor = colors.primary.withValues(alpha: 0.4);
      textColor = colors.accentPurple;
      weight = FontWeight.w600;
    } else {
      bg = _hovering ? colors.cardBackground : colors.searchBackground;
      borderColor = _hovering
          ? colors.onSurface.withValues(alpha: 0.18)
          : colors.onSurface.withValues(alpha: 0.12);
      textColor = colors.onSurface.withValues(alpha: 0.5);
      weight = FontWeight.w500;
    }

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          padding: const EdgeInsets.symmetric(horizontal: 11, vertical: 4),
          decoration: BoxDecoration(
            color: bg,
            borderRadius: BorderRadius.circular(theme.radii.chip),
            border: Border.all(color: borderColor),
          ),
          child: Center(
            child: Text(
              widget.label,
              style: theme.typography.filterTabChip.copyWith(
                color: textColor,
                fontWeight: weight,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
