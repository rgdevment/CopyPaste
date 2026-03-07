import Cocoa
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return false
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }

  override func applicationDidFinishLaunching(_ notification: Notification) {
    super.applicationDidFinishLaunching(notification)
    stripMenuBar()
  }

  private func stripMenuBar() {
    guard let menu = NSApp.mainMenu else { return }

    // Keep only App (index 0) and Edit (index 1) menus
    while menu.items.count > 2 {
      menu.removeItem(at: menu.items.count - 1)
    }

    // Remove keyboard shortcuts that bypass Dart cleanup
    if let appMenu = menu.items.first?.submenu {
      for item in appMenu.items where
        item.action == #selector(NSApplication.terminate(_:)) ||
        item.action == #selector(NSApplication.hide(_:)) ||
        item.action == #selector(NSApplication.hideOtherApplications(_:)) {
        item.keyEquivalent = ""
        item.keyEquivalentModifierMask = []
      }
    }
  }
}
