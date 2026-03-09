import 'dart:async';

import 'package:flutter/material.dart';
import 'package:listener/listener.dart';

import '../l10n/app_localizations.dart';

class AccessibilityDialog extends StatefulWidget {
  const AccessibilityDialog({super.key});

  /// Checks accessibility status and shows the dialog if not granted.
  ///
  /// Uses [checkAccessibility] (read-only check) instead of
  /// [requestAccessibility] to avoid triggering the macOS system prompt
  /// before the user has had a chance to read the explanation dialog.
  static Future<void> checkAndShow(BuildContext context) async {
    final granted = await ClipboardWriter.checkAccessibility();
    if (granted || !context.mounted) return;
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      barrierColor: Colors.black26,
      builder: (_) =>
          Theme(data: Theme.of(context), child: const AccessibilityDialog()),
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
        _pollTimer?.cancel();
        Navigator.of(context).pop();
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
    final cs = Theme.of(context).colorScheme;

    return AlertDialog(
      backgroundColor: cs.surface,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      icon: const Icon(Icons.security, size: 40, color: Colors.orange),
      title: Text(
        l.permissionsTitle,
        style: TextStyle(
          fontSize: 16,
          fontWeight: FontWeight.w600,
          color: cs.onSurface,
        ),
        textAlign: TextAlign.center,
      ),
      content: Text(
        l.permissionsMessage,
        style: TextStyle(fontSize: 13, color: cs.onSurfaceVariant),
        textAlign: TextAlign.center,
      ),
      actionsAlignment: MainAxisAlignment.center,
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(
            l.permissionsDismiss,
            style: TextStyle(color: cs.onSurfaceVariant),
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
