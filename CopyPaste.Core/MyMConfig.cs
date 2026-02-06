using System.Text.Json.Serialization;

namespace CopyPaste.Core;

/// <summary>
/// Unified configuration model for MyM.json file.
/// Contains ALL user-configurable settings for CopyPaste.
/// If MyM.json exists, these values take priority over defaults.
/// </summary>
public sealed class MyMConfig
{
    // ═══════════════════════════════════════════════════════════════
    // Localization Configuration
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Preferred language tag (BCP 47 format).
    /// "auto" = detect from Windows OS.
    /// "en-US", "es-CL", etc. = specific language.
    /// </summary>
    public string PreferredLanguage { get; set; } = "auto";

    // ═══════════════════════════════════════════════════════════════
    // Startup Configuration
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Whether to start CopyPaste automatically when Windows starts.
    /// </summary>
    public bool RunOnStartup { get; set; } = true;

    // ═══════════════════════════════════════════════════════════════
    // Theme Selection
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// ID of the active theme (e.g., "copypaste.default").
    /// Must match <see cref="CopyPaste.Core.Themes.ITheme.Id"/>.
    /// </summary>
    public string ThemeId { get; set; } = "copypaste.default";

    // ═══════════════════════════════════════════════════════════════
    // Hotkey Configuration
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Use Ctrl key as modifier.
    /// </summary>
    public bool UseCtrlKey { get; set; }

    /// <summary>
    /// Use Windows key as modifier.
    /// </summary>
    public bool UseWinKey { get; set; } = true;

    /// <summary>
    /// Use Alt key as modifier.
    /// </summary>
    public bool UseAltKey { get; set; } = true;

    /// <summary>
    /// Use Shift key as modifier.
    /// </summary>
    public bool UseShiftKey { get; set; }

    /// <summary>
    /// Virtual key code for the hotkey (V = 0x56).
    /// </summary>
    public uint VirtualKey { get; set; } = 0x56;

    /// <summary>
    /// Display name of the key (for UI).
    /// </summary>
    public string KeyName { get; set; } = "V";

    // ═══════════════════════════════════════════════════════════════
    // Pagination & Performance (applies to ALL themes)
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Number of clipboard items to load per page.
    /// </summary>
    public int PageSize { get; set; } = 20;

    /// <summary>
    /// Maximum number of items to keep in memory before cleanup.
    /// </summary>
    public int MaxItemsBeforeCleanup { get; set; } = 100;

    /// <summary>
    /// Scroll offset threshold (in pixels) from bottom to trigger loading more items.
    /// </summary>
    public int ScrollLoadThreshold { get; set; } = 100;



    // ═══════════════════════════════════════════════════════════════
    // Category Labels Configuration (applies to ALL themes)
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Custom labels for color categories. If null or empty, uses localized defaults.
    /// Keys: "Red", "Green", "Purple", "Yellow", "Blue", "Orange"
    /// Example: { "Red": "Urgent", "Green": "Personal", "Yellow": "Work" }
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Usage", "CA2227:Collection properties should be read only",
        Justification = "Setter required for JSON deserialization")]
    public Dictionary<string, string>? ColorLabels { get; set; }

    // ═══════════════════════════════════════════════════════════════
    // Storage Configuration
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Number of days to keep clipboard items before automatic cleanup.
    /// </summary>
    public int RetentionDays { get; set; } = 30;

    // ═══════════════════════════════════════════════════════════════
    // Paste Behavior Configuration
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Time window (ms) to ignore clipboard changes after app-initiated paste.
    /// Default: Seguro preset (450ms).
    /// </summary>
    public int DuplicateIgnoreWindowMs { get; set; } = 450;

    /// <summary>
    /// Delay (ms) before attempting to restore focus to previous window.
    /// Default: Seguro preset (100ms).
    /// </summary>
    public int DelayBeforeFocusMs { get; set; } = 100;

    /// <summary>
    /// Delay (ms) after restoring focus before simulating Ctrl+V.
    /// Default: Seguro preset (180ms).
    /// </summary>
    public int DelayBeforePasteMs { get; set; } = 180;

    /// <summary>
    /// Maximum attempts to verify focus was restored before pasting.
    /// Default: Seguro preset (15 attempts).
    /// </summary>
    public int MaxFocusVerifyAttempts { get; set; } = 15;


    // ═══════════════════════════════════════════════════════════════
    // Thumbnail Configuration (Advanced)
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Maximum width for generated thumbnails (height is calculated proportionally).
    /// </summary>
    public int ThumbnailWidth { get; set; } = 170;

    /// <summary>
    /// PNG encoding quality for image thumbnails (0-100).
    /// </summary>
    public int ThumbnailQualityPng { get; set; } = 80;

    /// <summary>
    /// JPEG encoding quality for video/media thumbnails (0-100).
    /// </summary>
    public int ThumbnailQualityJpeg { get; set; } = 80;

    /// <summary>
    /// Image size threshold (in bytes) to trigger garbage collection.
    /// </summary>
    public int ThumbnailGCThreshold { get; set; } = 1_000_000;

    /// <summary>
    /// Decode pixel height for displaying thumbnails in UI.
    /// </summary>
    public int ThumbnailUIDecodeHeight { get; set; } = 95;
}

/// <summary>
/// JSON serialization context for MyMConfig (required for trimming/AOT).
/// </summary>
[JsonSerializable(typeof(MyMConfig))]
[JsonSerializable(typeof(Dictionary<string, string>))]
[JsonSourceGenerationOptions(
    WriteIndented = true,
    PropertyNameCaseInsensitive = true,
    ReadCommentHandling = System.Text.Json.JsonCommentHandling.Skip,
    AllowTrailingCommas = true)]
public partial class MyMConfigJsonContext : JsonSerializerContext
{
}
