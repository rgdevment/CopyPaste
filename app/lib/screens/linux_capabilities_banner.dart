import 'package:core/core.dart';
import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../services/linux_capabilities.dart';
import '../theme/theme_provider.dart';

typedef LinuxBannerDismissCallback = Future<void> Function(
  AppConfig Function(AppConfig) update,
);

class LinuxCapabilitiesBanner extends StatelessWidget {
  const LinuxCapabilitiesBanner({
    super.key,
    required this.config,
    required this.capabilities,
    required this.onDismiss,
  });

  final AppConfig config;
  final LinuxCapabilities capabilities;
  final LinuxBannerDismissCallback onDismiss;

  _BannerKind? _resolveActiveBanner() {
    if (!capabilities.isUsable) return null;
    if (!capabilities.hasAppIndicator &&
        !config.linuxAppindicatorWarningDismissed) {
      return _BannerKind.appIndicator;
    }
    if (!capabilities.hasXTest && !config.linuxXtestWarningDismissed) {
      return _BannerKind.xtest;
    }
    if (!capabilities.hasClipboardManager &&
        !config.linuxClipboardManagerWarningDismissed) {
      return _BannerKind.clipboardManager;
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final kind = _resolveActiveBanner();
    if (kind == null) return const SizedBox.shrink();

    final l = AppLocalizations.of(context);
    final colors = CopyPasteTheme.colorsOf(context);
    final (title, body) = switch (kind) {
      _BannerKind.appIndicator => (
          l.linuxAppindicatorBannerTitle,
          l.linuxAppindicatorBannerBody,
        ),
      _BannerKind.xtest => (
          l.linuxXtestBannerTitle,
          l.linuxXtestBannerBody,
        ),
      _BannerKind.clipboardManager => (
          l.linuxClipboardManagerBannerTitle,
          l.linuxClipboardManagerBannerBody,
        ),
    };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
      color: colors.primary.withValues(alpha: 0.10),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.warning_amber_rounded,
            size: 16,
            color: colors.primary,
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                    color: colors.onSurface,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  body,
                  style: TextStyle(
                    fontSize: 11,
                    color: colors.onSurfaceMuted,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          GestureDetector(
            onTap: () => _dismiss(kind),
            child: MouseRegion(
              cursor: SystemMouseCursors.click,
              child: Tooltip(
                message: l.linuxBannerDismiss,
                child: Icon(
                  Icons.close_rounded,
                  size: 16,
                  color: colors.onSurfaceMuted,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _dismiss(_BannerKind kind) {
    return onDismiss((c) {
      switch (kind) {
        case _BannerKind.appIndicator:
          return c.copyWith(linuxAppindicatorWarningDismissed: true);
        case _BannerKind.xtest:
          return c.copyWith(linuxXtestWarningDismissed: true);
        case _BannerKind.clipboardManager:
          return c.copyWith(linuxClipboardManagerWarningDismissed: true);
      }
    });
  }
}

enum _BannerKind { appIndicator, xtest, clipboardManager }
