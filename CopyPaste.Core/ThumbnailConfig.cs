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
    /// Optimized for 85px height container.
    /// </summary>
    public static int Width { get; set; } = 170;

    /// <summary>
    /// PNG encoding quality for image thumbnails (0-100).
    /// Higher values = better quality but larger files.
    /// </summary>
    public static int QualityPng { get; set; } = 80;

    /// <summary>
    /// JPEG encoding quality for video/media thumbnails (0-100).
    /// Higher values = better quality but larger files.
    /// </summary>
    public static int QualityJpeg { get; set; } = 80;

    /// <summary>
    /// Image size threshold (in bytes) to trigger garbage collection after processing.
    /// Forces memory cleanup for large images to prevent memory buildup.
    /// </summary>
    public static int GarbageCollectionThreshold { get; set; } = 1_000_000; // 1MB

    /// <summary>
    /// Decode pixel height for displaying thumbnails in UI.
    /// Matches container height (85px) with small buffer for quality.
    /// </summary>
    public static int UIDecodeHeight { get; set; } = 95;
}
