# UI CONFIGURATION STANDARD - COMPLETE GUIDE

## 1. SCOPE
This document describes the configuration options available to customize the user interface behavior of CopyPaste. All settings are centralized in UIConfig.cs and can be modified in App.xaml.cs before window initialization.

## 2. PARAMETERS AND METRICS TABLE
| Parameter | Default | Recommended | Impact |
| :--- | :--- | :--- | :--- |
| PageSize | 20 | 15 - 30 | Batch items to load per page |
| MaxItemsBeforeCleanup | 100 | 50 - 150 | RAM limit before purging on window blur |
| ScrollLoadThreshold | 100px | 50 - 200px | Distance from bottom to trigger load |
| WindowWidth | 400px | 350 - 500px | Sidebar window width in pixels |
| WindowMarginTop | 8px | - | Vertical margin from top of workspace |
| WindowMarginBottom | 16px | - | Vertical margin from bottom of workspace |
| Hotkey.VirtualKey | 0x56 (V) | - | Hex code for the global shortcut key |
| Hotkey.UseWinKey | true | - | Use Windows key as modifier |
| Hotkey.UseAltKey | true | - | Include Alt modifier (Always true) |

## 3. DETAILED COMPONENT DEFINITIONS

### 3.1 Pagination and Memory Logic
* PageSize: Number of clipboard items to load per page. High values mean slower initial load but fewer frequent fetches.
* MaxItemsBeforeCleanup: Maximum items in memory before cleanup. Triggers when the window is deactivated (blur).
* ScrollLoadThreshold: Displacement threshold (in pixels) from the bottom to load more elements.

### 3.2 Global Keyboard Shortcut
CopyPaste registers a global keyboard shortcut to show/hide the window.
* Default: Win + Alt + V.
* Automatic Fallback: Ctrl + Alt + V (if Win key cannot be registered).
* Common Key Codes:
    * 0x43 = C
    * 0x56 = V
    * 0x58 = X
    * 0x5A = Z

## 4. PERFORMANCE NOTES
* Higher PageSize: Slower initial load, fewer frequent loads.
* Higher MaxItemsBeforeCleanup: Higher memory usage, fewer reloads.
* Wider WindowWidth: Better visualization of long content.
* Lower ScrollLoadThreshold: Smoother pre-loading (anticipation), more service calls.

## 5. RELATION WITH THUMBNAILCONFIG
UIConfig complements ThumbnailConfig (from the Core project):
* ThumbnailConfig: Controls generation and quality of thumbnails.
* UIConfig: Controls interface behavior and appearance.
* Both can be configured independently in App.xaml.cs.

## 6. IMPLEMENTATION EXAMPLES

### Custom Configuration in App.xaml.cs (OnLaunched)
// Custom UI Configuration
UIConfig.PageSize = 30;                 // Load more items
UIConfig.WindowWidth = 450;             // Wider window
UIConfig.Hotkey.UseWinKey = false;     // Use Ctrl instead of Win
UIConfig.Hotkey.VirtualKey = 0x43;     // Change to C key (Ctrl + Alt + C)

_window = new MainWindow(_service!);
_window.Activate();

### Disabling the Shortcut
To disable the keyboard shortcut, comment out these lines in MainWindow.xaml.cs:
// RegisterGlobalHotkey();
// HotkeyHelper.RegisterMessageHandler(this, OnHotkeyPressed);

## 7. MAINTENANCE AND TROUBLESHOOTING
* Hotkey: Works globally across the system. If Win registration fails, it auto-tries Ctrl.
* Optimization: For lower RAM usage, decrease PageSize and MaxItemsBeforeCleanup.
