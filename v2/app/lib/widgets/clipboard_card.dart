import 'dart:convert';
import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import '../l10n/app_localizations.dart';
import '../theme/app_theme_data.dart';
import '../theme/theme_provider.dart';
import 'label_color_dialog.dart';

class ClipboardCard extends StatefulWidget {
  const ClipboardCard({
    required this.item,
    required this.onTap,
    required this.onPin,
    required this.onDelete,
    required this.onLabelColor,
    this.onPastePlain,
    this.onExpandToggle,
    this.onSelect,
    this.isSelected = false,
    this.isExpanded = false,
    this.cardMinLines,
    this.cardMaxLines,
    super.key,
  });

  final ClipboardItem item;
  final VoidCallback onTap;
  final VoidCallback onPin;
  final VoidCallback onDelete;
  final void Function(String? label, CardColor color) onLabelColor;
  final VoidCallback? onPastePlain;
  final VoidCallback? onExpandToggle;
  final VoidCallback? onSelect;
  final bool isSelected;
  final bool isExpanded;
  final int? cardMinLines;
  final int? cardMaxLines;

  @override
  State<ClipboardCard> createState() => _ClipboardCardState();
}

class _ClipboardCardState extends State<ClipboardCard> {
  bool _hovering = false;
  String? _resolvedImagePath;
  bool _imagePathResolved = false;
  DateTime? _lastPrimaryDown;

  static const _doubleTapTimeout = Duration(milliseconds: 300);

  void _handlePointerDown(PointerDownEvent event) {
    if (event.buttons != kPrimaryButton) return;
    widget.onSelect?.call();
    final now = DateTime.now();
    if (_lastPrimaryDown != null &&
        now.difference(_lastPrimaryDown!) < _doubleTapTimeout) {
      _lastPrimaryDown = null;
      widget.onTap();
    } else {
      _lastPrimaryDown = now;
    }
  }

  @override
  void initState() {
    super.initState();
    _resolveImagePath();
  }

