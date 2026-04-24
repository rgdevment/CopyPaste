import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../helpers/url_helper.dart';
import '../l10n/app_localizations.dart';
import '../services/install_channel.dart';
import '../services/release_manifest_service.dart';

class BlockedVersionScreen extends StatelessWidget {
  const BlockedVersionScreen({
    required this.currentVersion,
    required this.manifest,
    super.key,
  });

  final String currentVersion;
  final ReleaseManifest manifest;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final cs = Theme.of(context).colorScheme;
    final tt = Theme.of(context).textTheme;

    final channel = InstallChannelDetector.detect();
    final channelInfo =
        manifest.channels[InstallChannelDetector.manifestKey(channel)];
    final notes = manifest.notesFor(
      Localizations.localeOf(context).toLanguageTag(),
    );

    final action = _resolveAction(context, l, channel, channelInfo);

    return Scaffold(
      backgroundColor: cs.surface,
      body: Center(
        child: SizedBox(
          width: 420,
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 32),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 64,
                  height: 64,
                  decoration: BoxDecoration(
                    color: cs.errorContainer.withValues(alpha: 0.6),
                    borderRadius: BorderRadius.circular(20),
                  ),
                  child: Icon(
                    Icons.lock_outline,
                    size: 32,
                    color: cs.onErrorContainer,
                  ),
                ),
                const SizedBox(height: 16),
                Text(
                  l.blockedTitle,
                  style: tt.titleLarge?.copyWith(
                    fontWeight: FontWeight.w700,
                    letterSpacing: -0.3,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 12),
                Text(
                  l.blockedDescription(
                    currentVersion,
                    manifest.minimumSupported,
                  ),
                  style: tt.bodyMedium?.copyWith(
                    color: cs.onSurfaceVariant,
                    height: 1.5,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 12),
                Text(
                  notes?.summary ?? l.blockedReasonGeneric,
                  style: tt.bodySmall?.copyWith(color: cs.onSurfaceVariant),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 24),
                if (action != null)
                  FilledButton(
                    onPressed: action.onPressed,
                    style: FilledButton.styleFrom(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 24,
                        vertical: 14,
                      ),
                    ),
                    child: Text(action.label),
                  )
                else
                  Text(
                    l.blockedFallbackHint,
                    style: tt.bodySmall?.copyWith(color: cs.onSurfaceVariant),
                    textAlign: TextAlign.center,
                  ),
                const SizedBox(height: 12),
                TextButton(
                  onPressed: () => exit(0),
                  child: Text(l.blockedQuit),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  _BlockAction? _resolveAction(
    BuildContext context,
    AppLocalizations l,
    InstallChannel channel,
    ChannelInfo? info,
  ) {
    if (info == null) return null;

    if (channel == InstallChannel.msStore) {
      final url = info.url;
      if (url == null) return null;
      return _BlockAction(
        label: l.updateActionOpenStore,
        onPressed: () => UrlHelper.open(url),
      );
    }

    if (channel == InstallChannel.homebrew || channel == InstallChannel.snap) {
      final cmd = info.command;
      if (cmd == null) return null;
      return _BlockAction(
        label: l.updateActionCopyBrew,
        onPressed: () async {
          await Clipboard.setData(ClipboardData(text: cmd));
          if (!context.mounted) return;
          ScaffoldMessenger.of(
            context,
          ).showSnackBar(SnackBar(content: Text(l.updateActionCopied)));
        },
      );
    }

    final url = info.url;
    if (url == null) return null;
    return _BlockAction(
      label: l.updateActionDownload,
      onPressed: () => UrlHelper.open(url),
    );
  }
}

class _BlockAction {
  _BlockAction({required this.label, required this.onPressed});
  final String label;
  final VoidCallback onPressed;
}
