import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../l10n/app_localizations.dart';
import '../theme/theme_provider.dart';

class CopyPasteSearchBox extends StatefulWidget {
  const CopyPasteSearchBox({
    required this.controller,
    required this.focusNode,
    required this.onChanged,
    required this.onDownArrow,
    this.trailing,
    super.key,
  });

  final TextEditingController controller;
  final FocusNode focusNode;
  final void Function(String query) onChanged;
  final VoidCallback onDownArrow;
  final Widget? trailing;

  @override
  State<CopyPasteSearchBox> createState() => _CopyPasteSearchBoxState();
}

class _CopyPasteSearchBoxState extends State<CopyPasteSearchBox> {
  Timer? _debounce;

  @override
  void dispose() {
    _debounce?.cancel();
    super.dispose();
  }

  void _onTextChanged(String text) {
    _debounce?.cancel();
    final theme = CopyPasteTheme.of(context);
    _debounce = Timer(theme.searchStyle.debounceDuration, () {
      widget.onChanged(text);
    });
  }

  KeyEventResult _onKeyEvent(FocusNode node, KeyEvent event) {
    if (event is KeyDownEvent &&
        event.logicalKey == LogicalKeyboardKey.arrowDown) {
      widget.onDownArrow();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);

    return SizedBox(
      height: theme.sizing.searchBoxHeight,
      child: Focus(
        onKeyEvent: _onKeyEvent,
        child: TextField(
          controller: widget.controller,
          focusNode: widget.focusNode,
          onChanged: _onTextChanged,
          style: theme.typography.searchInput.copyWith(
            color: colors.onSurface,
          ),
          decoration: InputDecoration(
            hintText: AppLocalizations.of(context).searchPlaceholder,
            hintStyle: theme.typography.searchInput.copyWith(
              color: colors.onSurfaceMuted,
            ),
            prefixIcon: Padding(
              padding: const EdgeInsets.only(left: 8, right: 4),
              child: Icon(
                theme.icons.search,
                size: theme.sizing.iconSizeMd,
                color: colors.onSurface.withValues(
                  alpha: theme.searchStyle.iconOpacity,
                ),
              ),
            ),
            prefixIconConstraints: const BoxConstraints(
              minWidth: 28,
              minHeight: 0,
            ),
            suffixIcon: Row(
              mainAxisSize: MainAxisSize.min,
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                if (widget.controller.text.isNotEmpty)
                  GestureDetector(
                    onTap: () {
                      widget.controller.clear();
                      widget.onChanged('');
                      setState(() {});
                    },
                    child: Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 4),
                      child: Icon(
                        theme.icons.close,
                        size: 12,
                        color: colors.onSurfaceMuted,
                      ),
                    ),
                  ),
                if (widget.trailing != null) widget.trailing!,
              ],
            ),
            suffixIconConstraints: const BoxConstraints(
              minWidth: 0,
              minHeight: 0,
            ),
            contentPadding: theme.searchStyle.padding,
            filled: true,
            fillColor: colors.searchBackground,
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(theme.radii.searchBox),
              borderSide: BorderSide.none,
            ),
            enabledBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(theme.radii.searchBox),
              borderSide: BorderSide.none,
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(theme.radii.searchBox),
              borderSide: BorderSide(
                color: colors.primary.withValues(alpha: 0.5),
                width: 1.5,
              ),
            ),
            isDense: true,
          ),
        ),
      ),
    );
  }
}
