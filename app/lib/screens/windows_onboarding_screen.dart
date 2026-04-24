import 'package:core/core.dart';
import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';

class WindowsOnboardingScreen extends StatefulWidget {
  const WindowsOnboardingScreen({
    required this.hotkey,
    required this.initialConfig,
    required this.onDismiss,
    required this.onSettings,
    super.key,
  });

  final String hotkey;
  final AppConfig initialConfig;
  final void Function(AppConfig updated) onDismiss;
  final void Function(AppConfig updated) onSettings;

  @override
  State<WindowsOnboardingScreen> createState() =>
      _WindowsOnboardingScreenState();
}

class _WindowsOnboardingScreenState extends State<WindowsOnboardingScreen> {
  AppConfig _buildConfig() => widget.initialConfig;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final cs = Theme.of(context).colorScheme;
    final tt = Theme.of(context).textTheme;

    return Scaffold(
      backgroundColor: cs.surface,
      body: Center(
        child: SizedBox(
          width: 360,
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 32),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                ClipRRect(
                  borderRadius: BorderRadius.circular(20),
                  child: Image.asset(
                    'assets/icons/icon_app_256.png',
                    width: 64,
                    height: 64,
                  ),
                ),
                const SizedBox(height: 14),
                Text(
                  l.onboardingTitle,
                  style: tt.titleLarge?.copyWith(
                    fontWeight: FontWeight.w700,
                    letterSpacing: -0.3,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 4),
                Text(
                  l.onboardingSubtitle,
                  style: tt.bodyMedium?.copyWith(color: cs.onSurfaceVariant),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 14),
                _PrivacyBadge(label: l.onboardingPrivacyBadge, colorScheme: cs),
                const SizedBox(height: 20),
                Divider(color: cs.outlineVariant, height: 1),
                const SizedBox(height: 16),
                Text(
                  l.onboardingDescription(widget.hotkey),
                  style: tt.bodyMedium?.copyWith(
                    color: cs.onSurfaceVariant,
                    height: 1.5,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                _HotkeyChip(hotkey: widget.hotkey, colorScheme: cs),
                const SizedBox(height: 8),
                Text(
                  l.onboardingTrayHint,
                  style: tt.bodySmall?.copyWith(color: cs.onSurfaceVariant),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 20),
                Wrap(
                  alignment: WrapAlignment.center,
                  spacing: 10,
                  runSpacing: 8,
                  children: [
                    OutlinedButton(
                      onPressed: () => widget.onSettings(_buildConfig()),
                      style: OutlinedButton.styleFrom(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 20,
                          vertical: 12,
                        ),
                      ),
                      child: Text(l.onboardingSettingsButton),
                    ),
                    FilledButton(
                      onPressed: () => widget.onDismiss(_buildConfig()),
                      style: FilledButton.styleFrom(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 20,
                          vertical: 12,
                        ),
                      ),
                      child: Text(l.onboardingDismissButton),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _OnboardingToggle extends StatelessWidget {
  const _OnboardingToggle({
    required this.label,
    required this.value,
    required this.colorScheme,
    required this.onChanged,
  });

  final String label;
  final bool value;
  final ColorScheme colorScheme;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: Text(
            label,
            style: TextStyle(fontSize: 13, color: colorScheme.onSurface),
          ),
        ),
        Switch(value: value, onChanged: onChanged),
      ],
    );
  }
}

class _PrivacyBadge extends StatelessWidget {
  const _PrivacyBadge({required this.label, required this.colorScheme});

  final String label;
  final ColorScheme colorScheme;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: colorScheme.primary.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(20),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.lock_outline_rounded,
            size: 13,
            color: colorScheme.primary,
          ),
          const SizedBox(width: 6),
          Flexible(
            child: Text(
              label,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                fontSize: 12,
                color: colorScheme.primary,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _HotkeyChip extends StatelessWidget {
  const _HotkeyChip({required this.hotkey, required this.colorScheme});

  final String hotkey;
  final ColorScheme colorScheme;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: colorScheme.outlineVariant),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.keyboard_rounded,
            size: 15,
            color: colorScheme.onSurfaceVariant,
          ),
          const SizedBox(width: 7),
          Text(
            hotkey,
            style: TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w600,
              color: colorScheme.onSurface,
              letterSpacing: 0.2,
            ),
          ),
        ],
      ),
    );
  }
}
