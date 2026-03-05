import 'package:core/core.dart';
import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../theme/app_theme_data.dart';
import '../theme/theme_provider.dart';

class FilterBar extends StatefulWidget {
  const FilterBar({
    required this.selectedTypes,
    required this.selectedColors,
    required this.onTypesChanged,
    required this.onColorsChanged,
    this.colorLabels = const {},
    this.onClear,
    super.key,
  });

  final List<ClipboardContentType> selectedTypes;
  final List<CardColor> selectedColors;
  final void Function(List<ClipboardContentType>) onTypesChanged;
  final void Function(List<CardColor>) onColorsChanged;
  final Map<String, String> colorLabels;
  final VoidCallback? onClear;

  @override
  State<FilterBar> createState() => FilterBarState();
}

class FilterBarState extends State<FilterBar> {
  void openMenu() => _showFilterMenu(context);

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final hasFilters = widget.selectedColors.isNotEmpty;

    return _FilterButton(
      icon: theme.icons.filter,
      isActive: hasFilters,
      badge: hasFilters ? widget.selectedColors.length : 0,
      onTap: () => _showFilterMenu(context),
    );
  }

  void _showFilterMenu(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);
    final renderBox = context.findRenderObject()! as RenderBox;
    final offset = renderBox.localToGlobal(Offset.zero);

    showMenu<void>(
      context: context,
      position: RelativeRect.fromLTRB(
        offset.dx,
        offset.dy + renderBox.size.height + 4,
        offset.dx + renderBox.size.width,
        0,
      ),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(theme.radii.md),
      ),
      color: colors.cardBackground,
      items: [
        if (widget.selectedColors.isNotEmpty)
          PopupMenuItem<void>(
            height: 32,
            onTap: widget.onClear,
            child: Row(
              children: [
                Icon(
                  theme.icons.clear,
                  size: theme.sizing.iconSizeSm,
                  color: colors.danger,
                ),
                const SizedBox(width: 8),
                Text(
                  'Clear all filters',
                  style: theme.typography.filterChip.copyWith(
                    color: colors.danger,
                  ),
                ),
              ],
            ),
          ),
        if (widget.selectedColors.isNotEmpty)
          const PopupMenuDivider(height: 8),
        PopupMenuItem<void>(
          enabled: false,
          height: 28,
          child: Text(
            'COLOR',
            style: theme.typography.filterChip.copyWith(
              color: colors.onSurfaceMuted,
              fontSize: 10,
              letterSpacing: 0.8,
            ),
          ),
        ),
        ..._getColorEntries(context).map(
          (entry) =>
              _buildColorItem(context, entry.$1, entry.$2, theme, colors),
        ),
      ],
    );
  }

  List<(CardColor, String)> _getColorEntries(BuildContext context) {
    final l = AppLocalizations.of(context);
    final cl = widget.colorLabels;
    return [
      (CardColor.red, cl['Red'] ?? l.colorRed),
      (CardColor.green, cl['Green'] ?? l.colorGreen),
      (CardColor.purple, cl['Purple'] ?? l.colorPurple),
      (CardColor.yellow, cl['Yellow'] ?? l.colorYellow),
      (CardColor.blue, cl['Blue'] ?? l.colorBlue),
      (CardColor.orange, cl['Orange'] ?? l.colorOrange),
    ];
  }

  PopupMenuItem<void> _buildColorItem(
    BuildContext context,
    CardColor cardColor,
    String label,
    AppThemeData theme,
    AppThemeColorScheme colors,
  ) {
    final isSelected = widget.selectedColors.contains(cardColor);
    final dotColor = colors.accentForIndex(cardColor.value);

    return PopupMenuItem<void>(
      height: 32,
      onTap: () {
        final updated = List<CardColor>.from(widget.selectedColors);
        if (isSelected) {
          updated.remove(cardColor);
        } else {
          updated.add(cardColor);
        }
        widget.onColorsChanged(updated);
      },
      child: Row(
        children: [
          Container(
            width: theme.sizing.colorDotSize,
            height: theme.sizing.colorDotSize,
            decoration: BoxDecoration(
              color: dotColor,
              shape: BoxShape.circle,
              border: Border.all(
                color: dotColor.withValues(alpha: 0.5),
              ),
            ),
          ),
          const SizedBox(width: 8),
          Text(
            label,
            style: theme.typography.filterChip.copyWith(
              color:
                  isSelected ? colors.onSurface : colors.onSurfaceVariant,
            ),
          ),
          const Spacer(),
          if (isSelected)
            Icon(
              Icons.check,
              size: 14,
              color: colors.primary,
            ),
        ],
      ),
    );
  }
}

class _FilterButton extends StatefulWidget {
  const _FilterButton({
    required this.icon,
    required this.isActive,
    required this.badge,
    required this.onTap,
  });

  final IconData icon;
  final bool isActive;
  final int badge;
  final VoidCallback onTap;

  @override
  State<_FilterButton> createState() => _FilterButtonState();
}

class _FilterButtonState extends State<_FilterButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          width: 30,
          height: 30,
          margin: const EdgeInsets.only(right: 3),
          decoration: BoxDecoration(
            color: _hovering
                ? colors.onSurface.withValues(alpha: 0.08)
                : widget.isActive
                    ? colors.primary.withValues(alpha: 0.1)
                    : Colors.transparent,
            borderRadius: BorderRadius.circular(6),
          ),
          child: Stack(
            alignment: Alignment.center,
            children: [
              Icon(
                widget.icon,
                size: theme.sizing.iconSizeMd,
                color: widget.isActive
                    ? colors.primary
                    : colors.onSurface.withValues(alpha: 0.5),
              ),
              if (widget.badge > 0)
                Positioned(
                  top: 2,
                  right: 2,
                  child: Container(
                    width: 14,
                    height: 14,
                    decoration: BoxDecoration(
                      color: colors.primary,
                      shape: BoxShape.circle,
                    ),
                    child: Center(
                      child: Text(
                        '${widget.badge}',
                        style: TextStyle(
                          color: colors.onPrimary,
                          fontSize: 8,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
