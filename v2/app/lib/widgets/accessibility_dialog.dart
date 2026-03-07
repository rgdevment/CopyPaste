import 'dart:async';

import 'package:flutter/material.dart';
import 'package:listener/listener.dart';

import '../l10n/app_localizations.dart';
import '../theme/theme_provider.dart';

class AccessibilityDialog extends StatefulWidget {
  const AccessibilityDialog({super.key});

  static Future<void> checkAndShow(BuildContext context) async {
    final granted = await ClipboardWriter.checkAccessibility();
    if (granted || !context.mounted) return;
    final theme = CopyPasteTheme.of(context);
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      barrierColor: Colors.black26,
      builder: (_) => CopyPasteTheme(
        themeData: theme,
        child: Theme(
          data: Theme.of(context),
          child: const AccessibilityDialog(),
        ),
      ),
    );
  }

  @override
  State<AccessibilityDialog> createState() => _AccessibilityDialogState();
}

class _AccessibilityDialogState extends State<AccessibilityDialog> {
  Timer? _pollTimer;

  @override
  void initState() {
    super.initState();
    _pollTimer = Timer.periodic(const Duration(seconds: 2), (_) async {
      final granted = await ClipboardWriter.checkAccessibility();
      if (granted && mounted) {
        Navigator.of(context).pop();
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(AppLocalizations.of(context).permissionsGranted),
            duration: const Duration(seconds: 2),
          ),
        );
      }
    });
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final colors = CopyPasteTheme.colorsOf(context);

    return AlertDialog(
      backgroundColor: colors.surface,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      icon: const Icon(Icons.security, size: 40, color: Colors.orange),
      title: Text(
        l.permissionsTitle,
        style: TextStyle(
          fontSize: 16,
          fontWeight: FontWeight.w600,
          color: colors.onSurface,
        ),
        textAlign: TextAlign.center,
      ),
      content: Text(
        l.permissionsMessage,
        style: TextStyle(fontSize: 13, color: colors.onSurfaceVariant),
        textAlign: TextAlign.center,
      ),
      actionsAlignment: MainAxisAlignment.center,
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(
            l.permissionsDismiss,
            style: TextStyle(color: colors.onSurfaceVariant),
          ),
        ),
        const SizedBox(width: 8),
        FilledButton(
          onPressed: () => ClipboardWriter.openAccessibilitySettings(),
          child: Text(l.permissionsOpenSettings),
        ),
      ],
    );
  }
}
