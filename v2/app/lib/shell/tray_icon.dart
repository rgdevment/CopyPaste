// coverage:ignore-file
import 'dart:io';

import 'package:tray_manager/tray_manager.dart';

class TrayIcon with TrayListener {
  TrayIcon({required this.onToggle, required this.onExit});

  final void Function() onToggle;
  final Future<void> Function() onExit;

  static String get _iconPath {
    if (Platform.isMacOS) return 'assets/icons/icon_tray_32.png';
    return 'assets/icons/icon_tray.ico';
  }

  Future<void> init() async {
    trayManager.addListener(this);
    await trayManager.setIcon(_iconPath);
    // Set a basic menu immediately so right-click works even before the first
    // build() triggers rebuild() with localized strings.
    await trayManager.setContextMenu(
      Menu(
        items: [
          MenuItem(key: 'toggle', label: 'Show/Hide'),
          MenuItem.separator(),
          MenuItem(key: 'exit', label: 'Exit'),
        ],
      ),
    );
  }

  Future<void> rebuild({
    required String showHideLabel,
    required String exitLabel,
    required String tooltip,
  }) async {
    await trayManager.setToolTip(tooltip);
    await trayManager.setContextMenu(
      Menu(
        items: [
          MenuItem(key: 'toggle', label: showHideLabel),
          MenuItem.separator(),
          MenuItem(key: 'exit', label: exitLabel),
        ],
      ),
    );
  }

  @override
  void onTrayIconMouseDown() => onToggle();

  @override
  void onTrayIconRightMouseDown() {
    trayManager.popUpContextMenu();
  }

  @override
  void onTrayMenuItemClick(MenuItem menuItem) {
    switch (menuItem.key) {
      case 'toggle':
        onToggle();
      case 'exit':
        onExit();
    }
  }

  Future<void> dispose() async {
    trayManager.removeListener(this);
    await trayManager.destroy();
  }
}
