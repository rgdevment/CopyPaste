import 'dart:async';

import 'package:flutter/material.dart';
import 'package:listener/listener.dart';

import '../l10n/app_localizations.dart';

/// Phases of the permission dialog, driving which message and actions are shown.
enum _DialogPhase {
  /// First-time request — standard "you need to grant permission" message.
  initial,

  /// Polling timed out — suggest restarting the app.
  retryNeeded,
}

class AccessibilityDialog extends StatefulWidget {
  const AccessibilityDialog({required this.previouslyGranted, super.key});

  /// Whether we know the user had granted permission before (stored in config).
  /// When true, the dialog shows Gatekeeper-specific instructions (remove +
  /// re-add in Accessibility settings) instead of the standard first-time msg.
  final bool previouslyGranted;

  /// Checks accessibility status and shows the dialog if not granted.
  ///
  /// Returns `true` if permission is (or became) granted, `false` otherwise.
  ///
  /// Uses [ClipboardWriter.checkAccessibility] (read-only) to avoid triggering
  /// the macOS system prompt before the user reads the explanation dialog.
  ///
  /// [previouslyGranted] drives which message variant is shown — see
  /// [_DialogPhase] and [previouslyGranted] for details.
  static Future<bool> checkAndShow(
    BuildContext context, {
    bool previouslyGranted = false,
  }) async {
    final granted = await ClipboardWriter.checkAccessibility();
    if (granted || !context.mounted) return granted;
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      barrierColor: Colors.black26,
      builder: (_) => Theme(
        data: Theme.of(context),
        child: AccessibilityDialog(previouslyGranted: previouslyGranted),
      ),
    );
    // Return final state after dialog dismissed.
    return ClipboardWriter.checkAccessibility();
  }

  @override
  State<AccessibilityDialog> createState() => _AccessibilityDialogState();
}

class _AccessibilityDialogState extends State<AccessibilityDialog> {
  Timer? _pollTimer;
  int _pollCount = 0;
  _DialogPhase _phase = _DialogPhase.initial;
  bool _checking = false;

  /// After this many 1-second polls without success, switch to
  /// [_DialogPhase.retryNeeded] to suggest restarting the app.
  static const _maxPollsBeforeRetry = 30;

  @override
  void initState() {
    super.initState();
    _pollTimer = Timer.periodic(const Duration(seconds: 1), (_) async {
      _pollCount++;
      final granted = await ClipboardWriter.checkAccessibility();
      if (granted && mounted) {
        _pollTimer?.cancel();
        Navigator.of(context).pop();
        return;
      }
      if (_pollCount >= _maxPollsBeforeRetry &&
          _phase == _DialogPhase.initial &&
          mounted) {
        setState(() => _phase = _DialogPhase.retryNeeded);
      }
    });
  }

  /// Manual "Check Again" action — uses [requestAccessibility] which calls
  /// `AXIsProcessTrustedWithOptions(prompt: true)` to give the OS another
  /// chance to recognise the current process identity.
  Future<void> _manualCheck() async {
    if (_checking) return;
    setState(() => _checking = true);

    final granted = await ClipboardWriter.requestAccessibility();

    if (granted && mounted) {
      _pollTimer?.cancel();
      Navigator.of(context).pop();
    } else if (mounted) {
      setState(() {
        _checking = false;
        _phase = _DialogPhase.retryNeeded;
      });
    }
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
    final isStale = widget.previouslyGranted;

    return AlertDialog(
      backgroundColor: cs.surface,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      icon: Icon(
        isStale ? Icons.warning_amber_rounded : Icons.security,
        size: 40,
        color: isStale ? Colors.red : Colors.orange,
      ),
      title: Text(
        isStale ? l.permissionsResetTitle : l.permissionsTitle,
        style: TextStyle(
          fontSize: 16,
          fontWeight: FontWeight.w600,
          color: cs.onSurface,
        ),
        textAlign: TextAlign.center,
      ),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            isStale
                ? l.permissionsResetMessage
                : (_phase == _DialogPhase.retryNeeded
                      ? l.permissionsRestartMessage
                      : l.permissionsMessage),
            style: TextStyle(fontSize: 13, color: cs.onSurfaceVariant),
            textAlign: TextAlign.center,
          ),
        ],
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
        if (_phase == _DialogPhase.retryNeeded || isStale) ...[
          const SizedBox(width: 4),
          OutlinedButton(
            onPressed: _checking ? null : _manualCheck,
            child: Text(_checking ? '...' : l.permissionsCheckAgain),
          ),
        ],
        const SizedBox(width: 4),
        FilledButton(
          onPressed: () => ClipboardWriter.openAccessibilitySettings(),
          child: Text(l.permissionsOpenSettings),
        ),
      ],
    );
  }
}
