namespace CopyPaste.Core.Themes;

/// <summary>
/// Contract for a CopyPaste UI theme.
/// A theme replaces the entire main clipboard window while respecting host-level
/// controls (tray icon, hotkey, ConfigWindow, HelpWindow).
///
/// <b>Lifecycle:</b>
/// The Host calls <see cref="CreateWindow"/> once, then <see cref="Show"/>/<see cref="Hide"/>
/// on hotkey, and finally <see cref="IDisposable.Dispose"/> on app exit.
///
/// <b>Required UI behaviors (enforced by design, not by signature):</b>
/// <list type="bullet">
///   <item>Display clipboard items from <see cref="ThemeContext.Service"/>.GetHistoryAdvanced</item>
///   <item>Search: text filtering via service query</item>
///   <item>Category filtering: by <see cref="CardColor"/> and <see cref="ClipboardContentType"/></item>
///   <item>Tabs/sections: Recent and Pinned views (isPinned filter)</item>
///   <item>Paste: set clipboard + call MarkItemUsed + hide + simulate Ctrl+V</item>
///   <item>Delete: call Service.RemoveItem</item>
///   <item>Pin/Unpin: call Service.UpdatePin</item>
///   <item>Edit label/color: call Service.UpdateLabelAndColor</item>
///   <item>Pagination: lazy-load items as user scrolls (Config.PageSize)</item>
///   <item>Keyboard navigation: arrow keys, Enter=paste, Delete=remove, Esc=clear/hide</item>
///   <item>Real-time updates: subscribe to Service.OnItemAdded, OnThumbnailReady, OnItemReactivated</item>
///   <item>Shell access: call ThemeContext.OpenSettings/OpenHelp/RequestExit for host windows</item>
/// </list>
///
/// <b>Config values from <see cref="ThemeContext.Config"/> (shared across all themes):</b>
/// <list type="bullet">
///   <item>PageSize, ScrollLoadThreshold, MaxItemsBeforeCleanup — pagination</item>
///   <item>ThumbnailUIDecodeHeight — image rendering</item>
///   <item>DelayBeforeFocusMs, DelayBeforePasteMs, MaxFocusVerifyAttempts — paste timing</item>
///   <item>ColorLabels — custom color display names</item>
/// </list>
///
/// <b>Theme-specific settings (managed by each theme independently):</b>
/// <para>
/// Each theme defines its own settings class (e.g., DefaultThemeSettings) persisted
/// in its own JSON file (e.g., DefaultTheme.json). These are exposed to ConfigWindow
/// via <see cref="CreateSettingsSection"/>, <see cref="SaveThemeSettings"/>, and
/// <see cref="ResetThemeSettings"/>, displayed in a dedicated tab.
/// Examples: window geometry, card display lines, show-behavior reset flags.
/// </para>
/// </summary>
public interface ITheme : IDisposable
{
    /// <summary>Unique identifier (e.g., "copypaste.default").</summary>
    string Id { get; }

    /// <summary>Display name (e.g., "Default").</summary>
    string Name { get; }

    /// <summary>SemVer version (e.g., "1.0.0").</summary>
    string Version { get; }

    /// <summary>Theme author name.</summary>
    string Author { get; }

    /// <summary>
    /// Creates the theme window. Called once by the Host during startup.
    /// The theme must store the <paramref name="context"/> and wire up all
    /// service events, keyboard handlers, and config values.
    /// </summary>
    /// <returns>The native window handle (HWND) for hotkey message routing.</returns>
    IntPtr CreateWindow(ThemeContext context);

    /// <summary>Show the clipboard window and give it focus.</summary>
    void Show();

    /// <summary>
    /// Hide the clipboard window. The theme should collapse/reset transient UI state
    /// (expanded cards, selection) before hiding.
    /// </summary>
    void Hide();

    /// <summary>Toggle visibility. Called by the Host on global hotkey press.</summary>
    void Toggle();

    /// <summary>Whether the window is currently visible.</summary>
    bool IsVisible { get; }

    /// <summary>
    /// Creates the theme's settings UI section for embedding in ConfigWindow.
    /// Returns a UIElement (as object to avoid WinUI dependency in Core).
    /// Return null if the theme has no configurable settings.
    /// </summary>
    object? CreateSettingsSection();

    /// <summary>
    /// Saves theme-specific settings to persistent storage.
    /// Called by ConfigWindow when the user clicks Save.
    /// </summary>
    void SaveThemeSettings();

    /// <summary>
    /// Resets theme-specific settings to defaults and refreshes the settings UI.
    /// Called by ConfigWindow when the user clicks Reset.
    /// </summary>
    void ResetThemeSettings();
}
