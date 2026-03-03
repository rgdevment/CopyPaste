import 'dart:ui' show Size;

import 'package:flutter_acrylic/flutter_acrylic.dart';
import 'package:window_manager/window_manager.dart';

class AppWindow {
  AppWindow({this.onVisibilityChanged});

  static const double _width = 420;
  static const double _height = 620;

  final void Function(bool visible)? onVisibilityChanged;
  bool _visible = false;
  bool pinned = false;

  bool get isVisible => _visible;

  Future<void> init() async {
    await Window.initialize();
    await Window.setEffect(effect: WindowEffect.mica);

    await windowManager.setTitle('CopyPaste');
    await windowManager.setSize(const Size(_width, _height));
    await windowManager.setMinimumSize(const Size(_width, 400));
    await windowManager.setMaximumSize(const Size(_width, 900));
    await windowManager.setTitleBarStyle(TitleBarStyle.hidden);
    await windowManager.setAlwaysOnTop(true);
    await windowManager.setSkipTaskbar(true);
    await windowManager.setResizable(false);
    await windowManager.setPreventClose(true);

    await windowManager.center();
    await windowManager.hide();
    _visible = false;
  }

  Future<void> show() async {
    await windowManager.center();
    await windowManager.show();
    await windowManager.focus();
    _visible = true;
    onVisibilityChanged?.call(true);
  }

  Future<void> hide() async {
    await windowManager.hide();
    _visible = false;
    onVisibilityChanged?.call(false);
  }

  Future<void> toggle() async {
    if (_visible) {
      await hide();
    } else {
      await show();
    }
  }

  void hideIfNotPinned() {
    if (!pinned && _visible) {
      hide();
    }
  }
}
