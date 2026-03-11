// coverage:ignore-file
import 'dart:async';
import 'dart:io';

import 'package:tray_manager/tray_manager.dart';

import 'linux_shell.dart';

class TrayIcon with TrayListener {
  TrayIcon({required this.onToggle, required this.onExit});

  final void Function() onToggle;
  final Future<void> Function() onExit;

  StreamSubscription<String>? _linuxEventsSubscription;

  static String get _iconPath {
    if (Platform.isMacOS) return 'assets/icons/icon_mac_tray.png';
    if (Platform.isLinux) return 'assets/icons/icon_tray_64.png';
    return 'assets/icons/icon_tray.ico';
  }

  Future<void> init() async {
    if (Platform.isLinux) {
      _linuxEventsSubscription ??= LinuxShell.events.listen((event) {
        switch (event) {
          case 'toggle':
            onToggle();
          case 'exit':
            onExit();
        }
      });
      await LinuxShell.initTray(
        iconPath: _iconPath,
        showHideLabel: 'Show/Hide',
        exitLabel: 'Exit',
        tooltip: 'CopyPaste',
      );
      return;
    }

    trayManager.addListener(this);
    await trayManager.setIcon(_iconPath);
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
    if (Platform.isLinux) {
      await LinuxShell.updateTray(
        iconPath: _iconPath,
        showHideLabel: showHideLabel,
        exitLabel: exitLabel,
        tooltip: tooltip,
      );
      return;
    }

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
    if (Platform.isLinux) {
      await _linuxEventsSubscription?.cancel();
      _linuxEventsSubscription = null;
      await LinuxShell.destroyTray();
      return;
    }

    trayManager.removeListener(this);
    await trayManager.destroy();
  }
}
