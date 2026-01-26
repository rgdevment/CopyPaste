namespace CopyPaste.Core;

/// <summary>
/// Centralized configuration for thumbnail generation and memory optimization.
/// Modify these values to balance between quality and memory consumption.
///
/// Warning: Changing these settings may impact application performance and memory usage.
/// </summary>
public static class ThumbnailConfig
{
    /// <summary>
    /// Maximum width for generated thumbnails (height is calculated proportionally).
    /// Lower values = less memory, faster processing, lower quality.
    /// Recommended: 120-180px for compact cards
    /// </summary>
    public static int Width { get; set; } = 160;

    /// <summary>
    /// PNG encoding quality for image thumbnails (0-100).
    /// Higher values = better quality but larger files.
    /// Recommended: 70-85 for compact thumbnails
    /// </summary>
    public static int QualityPng { get; set; } = 75;

    /// <summary>
    /// JPEG encoding quality for video/media thumbnails (0-100).
    /// Higher values = better quality but larger files.
    /// Recommended: 70-85 for compact thumbnails
    /// </summary>
    public static int QualityJpeg { get; set; } = 75;

    /// <summary>
    /// Image size threshold (in bytes) to trigger garbage collection after processing.
    /// Forces memory cleanup for large images to prevent memory buildup.
    /// Recommended: 500KB - 2MB
    /// </summary>
    public static int GarbageCollectionThreshold { get; set; } = 1_000_000; // 1MB

    /// <summary>
    /// Decode pixel height for displaying thumbnails in UI.
    /// Optimized for compact card display (MaxHeight ~80px).
    /// Recommended: 100-120px
    /// </summary>
    public static int UIDecodeHeight { get; set; } = 100;
}
