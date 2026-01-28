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
    // Startup Configuration
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Whether to start CopyPaste automatically when Windows starts.
    /// </summary>
    public bool RunOnStartup { get; set; } = true;

    // ═══════════════════════════════════════════════════════════════
    // Hotkey Configuration
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Use Ctrl key as modifier.
    /// </summary>
    public bool UseCtrlKey { get; set; } = false;

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
    public bool UseShiftKey { get; set; } = false;

    /// <summary>
    /// Virtual key code for the hotkey (V = 0x56).
    /// </summary>
    public uint VirtualKey { get; set; } = 0x56;

    /// <summary>
    /// Display name of the key (for UI).
    /// </summary>
    public string KeyName { get; set; } = "V";

    // ═══════════════════════════════════════════════════════════════
    // UI Configuration
    // ═══════════════════════════════════════════════════════════════

    /// <summary>
    /// Width of the sidebar window in pixels.
    /// </summary>
    public int WindowWidth { get; set; } = 400;

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

    /// <summary>
    /// Vertical margin from top of work area in pixels.
    /// </summary>
    public int WindowMarginTop { get; set; } = 8;

    /// <summary>
    /// Vertical margin from bottom of work area in pixels.
    /// </summary>
    public int WindowMarginBottom { get; set; } = 16;

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
    /// </summary>
    public int DuplicateIgnoreWindowMs { get; set; } = 300;

    /// <summary>
    /// Delay (ms) before attempting to restore focus to previous window.
    /// </summary>
    public int DelayBeforeFocusMs { get; set; } = 50;

    /// <summary>
    /// Delay (ms) after restoring focus before simulating Ctrl+V.
    /// </summary>
    public int DelayBeforePasteMs { get; set; } = 100;

    /// <summary>
    /// Maximum attempts to verify focus was restored before pasting.
    /// </summary>
    public int MaxFocusVerifyAttempts { get; set; } = 10;

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
[JsonSourceGenerationOptions(
    WriteIndented = true,
    PropertyNameCaseInsensitive = true,
    ReadCommentHandling = System.Text.Json.JsonCommentHandling.Skip,
    AllowTrailingCommas = true)]
public partial class MyMConfigJsonContext : JsonSerializerContext
{
}
