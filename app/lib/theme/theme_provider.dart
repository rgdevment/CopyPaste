import 'package:flutter/material.dart';

import 'app_theme_data.dart';

class CopyPasteTheme extends InheritedWidget {
  const CopyPasteTheme({
    required this.themeData,
    required super.child,
    super.key,
  });

  final AppThemeData themeData;

  static AppThemeData of(BuildContext context) {
    final widget = context.dependOnInheritedWidgetOfExactType<CopyPasteTheme>();
    if (widget == null) {
      throw FlutterError('No CopyPasteTheme found in context');
    }
    return widget.themeData;
  }

  static AppThemeColorScheme colorsOf(BuildContext context) {
    final theme = of(context);
    final brightness = Theme.of(context).brightness;
    return brightness == Brightness.dark ? theme.dark : theme.light;
  }

  @override
  bool updateShouldNotify(CopyPasteTheme oldWidget) =>
      themeData.id != oldWidget.themeData.id;
}
