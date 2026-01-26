namespace CopyPaste.UI;

/// <summary>
/// Centralized configuration for UI behavior and appearance.
/// Modify these values to customize the application experience.
/// </summary>
public static class UIConfig
{
    /// <summary>
    /// Number of clipboard items to load per page.
    /// Higher values = more initial load time but less frequent loading.
    /// Recommended: 15-30
    /// </summary>
    public static int PageSize { get; set; } = 20;

    /// <summary>
    /// Maximum number of items to keep in memory before cleanup when window is deactivated.
    /// Helps prevent memory buildup while maintaining recent history.
    /// Recommended: 50-150
    /// </summary>
    public static int MaxItemsBeforeCleanup { get; set; } = 100;

    /// <summary>
    /// Scroll offset threshold (in pixels) from bottom to trigger loading more items.
    /// Lower values = earlier loading, smoother experience.
    /// Recommended: 50-200
    /// </summary>
    public static int ScrollLoadThreshold { get; set; } = 100;

    /// <summary>
    /// Width of the sidebar window in pixels.
    /// Recommended: 350-500
    /// </summary>
    public static int WindowWidth { get; set; } = 400;

    /// <summary>
    /// Vertical margin from top of work area in pixels.
    /// </summary>
    public static int WindowMarginTop { get; set; } = 8;

    /// <summary>
    /// Vertical margin from bottom of work area in pixels.
    /// </summary>
    public static int WindowMarginBottom { get; set; } = 16;

    /// <summary>
    /// Number of days to keep clipboard items before automatic cleanup.
    /// Items older than this will be deleted (except pinned items).
    /// Set to 0 to disable automatic cleanup.
    /// Recommended: 7-90
    /// </summary>
    public static int RetentionDays { get; set; } = 30;
}

/// <summary>
/// Global hotkey to show/hide the window.
/// Modifiers: Win (default) or Ctrl (if Win fails to register)
/// Key: Alt + V
/// </summary>
public static class UIHotkey
{
    /// <summary>
    /// Virtual key code for the hotkey (V = 0x56)
    /// </summary>
    public static uint VirtualKey { get; set; } = 0x56; // V key

    /// <summary>
    /// Use Windows key as modifier (true) or Ctrl key (false)
    /// </summary>
    public static bool UseWinKey { get; set; } = true;

    /// <summary>
    /// Include Alt modifier (always true for this app)
    /// </summary>
    public static bool UseAltKey { get; set; } = true;
}