  @override
  void didUpdateWidget(ClipboardCard oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.item.id != widget.item.id ||
        oldWidget.item.content != widget.item.content ||
        oldWidget.item.metadata != widget.item.metadata) {
      _imagePathResolved = false;
      _resolveImagePath();
    }
  }

  bool _needsExpandToggle(ClipboardItem item) {
    if (widget.isExpanded) return true;
    final type = item.type;
    if (type == ClipboardContentType.text ||
        type == ClipboardContentType.unknown) {
      return item.content.contains('\n') || item.content.length > 80;
    }
    return false;
  }

  void _resolveImagePath() {
    final item = widget.item;
    if (item.type != ClipboardContentType.image &&
        item.type != ClipboardContentType.video &&
        item.type != ClipboardContentType.audio) {
      return;
    }
    String? path;
    if (item.type == ClipboardContentType.image) {
      path = item.content;
    }
    if (path != null) {
      _checkFileAsync(path);
    } else {
      _resolvedImagePath = null;
      _imagePathResolved = true;
      if (mounted) setState(() {});
    }
  }

  Future<void> _checkFileAsync(String path) async {
    if (path.isEmpty) {
      _resolvedImagePath = null;
      _imagePathResolved = true;
      if (mounted) setState(() {});
      return;
    }
    final exists = await File(path).exists();
    _resolvedImagePath = exists ? path : null;
    _imagePathResolved = true;
    if (mounted) setState(() {});
  }

  Future<void> _editLabelColor(BuildContext context) async {
    final result = await LabelColorDialog.show(
      context,
      currentLabel: widget.item.label,
      currentColor: widget.item.cardColor,
    );
    if (result != null && mounted) {
      widget.onLabelColor(result.label, result.color);
    }
  }

  bool get _isPlainPasteable =>
      widget.item.type == ClipboardContentType.text ||
      widget.item.type == ClipboardContentType.link;

  Future<void> _showContextMenu(BuildContext ctx, Offset position) async {
    final size = MediaQuery.of(ctx).size;
    final item = widget.item;
    final colors = CopyPasteTheme.colorsOf(ctx);
    final isDark = Theme.of(ctx).brightness == Brightness.dark;
    final l = AppLocalizations.of(ctx);
    final action = await showMenu<_ContextAction>(
      context: ctx,
      position: RelativeRect.fromLTRB(
        position.dx,
        position.dy,
        size.width - position.dx,
        size.height - position.dy,
      ),
      elevation: 8,
      color: isDark ? colors.surfaceVariant : colors.cardBackground,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      items: [
        PopupMenuItem(
          value: _ContextAction.paste,
          height: 32,
          child: _ContextMenuItem(
            icon: Icons.content_paste_rounded,
            label: l.menuPaste,
            colors: colors,
          ),
        ),
        if (_isPlainPasteable && widget.onPastePlain != null)
          PopupMenuItem(
            value: _ContextAction.pastePlain,
            height: 32,
            child: _ContextMenuItem(
              icon: Icons.format_clear_rounded,
              label: l.menuPastePlain,
              colors: colors,
            ),
          ),
        const PopupMenuDivider(height: 1),
        PopupMenuItem(
          value: _ContextAction.pin,
          height: 32,
          child: _ContextMenuItem(
            icon: item.isPinned
                ? Icons.push_pin_rounded
                : Icons.push_pin_outlined,
            label: item.isPinned ? l.menuUnpin : l.menuPin,
            colors: colors,
          ),
        ),
        PopupMenuItem(
          value: _ContextAction.edit,
          height: 32,
          child: _ContextMenuItem(
            icon: Icons.edit_rounded,
            label: l.menuEdit,
            colors: colors,
          ),
        ),
        const PopupMenuDivider(height: 1),
        PopupMenuItem(
          value: _ContextAction.delete,
          height: 32,
          child: _ContextMenuItem(
            icon: Icons.delete_rounded,
            label: l.menuDelete,
            colors: colors,
            danger: true,
          ),
        ),
      ],
    );
    if (!mounted) return;
    switch (action) {
      case _ContextAction.paste:
        widget.onTap();
      case _ContextAction.pastePlain:
        widget.onPastePlain?.call();
      case _ContextAction.pin:
        widget.onPin();
      case _ContextAction.edit:
        await _editLabelColor(context);
      case _ContextAction.delete:
        widget.onDelete();
      case null:
        break;
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final item = widget.item;
    final accentColor = colors.accentForIndex(item.cardColor.value);
    final hasColor = item.cardColor != CardColor.none;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: Listener(
        onPointerDown: _handlePointerDown,
        child: GestureDetector(
          onSecondaryTapUp: (d) => _showContextMenu(context, d.globalPosition),
          child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          curve: Curves.easeOut,
          constraints: BoxConstraints(minHeight: theme.sizing.cardMinHeight),
          transform: _hovering ? Matrix4.translationValues(0, -1, 0) : null,
          decoration: BoxDecoration(
            color: _hovering && isDark
                ? colors.surfaceVariant
                : colors.cardBackground,
            borderRadius: BorderRadius.circular(theme.radii.card),
            border: Border.all(
              color: widget.isSelected
                  ? colors.primary.withValues(alpha: 0.5)
                  : _hovering
                  ? colors.onSurface.withValues(alpha: isDark ? 0.1 : 0.18)
                  : colors.cardBorder,
              width: theme.cardStyle.borderWidth,
            ),
            boxShadow: [
              if (widget.isSelected)
                BoxShadow(
                  color: colors.primary.withValues(alpha: 0.2),
                  blurRadius: 8,
                  spreadRadius: 1,
                ),
              if (isDark)
                BoxShadow(
                  color: Colors.black.withValues(alpha: _hovering ? 0.3 : 0.2),
                  blurRadius: _hovering ? 12 : 6,
                  offset: Offset(0, _hovering ? 3 : 1),
                )
              else
                BoxShadow(
                  color: Colors.black.withValues(alpha: _hovering ? 0.1 : 0.07),
                  blurRadius: _hovering ? 10 : 4,
                  offset: Offset(0, _hovering ? 3 : 1),
                ),
            ],
          ),
          child: Stack(
            children: [
              if (hasColor)
                Positioned(
                  left: 0,
                  top: 0,
                  bottom: 0,
                  child: Container(
                    width: theme.sizing.colorIndicatorWidth,
                    decoration: BoxDecoration(
                      color: accentColor,
                      borderRadius: theme.cardStyle.colorIndicatorBorderRadius,
                    ),
                  ),
                ),
              Padding(
                padding: theme.spacing.cardPadding.copyWith(
                  left: hasColor
                      ? theme.spacing.cardPadding.left +
                            theme.sizing.colorIndicatorWidth +
                            2
                      : theme.spacing.cardPadding.left,
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    _buildHeader(theme, colors, item),
                    const SizedBox(height: 4),
                    _buildContent(theme, colors, item),
                    if (_hasFooter(item)) ...[
                      const SizedBox(height: 6),
                      _buildFooter(theme, colors, item),
                    ],
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
      ),
    );
  }

  Widget _buildHeader(
    AppThemeData theme,
    AppThemeColorScheme colors,
    ClipboardItem item,
  ) {
    final l = AppLocalizations.of(context);
    final typeColor = _typeColor(item.type, colors);
    final iconSize = theme.sizing.cardTypeIconContainerSize;

    final isDark = Theme.of(context).brightness == Brightness.dark;
    final iconBgAlpha = isDark ? 0.2 : 0.13;

    return Stack(
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Container(
              width: iconSize,
              height: iconSize,
              decoration: BoxDecoration(
                color: typeColor.withValues(alpha: iconBgAlpha),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Center(
                child: Icon(
                  theme.icons.forContentType(item.type.value),
                  size: 16,
                  color: typeColor,
                ),
              ),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.only(right: 40),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      item.label ?? _contentTypeName(item.type, l),
                      style: theme.typography.cardLabel.copyWith(
                        color: item.label != null
                            ? typeColor.withValues(alpha: 0.85)
                            : colors.onSurface.withValues(
                                alpha: theme.cardStyle.headerOpacity,
                              ),
                        letterSpacing: 0.06,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    if (item.appSource != null) ...[
                      const SizedBox(height: 1),
                      Text(
                        item.appSource!,
                        style: theme.typography.cardFooter.copyWith(
                          color: colors.onSurface.withValues(
                            alpha: theme.cardStyle.appSourceOpacity,
                          ),
                          fontSize: 10,
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
        Positioned(
          right: 0,
          top: 0,
          bottom: 0,
          child: Align(
            alignment: Alignment.centerRight,
            child: Stack(
              alignment: Alignment.centerRight,
              children: [
                AnimatedOpacity(
                  opacity: _hovering ? 0.0 : 1.0,
                  duration: const Duration(milliseconds: 120),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (item.isPinned)
                        Padding(
                          padding: const EdgeInsets.only(right: 4),
                          child: Icon(
                            theme.icons.pinFilled,
                            size: theme.sizing.iconSizeXs,
                            color: colors.primary.withValues(alpha: 0.5),
                          ),
                        ),
                      Text(
                        _formatTimestamp(item.modifiedAt, l),
                        style: theme.typography.cardTimestamp.copyWith(
                          color: colors.onSurface.withValues(
                            alpha: theme.cardStyle.timestampOpacity,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                IgnorePointer(
                  ignoring: !_hovering,
                  child: AnimatedOpacity(
                    opacity: _hovering ? 1.0 : 0.0,
                    duration: const Duration(milliseconds: 120),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        _CardActionButton(
                          icon: theme.icons.paste,
                          tooltip: l.menuPaste,
                          onTap: widget.onTap,
                        ),
                        const SizedBox(width: 3),
                        if (_isPlainPasteable &&
                            widget.onPastePlain != null) ...[
                          _CardActionButton(
                            icon: Icons.notes_rounded,
                            tooltip: l.menuPastePlain,
                            onTap: widget.onPastePlain!,
                          ),
                          const SizedBox(width: 3),
                        ],
                        _CardActionButton(
                          icon: theme.icons.edit,
                          tooltip: l.menuEdit,
                          onTap: () => _editLabelColor(context),
                        ),
                        const SizedBox(width: 3),
                        _CardActionButton(
                          icon: item.isPinned
                              ? theme.icons.pinFilled
                              : theme.icons.pin,
                          tooltip: item.isPinned ? l.menuUnpin : l.menuPin,
                          onTap: widget.onPin,
                        ),
                        const SizedBox(width: 3),
                        _CardActionButton(
                          icon: theme.icons.delete,
                          tooltip: l.menuDelete,
                          onTap: widget.onDelete,
                          isDanger: true,
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildContent(
    AppThemeData theme,
    AppThemeColorScheme colors,
    ClipboardItem item,
  ) {
    switch (item.type) {
      case ClipboardContentType.image:
        return _buildImageContent(theme, colors, item);
      case ClipboardContentType.audio:
        return _buildMediaContent(theme, colors, item);
      case ClipboardContentType.video:
        return _buildMediaContent(theme, colors, item);
      case ClipboardContentType.file:
      case ClipboardContentType.folder:
        return _buildFileContent(theme, colors, item);
      case ClipboardContentType.link:
        return _buildLinkContent(theme, colors, item);
      case ClipboardContentType.text:
      case ClipboardContentType.unknown:
        return _buildTextContent(theme, colors, item);
    }
  }

  Widget _buildTextContent(
    AppThemeData theme,
    AppThemeColorScheme colors,
    ClipboardItem item,
  ) {
    return Text(
      item.content,
      style: theme.typography.cardContent.copyWith(
        color: colors.onSurface.withValues(
          alpha: theme.cardStyle.contentOpacity,
        ),
      ),
      maxLines: widget.isExpanded
          ? (widget.cardMaxLines ?? theme.sizing.cardMaxLines)
          : 2,
      overflow: TextOverflow.ellipsis,
    );
  }

  Widget _buildImageContent(
    AppThemeData theme,
    AppThemeColorScheme colors,
    ClipboardItem item,
  ) {
    // Show loading placeholder until path resolution completes
    if (!_imagePathResolved) {
      return Container(
        height: theme.sizing.cardImageHeight,
        decoration: BoxDecoration(
          color: colors.surfaceVariant,
          borderRadius: BorderRadius.circular(theme.radii.thumbnail),
        ),
      );
    }

    // Always use the original file for image type — best quality.
    // cacheWidth limits decode memory while keeping enough pixels for sharp display.
    final originalPath = item.content.trim();
    if (originalPath.isEmpty) {
      return Container(
        height: theme.sizing.cardImageHeight,
        decoration: BoxDecoration(
          color: colors.surfaceVariant,
          borderRadius: BorderRadius.circular(theme.radii.thumbnail),
        ),
        child: Center(
          child: Icon(
            theme.icons.image,
            size: theme.sizing.iconSizeLg,
            color: colors.onSurfaceMuted,
          ),
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ClipRRect(
          borderRadius: BorderRadius.circular(theme.radii.thumbnail),
          child: Container(
            height: theme.sizing.cardImageHeight,
            width: double.infinity,
            color: colors.surfaceVariant,
            child: Image.file(
              File(originalPath),
              fit: BoxFit.cover,
              cacheWidth: 700,
              errorBuilder: (_, e, s) => Center(
                child: Icon(
                  theme.icons.warning,
                  color: colors.warning,
                  size: theme.sizing.iconSizeLg,
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildFileContent(
    AppThemeData theme,
    AppThemeColorScheme colors,
    ClipboardItem item,
  ) {
    final files = item.content.split('\n').where((s) => s.isNotEmpty).toList();
    final available = item.isFileAvailable();
    final firstName = files.isEmpty
        ? ''
        : files.first.split(Platform.pathSeparator).last;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          firstName.isEmpty ? item.content : firstName,
          style: theme.typography.cardContent.copyWith(
            color: colors.onSurface.withValues(
              alpha: theme.cardStyle.contentOpacity,
            ),
          ),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        if (files.length > 1 || !available) ...[
          const SizedBox(height: 4),
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (files.length > 1) ...[
                _ExtBadge(
                  label: '+${files.length - 1}',
                  color: colors.onSurfaceMuted,
                ),
              ],
              if (!available) ...[
                if (files.length > 1) const SizedBox(width: 4),
                _ExtBadge(
                  label: AppLocalizations.of(context).fileNotFound,
                  color: colors.warning,
                ),
              ],
            ],
          ),
        ],
      ],
    );
  }

  Widget _buildMediaContent(
    AppThemeData theme,
    AppThemeColorScheme colors,
    ClipboardItem item,
  ) {
    final path = item.content.trim();
    final filename = path.isEmpty
        ? ''
        : path.split(Platform.pathSeparator).last;
    final isAudio = item.type == ClipboardContentType.audio;
    final typeColor = _typeColor(item.type, colors);

    final hasThumb = _imagePathResolved && _resolvedImagePath != null;

    // Video with thumbnail: show full-width thumbnail like images
    if (!isAudio && hasThumb) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          ClipRRect(
            borderRadius: BorderRadius.circular(theme.radii.thumbnail),
            child: Container(
              height: theme.sizing.cardImageHeight,
              width: double.infinity,
              color: colors.surfaceVariant,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  Image.file(
                    File(_resolvedImagePath!),
                    fit: BoxFit.contain,
                    cacheWidth: 700,
                    errorBuilder: (_, e, st) => _MediaIcon(
                      isAudio: false,
                      typeColor: typeColor,
                      radius: theme.radii.thumbnail,
                    ),
                  ),
                  Center(
                    child: Container(
                      width: 28,
                      height: 28,
                      decoration: BoxDecoration(
                        color: Colors.black.withValues(alpha: 0.45),
                        shape: BoxShape.circle,
                      ),
                      child: const Center(
                        child: Icon(
                          Icons.play_arrow_rounded,
                          size: 16,
                          color: Colors.white,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 4),
          Text(
            filename.isEmpty
                ? AppLocalizations.of(context).videoFile
                : filename,
            style: theme.typography.cardContent.copyWith(
              color: colors.onSurface.withValues(
                alpha: theme.cardStyle.contentOpacity,
              ),
              fontSize: 11,
            ),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
        ],
      );
    }

    // Audio or video without thumbnail: just show filename (type shown in header + footer badge)
    return Text(
      filename.isEmpty
          ? (isAudio
                ? AppLocalizations.of(context).audioFile
                : AppLocalizations.of(context).videoFile)
          : filename,
      style: theme.typography.cardContent.copyWith(
        color: colors.onSurface.withValues(
          alpha: theme.cardStyle.contentOpacity,
        ),
      ),
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
    );
  }

  Widget _buildLinkContent(
    AppThemeData theme,
    AppThemeColorScheme colors,
    ClipboardItem item,
  ) {
    final uri = Uri.tryParse(item.content.trim());
    final domain = uri?.host ?? '';
    final typeColor = _typeColor(item.type, colors);

    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                item.content.trim(),
                style: theme.typography.cardContent.copyWith(
                  color: colors.primary.withValues(alpha: 0.85),
                  decoration: TextDecoration.underline,
                  decorationColor: colors.primary.withValues(alpha: 0.3),
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              if (domain.isNotEmpty) ...[
                const SizedBox(height: 3),
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [_ExtBadge(label: domain, color: typeColor)],
                ),
              ],
            ],
          ),
        ),
      ],
    );
  }

  Map<String, dynamic>? _parseMetadata(ClipboardItem item) {
    if (item.metadata == null || item.metadata!.isEmpty) return null;
    try {
      return json.decode(item.metadata!) as Map<String, dynamic>;
    } catch (_) {
      return null;
    }
  }

  String _getExtForItem(ClipboardItem item) {
    if (item.type != ClipboardContentType.file &&
        item.type != ClipboardContentType.folder &&
        item.type != ClipboardContentType.audio &&
        item.type != ClipboardContentType.video &&
        item.type != ClipboardContentType.image) {
      return '';
    }
    final lines = item.content.split('\n').where((s) => s.isNotEmpty).toList();
    if (lines.isEmpty) return '';
    final firstName = lines.first.split(Platform.pathSeparator).last;
    return firstName.contains('.')
        ? firstName.split('.').last.toUpperCase()
        : '';
  }

  bool _hasFooter(ClipboardItem item) {
    if (_needsExpandToggle(item)) return true;
    if (item.pasteCount > 0) return true;
    if (_getExtForItem(item).isNotEmpty) return true;
    final meta = _parseMetadata(item);
    if (meta == null) return false;
    return meta.containsKey('file_size') ||
        meta.containsKey('size') ||
        meta.containsKey('width') ||
        meta.containsKey('video_width') ||
        meta.containsKey('duration');
  }

  Widget _buildFooter(
    AppThemeData theme,
    AppThemeColorScheme colors,
    ClipboardItem item,
  ) {
    final meta = _parseMetadata(item);
    final footerAlpha = theme.cardStyle.footerOpacity;
    final footerColor = colors.onSurface.withValues(alpha: footerAlpha);
    final footerStyle = theme.typography.cardFooter.copyWith(
      color: footerColor,
    );
    final iconColor = colors.onSurface.withValues(alpha: footerAlpha - 0.1);

    final ext = _getExtForItem(item);
    final typeColor = _typeColor(item.type, colors);
    final widgets = <Widget>[];

    // Image dimensions
    final w = meta?['width'] ?? meta?['video_width'];
    final h = meta?['height'] ?? meta?['video_height'];
    if (w != null && h != null) {
      widgets.add(
        _FooterChip(
          icon: Icons.aspect_ratio_rounded,
          label: '$w×$h',
          style: footerStyle,
          iconColor: iconColor,
          iconSize: theme.sizing.iconSizeXs,
        ),
      );
    }

    // File size
    final fileSize = meta?['file_size'] ?? meta?['size'];
    if (fileSize != null && fileSize is num && fileSize > 0) {
      widgets.add(
        _FooterChip(
          icon: Icons.storage_rounded,
          label: _formatFileSize(fileSize.toInt()),
          style: footerStyle,
          iconColor: iconColor,
          iconSize: theme.sizing.iconSizeXs,
        ),
      );
    }

    // Media duration
    final duration = meta?['duration'];
    if (duration != null && duration is num && duration > 0) {
      widgets.add(
        _FooterChip(
          icon: Icons.timer_outlined,
          label: _formatDuration(duration.toInt()),
          style: footerStyle,
          iconColor: iconColor,
          iconSize: theme.sizing.iconSizeXs,
        ),
      );
    }

    // Paste count
    if (item.pasteCount > 0) {
      widgets.add(Text('×${item.pasteCount}', style: footerStyle));
    }

    final showExpand = _needsExpandToggle(item);

    return Row(
      children: [
        if (ext.isNotEmpty) _ExtBadge(label: ext, color: typeColor),
        if (showExpand)
          Padding(
            padding: const EdgeInsets.only(left: 6),
            child: Material(
              color: Colors.transparent,
              child: InkWell(
                onTap: () => widget.onExpandToggle?.call(),
                canRequestFocus: false,
                borderRadius: BorderRadius.circular(8),
                hoverColor: colors.onSurface.withValues(alpha: 0.06),
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 4,
                    vertical: 1,
                  ),
                  child: Icon(
                    widget.isExpanded
                        ? Icons.expand_less_rounded
                        : Icons.expand_more_rounded,
                    size: 14,
                    color: colors.onSurface.withValues(alpha: 0.35),
                  ),
                ),
              ),
            ),
          ),
        const Spacer(),
        for (int i = 0; i < widgets.length; i++) ...[
          if (i > 0) const SizedBox(width: 8),
          widgets[i],
        ],
      ],
    );
  }

  static String _formatFileSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB';
  }

  static String _formatDuration(int seconds) {
    final h = seconds ~/ 3600;
    final m = (seconds % 3600) ~/ 60;
    final s = seconds % 60;
    if (h > 0) {
      return '$h:${m.toString().padLeft(2, '0')}:${s.toString().padLeft(2, '0')}';
    }
    return '$m:${s.toString().padLeft(2, '0')}';
  }

  Color _typeColor(ClipboardContentType type, AppThemeColorScheme colors) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return switch (type) {
      ClipboardContentType.text => colors.accentBlue,
      ClipboardContentType.image => colors.accentOrange,
      ClipboardContentType.file => colors.accentYellow,
      ClipboardContentType.folder => colors.accentYellow,
      ClipboardContentType.link => colors.accentGreen,
      ClipboardContentType.audio =>
        isDark ? const Color(0xFF7DD3FC) : const Color(0xFF075985),
      ClipboardContentType.video => colors.accentRed,
      ClipboardContentType.unknown => colors.onSurfaceMuted,
    };
  }

  String _contentTypeName(ClipboardContentType type, AppLocalizations l) =>
      switch (type) {
        ClipboardContentType.text => l.typeText,
        ClipboardContentType.image => l.typeImage,
        ClipboardContentType.file => l.typeFile,
        ClipboardContentType.folder => l.typeFolder,
        ClipboardContentType.link => l.typeLink,
        ClipboardContentType.audio => l.typeAudio,
        ClipboardContentType.video => l.typeVideo,
        ClipboardContentType.unknown => 'Unknown',
      };

  String _formatTimestamp(DateTime dt, AppLocalizations l) {
    final now = DateTime.now();
    final diff = now.difference(dt);

    if (diff.inMinutes < 1) return l.timeNow;
    if (diff.inMinutes < 60) return '${diff.inMinutes}m';
    if (diff.inHours < 24) return '${diff.inHours}h';
    if (diff.inDays < 7) return '${diff.inDays}d';
    return '${dt.month}/${dt.day}';
  }
}

class _CardActionButton extends StatelessWidget {
  const _CardActionButton({
    required this.icon,
    required this.onTap,
    this.tooltip,
    this.isDanger = false,
  });

  final IconData icon;
  final VoidCallback onTap;
  final String? tooltip;
  final bool isDanger;

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);
    final isDark = Theme.of(context).brightness == Brightness.dark;

    final bg = isDark
        ? colors.surfaceVariant
        : Colors.white.withValues(alpha: 0.95);

    final button = SizedBox(
      width: 30,
      height: 30,
      child: Material(
        color: bg,
        borderRadius: BorderRadius.circular(theme.radii.button),
        child: InkWell(
          onTap: onTap,
          canRequestFocus: false,
          borderRadius: BorderRadius.circular(theme.radii.button),
          hoverColor: isDanger
              ? colors.danger.withValues(alpha: 0.08)
              : colors.onSurface.withValues(alpha: 0.06),
          splashColor: isDanger
              ? colors.danger.withValues(alpha: 0.15)
              : colors.onSurface.withValues(alpha: 0.1),
          child: Center(
            child: Icon(
              icon,
              size: 13,
              color: isDanger
                  ? colors.danger.withValues(alpha: 0.7)
                  : colors.onSurface.withValues(alpha: 0.5),
            ),
          ),
        ),
      ),
    );

    if (tooltip != null) {
      return Tooltip(
        message: tooltip!,
        textStyle: const TextStyle(fontSize: 10, color: Colors.white),
        decoration: BoxDecoration(
          color: Colors.black.withValues(alpha: 0.75),
          borderRadius: BorderRadius.circular(4),
        ),
        preferBelow: false,
        verticalOffset: 16,
        waitDuration: const Duration(milliseconds: 400),
        child: button,
      );
    }
    return button;
  }
}

class _MediaIcon extends StatelessWidget {
  const _MediaIcon({
    required this.isAudio,
    required this.typeColor,
    required this.radius,
  });

  final bool isAudio;
  final Color typeColor;
  final double radius;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 44,
      height: 44,
      decoration: BoxDecoration(
        color: typeColor.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(radius),
      ),
      child: Center(
        child: Icon(
          isAudio
              ? Icons.music_note_rounded
              : Icons.play_circle_outline_rounded,
          size: 22,
          color: typeColor,
        ),
      ),
    );
  }
}

class _ExtBadge extends StatelessWidget {
  const _ExtBadge({required this.label, required this.color});

  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: 9,
          fontWeight: FontWeight.w600,
          color: color.withValues(alpha: 0.85),
          letterSpacing: 0.3,
        ),
      ),
    );
  }
}

class _FooterChip extends StatelessWidget {
  const _FooterChip({
    required this.icon,
    required this.label,
    required this.style,
    required this.iconColor,
    required this.iconSize,
  });

  final IconData icon;
  final String label;
  final TextStyle style;
  final Color iconColor;
  final double iconSize;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(icon, size: iconSize, color: iconColor),
        const SizedBox(width: 3),
        Text(label, style: style),
      ],
    );
  }
}

enum _ContextAction { paste, pastePlain, pin, edit, delete }

class _ContextMenuItem extends StatelessWidget {
  const _ContextMenuItem({
    required this.icon,
    required this.label,
    required this.colors,
    this.danger = false,
  });

  final IconData icon;
  final String label;
  final AppThemeColorScheme colors;
  final bool danger;

  @override
  Widget build(BuildContext context) {
    final color = danger ? colors.danger : colors.onSurface;
    return Row(
      children: [
        Icon(icon, size: 13, color: color.withValues(alpha: 0.7)),
        const SizedBox(width: 8),
        Text(label, style: TextStyle(fontSize: 12, color: color)),
      ],
    );
  }
}
