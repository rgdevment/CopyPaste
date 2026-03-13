import 'package:flutter/material.dart';
import 'package:copypaste/l10n/app_localizations.dart';
import 'package:copypaste/theme/compact_theme.dart';
import 'package:copypaste/theme/theme_provider.dart';

Widget wrapWidget(Widget child, {Brightness brightness = Brightness.light}) {
  return MaterialApp(
    locale: const Locale('en'),
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    theme: ThemeData(brightness: brightness),
    home: CopyPasteTheme(
      themeData: CompactTheme(),
      child: Scaffold(body: child),
    ),
  );
}
