import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../theme/theme_provider.dart';

class EmptyState extends StatelessWidget {
  const EmptyState({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = CopyPasteTheme.of(context);
    final colors = CopyPasteTheme.colorsOf(context);

    return Center(
      child: Padding(
        padding: const EdgeInsets.only(bottom: 40),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 64,
              height: 64,
              decoration: BoxDecoration(
                color: colors.primary.withValues(alpha: 0.08),
                borderRadius: BorderRadius.circular(16),
              ),
              child: Icon(
                Icons.content_paste_rounded,
                size: 28,
                color: colors.primary.withValues(alpha: 0.4),
              ),
            ),
            const SizedBox(height: 16),
            Text(
              AppLocalizations.of(context).emptyState,
              style: theme.typography.emptyState.copyWith(
                color: colors.onSurfaceVariant,
                fontWeight: FontWeight.w500,
              ),
            ),
            const SizedBox(height: 6),
            Text(
              'Copy something to get started',
              style: theme.typography.cardFooter.copyWith(
                color: colors.onSurfaceMuted,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
