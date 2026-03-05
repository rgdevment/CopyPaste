import 'package:core/core.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../l10n/app_localizations.dart';

import '../theme/app_theme_data.dart';
import '../theme/theme_provider.dart';

class LabelColorResult {
  const LabelColorResult({required this.label, required this.color});
  final String? label;
  final CardColor color;
}

class LabelColorDialog extends StatefulWidget {
  const LabelColorDialog({
    required this.currentLabel,
    required this.currentColor,
    super.key,
  });

  final String? currentLabel;
  final CardColor currentColor;

  static Future<LabelColorResult?> show(
    BuildContext context, {
    String? currentLabel,
    CardColor currentColor = CardColor.none,
  }) {
    final theme = CopyPasteTheme.of(context);
    return showDialog<LabelColorResult>(
      context: context,
      barrierColor: Colors.black26,
      builder: (_) => CopyPasteTheme(
        themeData: theme,
        child: Theme(
          data: Theme.of(context),
          child: LabelColorDialog(
            currentLabel: currentLabel,
            currentColor: currentColor,
          ),
        ),
      ),
    );
  }

  @override
  State<LabelColorDialog> createState() => _LabelColorDialogState();
}

class _LabelColorDialogState extends State<LabelColorDialog> {
  late final TextEditingController _labelController;
  late CardColor _selectedColor;

  @override
  void initState() {
    super.initState();
    _labelController = TextEditingController(text: widget.currentLabel ?? '');
    _selectedColor = widget.currentColor;
  }

  @override
  void dispose() {
    _labelController.dispose();
    super.dispose();
  }

  void _submit() {
    final label = _labelController.text.trim();
    Navigator.of(context).pop(
      LabelColorResult(
        label: label.isEmpty ? null : label,
        color: _selectedColor,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);
    final l = AppLocalizations.of(context);

    return Dialog(
      backgroundColor: colors.cardBackground,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(theme.radii.lg),
      ),
      elevation: 8,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 280),
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                l.editDialogTitle,
                style: TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: colors.onSurface,
                ),
              ),
              const SizedBox(height: 16),
              SizedBox(
                height: 36,
                child: TextField(
                  controller: _labelController,
                  autofocus: true,
                  maxLength: ClipboardItem.maxLabelLength,
                  maxLengthEnforcement: MaxLengthEnforcement.enforced,
                  style: theme.typography.searchInput.copyWith(
                    color: colors.onSurface,
                  ),
                  decoration: InputDecoration(
                    hintText: l.editDialogHint,
                    hintStyle: theme.typography.searchInput.copyWith(
                      color: colors.onSurfaceMuted,
                    ),
                    counterText: '',
                    contentPadding: const EdgeInsets.symmetric(
                      horizontal: 10,
                      vertical: 8,
                    ),
                    filled: true,
                    fillColor: colors.searchBackground,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(theme.radii.sm),
                      borderSide: BorderSide.none,
                    ),
                    focusedBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(theme.radii.sm),
                      borderSide: BorderSide(
                        color: colors.primary.withValues(alpha: 0.5),
                        width: 1.5,
                      ),
                    ),
                    isDense: true,
                  ),
                  onSubmitted: (_) => _submit(),
                ),
              ),
              const SizedBox(height: 16),
              Text(
                l.editColorLabel,
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.w500,
                  color: colors.onSurfaceMuted,
                  letterSpacing: 0.5,
                ),
              ),
              const SizedBox(height: 10),
              _buildColorGrid(colors, theme),
              const SizedBox(height: 20),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  _DialogButton(
                    label: l.buttonCancel,
                    onTap: () => Navigator.of(context).pop(),
                    colors: colors,
                    theme: theme,
                  ),
                  const SizedBox(width: 8),
                  _DialogButton(
                    label: l.buttonSave,
                    onTap: _submit,
                    colors: colors,
                    theme: theme,
                    isPrimary: true,
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildColorGrid(AppThemeColorScheme colors, AppThemeData theme) {
    final l = AppLocalizations.of(context);
    final entries = [
      (CardColor.none, l.colorNone),
      (CardColor.red, l.colorRed),
      (CardColor.green, l.colorGreen),
      (CardColor.purple, l.colorPurple),
      (CardColor.yellow, l.colorYellow),
      (CardColor.blue, l.colorBlue),
      (CardColor.orange, l.colorOrange),
    ];

    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: entries.map((e) {
        final isSelected = _selectedColor == e.$1;
        final dotColor = e.$1 == CardColor.none
            ? colors.onSurfaceSubtle
            : colors.accentForIndex(e.$1.value);

        return GestureDetector(
          onTap: () => setState(() => _selectedColor = e.$1),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 120),
            width: 28,
            height: 28,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              border: Border.all(
                color: isSelected
                    ? colors.primary
                    : Colors.transparent,
                width: 2,
              ),
            ),
            child: Center(
              child: Container(
                width: 18,
                height: 18,
                decoration: BoxDecoration(
                  color: e.$1 == CardColor.none
                      ? Colors.transparent
                      : dotColor,
                  shape: BoxShape.circle,
                  border: e.$1 == CardColor.none
                      ? Border.all(color: colors.onSurfaceSubtle, width: 1.5)
                      : null,
                ),
                child: e.$1 == CardColor.none
                    ? Center(
                        child: Icon(
                          Icons.close_rounded,
                          size: 10,
                          color: colors.onSurfaceSubtle,
                        ),
                      )
                    : null,
              ),
            ),
          ),
        );
      }).toList(),
    );
  }
}

class _DialogButton extends StatefulWidget {
  const _DialogButton({
    required this.label,
    required this.onTap,
    required this.colors,
    required this.theme,
    this.isPrimary = false,
  });

  final String label;
  final VoidCallback onTap;
  final AppThemeColorScheme colors;
  final AppThemeData theme;
  final bool isPrimary;

  @override
  State<_DialogButton> createState() => _DialogButtonState();
}

class _DialogButtonState extends State<_DialogButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 100),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 7),
          decoration: BoxDecoration(
            color: widget.isPrimary
                ? (_hovering
                    ? widget.colors.primary.withValues(alpha: 0.9)
                    : widget.colors.primary)
                : (_hovering
                    ? widget.colors.onSurface.withValues(alpha: 0.08)
                    : Colors.transparent),
            borderRadius: BorderRadius.circular(widget.theme.radii.button),
          ),
          child: Text(
            widget.label,
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w500,
              color: widget.isPrimary
                  ? widget.colors.onPrimary
                  : widget.colors.onSurface,
            ),
          ),
        ),
      ),
    );
  }
}
