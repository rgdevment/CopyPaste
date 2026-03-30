import 'package:flutter/material.dart';

import '../helpers/url_helper.dart';
import '../l10n/app_localizations.dart';

class WaylandUnsupportedScreen extends StatelessWidget {
  const WaylandUnsupportedScreen({required this.onClose, super.key});

  final VoidCallback onClose;

  static const _repoUrl = 'https://github.com/rgdevment/CopyPaste';

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final cs = Theme.of(context).colorScheme;
    final tt = Theme.of(context).textTheme;

    return Scaffold(
      backgroundColor: cs.surface,
      body: Center(
        child: SizedBox(
          width: 320,
          child: ScrollConfiguration(
            behavior: ScrollConfiguration.of(
              context,
            ).copyWith(scrollbars: false),
            child: SingleChildScrollView(
              padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 36),
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
                    l.waylandUnsupportedTitle,
                    style: tt.titleLarge?.copyWith(
                      fontWeight: FontWeight.w700,
                      letterSpacing: -0.3,
                    ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 10),
                  _Badge(label: l.waylandUnsupportedBadge, colorScheme: cs),
                  const SizedBox(height: 20),
                  Divider(color: cs.outlineVariant, height: 1),
                  const SizedBox(height: 20),
                  Text(
                    l.waylandUnsupportedBody,
                    style: tt.bodyMedium?.copyWith(
                      color: cs.onSurfaceVariant,
                      height: 1.6,
                    ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 28),
                  SizedBox(
                    width: double.infinity,
                    child: FilledButton.icon(
                      onPressed: () => UrlHelper.open(_repoUrl),
                      icon: const Icon(Icons.open_in_new_rounded, size: 15),
                      label: Text(l.waylandUnsupportedGitHub),
                    ),
                  ),
                  const SizedBox(height: 8),
                  SizedBox(
                    width: double.infinity,
                    child: OutlinedButton(
                      onPressed: onClose,
                      child: Text(l.waylandUnsupportedClose),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _Badge extends StatelessWidget {
  const _Badge({required this.label, required this.colorScheme});

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
      child: Text(
        label,
        style: TextStyle(
          fontSize: 12,
          color: colorScheme.primary,
          fontWeight: FontWeight.w500,
        ),
      ),
    );
  }
}
