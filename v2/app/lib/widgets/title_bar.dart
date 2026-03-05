import 'dart:async';

import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import '../l10n/app_localizations.dart';
import '../theme/theme_provider.dart';

class TitleBar extends StatelessWidget {
  const TitleBar({
    required this.searchController,
    required this.searchFocusNode,
    required this.onSearchChanged,
    required this.trailing,
    super.key,
  });

  final TextEditingController searchController;
  final FocusNode searchFocusNode;
  final void Function(String) onSearchChanged;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);

    return DragToMoveArea(
      child: Padding(
        padding: theme.spacing.searchBarPadding,
        child: _SearchBar(
          controller: searchController,
          focusNode: searchFocusNode,
          onChanged: onSearchChanged,
          trailing: trailing,
        ),
      ),
    );
  }
}

class _SearchBar extends StatefulWidget {
  const _SearchBar({
    required this.controller,
    required this.focusNode,
    required this.onChanged,
    this.trailing,
  });

  final TextEditingController controller;
  final FocusNode focusNode;
  final void Function(String) onChanged;
  final Widget? trailing;

  @override
  State<_SearchBar> createState() => _SearchBarState();
}

class _SearchBarState extends State<_SearchBar> {
  Timer? _debounce;
  bool _focused = false;

  @override
  void initState() {
    super.initState();
    widget.focusNode.addListener(_onFocusChange);
  }

  @override
  void dispose() {
    _debounce?.cancel();
    widget.focusNode.removeListener(_onFocusChange);
    super.dispose();
  }

  void _onFocusChange() {
    setState(() => _focused = widget.focusNode.hasFocus);
  }

  void _onChanged(String value) {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 300), () {
      widget.onChanged(value);
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);

    return AnimatedContainer(
        duration: const Duration(milliseconds: 150),
        height: theme.sizing.searchBoxHeight,
        decoration: BoxDecoration(
          color: _focused ? colors.cardBackground : colors.searchBackground,
          borderRadius: BorderRadius.circular(theme.radii.searchBox),
          border: Border.all(
            color: _focused
                ? colors.primary.withValues(alpha: 0.5)
                : colors.searchBorder,
          ),
          boxShadow: [
            if (_focused)
              BoxShadow(
                color: colors.primary.withValues(alpha: 0.1),
                blurRadius: 8,
                spreadRadius: 2,
              )
            else
              BoxShadow(
                color: colors.onSurface.withValues(alpha: 0.06),
                blurRadius: 3,
                offset: const Offset(0, 1),
              ),
          ],
        ),
        child: TextField(
          controller: widget.controller,
          focusNode: widget.focusNode,
          onChanged: _onChanged,
          textAlignVertical: TextAlignVertical.center,
          style: theme.typography.searchInput.copyWith(
            color: colors.onSurface.withValues(alpha: 0.8),
          ),
          cursorColor: colors.primary,
          cursorWidth: 1.2,
          decoration: InputDecoration(
            hintText: AppLocalizations.of(context).searchPlaceholder,
            hintStyle: theme.typography.searchInput.copyWith(
              color: colors.onSurface.withValues(alpha: 0.35),
            ),
            prefixIcon: Padding(
              padding: const EdgeInsets.only(left: 14, right: 8),
              child: Icon(
                theme.icons.search,
                size: 14,
                color: colors.onSurface.withValues(
                  alpha: theme.searchStyle.iconOpacity,
                ),
              ),
            ),
            prefixIconConstraints: const BoxConstraints(
              minWidth: 0,
              minHeight: 0,
            ),
            suffixIcon: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (widget.controller.text.isNotEmpty)
                  GestureDetector(
                    onTap: () {
                      widget.controller.clear();
                      widget.onChanged('');
                    },
                    child: Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 4),
                      child: Icon(
                        Icons.close_rounded,
                        size: 12,
                        color: colors.onSurfaceMuted,
                      ),
                    ),
                  ),
                if (widget.trailing != null)
                  Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: widget.trailing!,
                  ),
              ],
            ),
            suffixIconConstraints: const BoxConstraints(
              minWidth: 0,
              minHeight: 0,
            ),
            contentPadding: EdgeInsets.symmetric(
              horizontal: theme.searchStyle.padding.left,
              vertical: 0,
            ),
            border: InputBorder.none,
            enabledBorder: InputBorder.none,
            focusedBorder: InputBorder.none,
            isDense: true,
          ),
        ),
    );
  }
}
