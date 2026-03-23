import 'dart:async';

import 'package:flutter/material.dart';
import 'package:listener/listener.dart';

import '../l10n/app_localizations.dart';

class PermissionGateScreen extends StatefulWidget {
  const PermissionGateScreen({
    required this.onGranted,
    required this.previouslyGranted,
    this.onRestart,
    super.key,
  });

  final VoidCallback onGranted;
  final bool previouslyGranted;
  final VoidCallback? onRestart;

  @override
  State<PermissionGateScreen> createState() => _PermissionGateScreenState();
}

class _PermissionGateScreenState extends State<PermissionGateScreen>
    with SingleTickerProviderStateMixin {
  Timer? _pollTimer;
  int _pollCount = 0;
  bool _timedOut = false;
  bool _checking = false;

  late final AnimationController _pulseController;
  late final Animation<double> _pulseAnimation;

  static const _maxPollsBeforeHint = 30;

  @override
  void initState() {
    super.initState();
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat(reverse: true);
    _pulseAnimation = Tween<double>(begin: 0.4, end: 1.0).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );

    _pollTimer = Timer.periodic(const Duration(seconds: 1), (_) async {
      _pollCount++;
      final granted = await ClipboardWriter.checkAccessibility();
      if (granted && mounted) {
        _pollTimer?.cancel();
        widget.onGranted();
        return;
      }
      if (_pollCount >= _maxPollsBeforeHint && !_timedOut && mounted) {
        _pollTimer?.cancel();
        setState(() => _timedOut = true);
      }
    });
  }

  Future<void> _manualCheck() async {
    if (_checking) return;
    setState(() => _checking = true);
    final granted = await ClipboardWriter.requestAccessibility();
    if (granted && mounted) {
      _pollTimer?.cancel();
      widget.onGranted();
    } else if (mounted) {
      setState(() {
        _checking = false;
        _timedOut = true;
      });
    }
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final cs = Theme.of(context).colorScheme;
    final isStale = widget.previouslyGranted;

    return Scaffold(
      backgroundColor: cs.surface,
      body: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.symmetric(horizontal: 40, vertical: 28),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              ClipRRect(
                borderRadius: BorderRadius.circular(20),
                child: Image.asset(
                  'assets/icons/icon_app_256.png',
                  width: 72,
                  height: 72,
                ),
              ),
              const SizedBox(height: 16),
              Text(
                'CopyPaste',
                style: TextStyle(
                  fontSize: 22,
                  fontWeight: FontWeight.w700,
                  color: cs.onSurface,
                  letterSpacing: -0.3,
                ),
              ),
              const SizedBox(height: 20),
              _StatusRow(
                icon: isStale
                    ? Icons.warning_amber_rounded
                    : Icons.lock_outline_rounded,
                iconColor: isStale ? Colors.red : Colors.orange,
                label: isStale ? l.permissionsResetTitle : l.permissionsTitle,
                colorScheme: cs,
              ),
              const SizedBox(height: 16),
              Text(
                isStale
                    ? l.permissionsResetMessage
                    : (_timedOut
                          ? l.permissionsRestartMessage
                          : l.permissionsMessage),
                style: TextStyle(
                  fontSize: 13,
                  color: cs.onSurfaceVariant,
                  height: 1.5,
                ),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 24),
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (_timedOut || isStale) ...[
                    OutlinedButton(
                      onPressed: _checking ? null : _manualCheck,
                      style: OutlinedButton.styleFrom(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 20,
                          vertical: 12,
                        ),
                      ),
                      child: Text(_checking ? '...' : l.permissionsCheckAgain),
                    ),
                    const SizedBox(width: 12),
                  ],
                  FilledButton.icon(
                    onPressed: () =>
                        ClipboardWriter.openAccessibilitySettings(),
                    icon: const Icon(Icons.settings, size: 18),
                    label: Text(l.permissionsOpenSettings),
                    style: FilledButton.styleFrom(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 20,
                        vertical: 12,
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 20),
              if (_timedOut || isStale)
                TextButton.icon(
                  onPressed: widget.onRestart,
                  icon: Icon(
                    Icons.refresh_rounded,
                    size: 16,
                    color: cs.onSurfaceVariant.withValues(alpha: 0.7),
                  ),
                  label: Text(
                    l.permissionsRestartApp,
                    style: TextStyle(
                      fontSize: 12,
                      color: cs.onSurfaceVariant.withValues(alpha: 0.7),
                    ),
                  ),
                ),
              if (!_timedOut && !isStale)
                FadeTransition(
                  opacity: _pulseAnimation,
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      SizedBox(
                        width: 12,
                        height: 12,
                        child: CircularProgressIndicator(
                          strokeWidth: 1.5,
                          color: cs.onSurfaceVariant.withValues(alpha: 0.5),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Text(
                        l.permissionsWaiting,
                        style: TextStyle(
                          fontSize: 11,
                          color: cs.onSurfaceVariant.withValues(alpha: 0.6),
                        ),
                      ),
                    ],
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _StatusRow extends StatelessWidget {
  const _StatusRow({
    required this.icon,
    required this.iconColor,
    required this.label,
    required this.colorScheme,
  });

  final IconData icon;
  final Color iconColor;
  final String label;
  final ColorScheme colorScheme;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
      decoration: BoxDecoration(
        color: iconColor.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(10),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 20, color: iconColor),
          const SizedBox(width: 10),
          Flexible(
            child: Text(
              label,
              style: TextStyle(
                fontSize: 13,
                fontWeight: FontWeight.w600,
                color: colorScheme.onSurface,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
