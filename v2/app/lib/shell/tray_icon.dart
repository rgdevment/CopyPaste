import 'package:tray_manager/tray_manager.dart';

class TrayIcon with TrayListener {
  TrayIcon({
    required this.onToggle,
    required this.onExit,
  });

  final void Function() onToggle;
  final Future<void> Function() onExit;

  Future<void> init() async {
    trayManager.addListener(this);
    await trayManager.setIcon('assets/icons/icon_tray_32.png');
    await trayManager.setToolTip('CopyPaste');
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

  @override
  void onTrayIconMouseDown() => onToggle();

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
