using SkiaSharp;
using System.Collections.ObjectModel;
using System.Security.Cryptography;
using System.Text.Json;

namespace CopyPaste.Core;

[System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:No capture tipos de excepción generales.")]
public class ClipboardService(IClipboardRepository repository)
{
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1003: primitive object")]
    public event Action<ClipboardItem>? OnThumbnailReady;

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1003: primitive object")]
    public event Action<ClipboardItem>? OnItemAdded;

    /// <summary>
    /// Fired when a duplicate item is detected and its ModifiedAt is updated.
    /// The UI should move this item to the top of the list.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1003: primitive object")]
    public event Action<ClipboardItem>? OnItemReactivated;

    // Track when app initiates a paste to avoid re-adding the same content
    private DateTime _lastPasteTime = DateTime.MinValue;
    private Guid _lastPastedItemId = Guid.Empty;
    private string? _lastPastedContent;

    /// <summary>
    /// Configurable time window (in milliseconds) to ignore clipboard changes after paste.
    /// Default: 450ms (Seguro preset). Configured via MyMConfig.DuplicateIgnoreWindowMs.
    /// </summary>
    public int PasteIgnoreWindowMs { get; set; } = 450;

    /// <summary>
    /// Notifies the service that a paste operation was initiated by the app.
    /// This prevents the clipboard listener from re-adding the same item.
    /// </summary>
    public void NotifyPasteInitiated(Guid itemId)
    {
        _lastPasteTime = DateTime.UtcNow;
        _lastPastedItemId = itemId;

        // Store the content of the pasted item to prevent duplicates
        var item = repository.GetById(itemId);
        _lastPastedContent = item?.Content;
    }

    /// <summary>
    /// Checks if a clipboard change should be ignored because we just pasted it.
    /// Uses both time-based and content-based checks.
    /// </summary>
    private bool ShouldIgnoreClipboardChange(string? content = null)
    {
        if (_lastPastedItemId == Guid.Empty) return false;

        // Time-based check
        if (DateTime.UtcNow - _lastPasteTime < TimeSpan.FromMilliseconds(PasteIgnoreWindowMs))
            return true;

        // Content-based check - ignore if same content within extended window (2 seconds)
        if (content != null && _lastPastedContent == content &&
            DateTime.UtcNow - _lastPasteTime < TimeSpan.FromSeconds(2))
            return true;

        return false;
    }

    public void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null)
    {
        if (ShouldIgnoreClipboardChange(text)) return;


        string? json = null;
        if (rtfBytes != null)
        {
            var meta = new Dictionary<string, object> { { "rtf", Convert.ToBase64String(rtfBytes) } };
            json = JsonSerializer.Serialize(meta, MetadataJsonContext.Default.DictionaryStringObject);
        }


        AddItem(new ClipboardItem { Content = text ?? string.Empty, Type = type, AppSource = source, Metadata = json });
    }

    public void AddImage(byte[]? dibData, string? source)
    {
        if (ShouldIgnoreClipboardChange(null)) return;
        if (dibData == null) return;

        byte[]? bmp = ConvertDibToBmp(dibData);
        if (bmp != null)
        {
            AddItem(new ClipboardItem { Type = ClipboardContentType.Image, AppSource = source }, bmp);
        }
    }

    public void AddFiles(Collection<string>? files, ClipboardContentType type, string? source)
    {
        if (ShouldIgnoreClipboardChange(null)) return;
        if (files == null || files.Count == 0) return;

        string firstFile = files[0];
        bool isDirectory = Directory.Exists(firstFile);
        string paths = string.Join(Environment.NewLine, files);

        var meta = new Dictionary<string, object>
        {
            { "file_count", files.Count },
            { "file_name", Path.GetFileName(firstFile) },
            { "first_ext", Path.GetExtension(firstFile) },
            { "is_directory", isDirectory }
        };

        if (!isDirectory && File.Exists(firstFile))
        {
            var fileInfo = new FileInfo(firstFile);
            meta["file_size"] = fileInfo.Length;
        }

        string json = JsonSerializer.Serialize(meta, MetadataJsonContext.Default.DictionaryStringObject);
        var item = new ClipboardItem { Content = paths, Type = type, Metadata = json, AppSource = source };
        AddItem(item);

        if (type is ClipboardContentType.Video or ClipboardContentType.Audio && File.Exists(firstFile))
        {
            _ = Task.Run(() => ProcessMediaThumbnailBackground(item, firstFile, type));
        }
        else if (type is ClipboardContentType.Image && File.Exists(firstFile))
        {
            _ = Task.Run(() => ProcessImageFileBackground(item, firstFile));
        }
    }

    private void AddItem(ClipboardItem item, byte[]? imageData = null)
    {
        string? currentHash = null;
        if (item.Type == ClipboardContentType.Image && imageData != null)
        {
            currentHash = BitConverter.ToString(SHA256.HashData(imageData));
            item.ContentHash = currentHash; // Store hash in indexed column
        }

        // Check for duplicates - use indexed lookup for images
        ClipboardItem? existingItem = FindExistingItem(item, currentHash);

        if (existingItem != null)
        {
            // Item already exists - update timestamp and notify UI to move it to top
            existingItem.ModifiedAt = DateTime.UtcNow;
            repository.Update(existingItem);
            OnItemReactivated?.Invoke(existingItem);
            return;
        }

        if (item.Id == Guid.Empty) item.Id = Guid.NewGuid();

        if (item.Type == ClipboardContentType.Image && currentHash != null)
        {
            var initialMeta = new Dictionary<string, object> { { "hash", currentHash } };
            item.Metadata = JsonSerializer.Serialize(initialMeta, MetadataJsonContext.Default.DictionaryStringObject);
        }

        repository.Save(item);
        OnItemAdded?.Invoke(item);

        if (item.Type == ClipboardContentType.Image && imageData != null)
        {
            _ = Task.Run(() => ProcessImageAssetsBackground(item, imageData, currentHash));
        }
    }

    private void ProcessImageAssetsBackground(ClipboardItem item, byte[] rawData, string? preCalculatedHash)
    {
        try
        {
            string originalPath = Path.Combine(StorageConfig.ImagesPath, $"{item.Id}.png");
            File.WriteAllBytes(originalPath, rawData);

            item.Content = originalPath;
            repository.Update(item);

            string thumbPath = Path.Combine(StorageConfig.ThumbnailsPath, $"{item.Id}_t.png");

            using var managedSrc = new MemoryStream(rawData);
            using var bitmap = SKBitmap.Decode(managedSrc) ?? throw new ArgumentException("Decode failed");

            var (width, height) = GenerateThumbnail(bitmap, thumbPath, SKEncodedImageFormat.Png);

            var dataMap = new Dictionary<string, object>
            {
                { "thumb_path", thumbPath },
                { "thumb_width", width },
                { "thumb_height", height },
                { "width", bitmap.Width },
                { "height", bitmap.Height },
                { "size", (long)rawData.Length },
                { "hash", preCalculatedHash ?? string.Empty }
            };

            item.Metadata = JsonSerializer.Serialize(dataMap, MetadataJsonContext.Default.DictionaryStringObject);
            repository.Update(item);

            OnThumbnailReady?.Invoke(item);

            // Force GC for large images to prevent memory buildup
            if (rawData.Length >= ConfigLoader.Config.ThumbnailGCThreshold)
            {
                GC.Collect(1, GCCollectionMode.Optimized, blocking: false);
            }
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or ArgumentException)
        {
            AppLogger.Exception(ex, "Asset processing failed");
        }
    }

    private void ProcessImageFileBackground(ClipboardItem item, string filePath)
    {
        var meta = ParseExistingMetadata(item.Metadata);

        try
        {
            byte[] rawData = File.ReadAllBytes(filePath);
            string hash = BitConverter.ToString(SHA256.HashData(rawData));
            meta["hash"] = hash;

            string originalPath = Path.Combine(StorageConfig.ImagesPath, $"{item.Id}.png");
            File.WriteAllBytes(originalPath, rawData);

            item.Content = originalPath;

            string thumbPath = Path.Combine(StorageConfig.ThumbnailsPath, $"{item.Id}_t.png");

            using var managedSrc = new MemoryStream(rawData);
            using var bitmap = SKBitmap.Decode(managedSrc);

            if (bitmap != null)
            {
                var (width, height) = GenerateThumbnail(bitmap, thumbPath, SKEncodedImageFormat.Png);

                meta["thumb_path"] = thumbPath;
                meta["thumb_width"] = width;
                meta["thumb_height"] = height;
                meta["width"] = bitmap.Width;
                meta["height"] = bitmap.Height;
                meta["size"] = (long)rawData.Length;

                // Force GC for large images to prevent memory buildup
                if (rawData.Length >= ConfigLoader.Config.ThumbnailGCThreshold)
                {
                    GC.Collect(1, GCCollectionMode.Optimized, blocking: false);
                }
            }
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or ArgumentException)
        {
            AppLogger.Exception(ex, "Image file processing failed");
        }
        finally
        {
            item.Metadata = JsonSerializer.Serialize(meta, MetadataJsonContext.Default.DictionaryStringObject);
            repository.Update(item);
            OnThumbnailReady?.Invoke(item);
        }
    }

    private void ProcessMediaThumbnailBackground(ClipboardItem item, string filePath, ClipboardContentType type)
    {
        var meta = ParseExistingMetadata(item.Metadata);

        try
        {
            byte[]? thumbData = null;

            // Extract thumbnail based on type
            try
            {
                if (type == ClipboardContentType.Video)
                {
                    AppLogger.Info($"Extracting video thumbnail: {Path.GetFileName(filePath)}");
                    thumbData = WindowsThumbnailExtractor.GetThumbnail(filePath, ConfigLoader.Config.ThumbnailWidth);
                    if (thumbData != null)
                    {
                        AppLogger.Info($"Video thumbnail extracted successfully: {thumbData.Length} bytes");
                    }
                    else
                    {
                        AppLogger.Warn($"Video thumbnail extraction returned null for: {Path.GetFileName(filePath)}");
                    }
                }
                else if (type == ClipboardContentType.Audio)
                {
                    AppLogger.Info($"Extracting audio artwork: {Path.GetFileName(filePath)}");
                    thumbData = ExtractAudioArtwork(filePath);
                    if (thumbData != null)
                    {
                        AppLogger.Info($"Audio artwork extracted successfully: {thumbData.Length} bytes");
                    }
                    else
                    {
                        AppLogger.Warn($"Audio artwork extraction returned null for: {Path.GetFileName(filePath)}");
                    }
                }
            }
            catch (Exception ex)
            {
                AppLogger.Exception(ex, $"Thumbnail extraction failed for: {Path.GetFileName(filePath)}");
            }

            // Save thumbnail if we got one
            if (thumbData != null && thumbData.Length > 0)
            {
                try
                {
                    // Windows already returns the thumbnail in the requested size as JPEG
                    // For videos: save directly without re-encoding (more efficient, preserves quality)
                    // For audio: artwork might be PNG/JPEG, detect and save accordingly
                    string extension = type == ClipboardContentType.Video ? "jpg" : DetectImageFormat(thumbData);
                    string thumbPath = Path.Combine(StorageConfig.ThumbnailsPath, $"{item.Id}_t.{extension}");

                    AppLogger.Info($"Saving {type} thumbnail directly: {thumbData.Length} bytes as {extension}");
                    File.WriteAllBytes(thumbPath, thumbData);

                    // Get dimensions for metadata
                    using var managedSrc = new MemoryStream(thumbData);
                    using var bitmap = SKBitmap.Decode(managedSrc);

                    if (bitmap != null)
                    {
                        meta["thumb_path"] = thumbPath;
                        meta["thumb_width"] = bitmap.Width;
                        meta["thumb_height"] = bitmap.Height;
                        meta["original_width"] = bitmap.Width;
                        meta["original_height"] = bitmap.Height;

                        AppLogger.Info($"Thumbnail saved successfully: {thumbPath} ({bitmap.Width}x{bitmap.Height})");
                    }
                    else
                    {
                        AppLogger.Warn($"Could not decode thumbnail for dimensions, but file was saved");
                        // Still save the path even if we couldn't decode dimensions
                        meta["thumb_path"] = thumbPath;
                    }
                }
                catch (Exception ex)
                {
                    AppLogger.Exception(ex, $"Thumbnail save failed for item {item.Id}");
                }
            }
            else
            {
                AppLogger.Warn($"No thumbnail data available for {type}: {Path.GetFileName(filePath)}");
            }

            // Extract metadata (duration, etc.) - always try even if thumbnail failed
            ExtractMediaMetadata(filePath, type, meta);
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Media processing failed");
        }
        finally
        {
            // ALWAYS update and notify - even if thumbnail failed, metadata may have been extracted
            try
            {
                item.Metadata = JsonSerializer.Serialize(meta, MetadataJsonContext.Default.DictionaryStringObject);
                repository.Update(item);
            }
            catch (Exception ex)
            {
                AppLogger.Exception(ex, "Failed to save metadata");
            }

            // ALWAYS notify UI so it can refresh (show placeholder or real thumb)
            OnThumbnailReady?.Invoke(item);
        }
    }

    private static Dictionary<string, object> ParseExistingMetadata(string? metadata)
    {
        var meta = new Dictionary<string, object>();
        if (string.IsNullOrEmpty(metadata)) return meta;

        try
        {
            using var doc = JsonDocument.Parse(metadata);
            foreach (var prop in doc.RootElement.EnumerateObject())
            {
                meta[prop.Name] = prop.Value.ValueKind switch
                {
                    JsonValueKind.String => prop.Value.GetString()!,
                    JsonValueKind.Number => prop.Value.GetInt64(),
                    _ => prop.Value.ToString()
                };
            }
        }
        catch (JsonException) { }

        return meta;
    }

    private static void ExtractMediaMetadata(string filePath, ClipboardContentType type, Dictionary<string, object> meta)
    {
        try
        {
            // Use TagLib for both audio and video metadata - it's fast and doesn't need FFmpeg
            using var tagFile = TagLib.File.Create(filePath);

            if (tagFile.Properties.Duration != TimeSpan.Zero)
                meta["duration"] = (long)tagFile.Properties.Duration.TotalSeconds;

            if (type == ClipboardContentType.Video)
            {
                if (tagFile.Properties.VideoWidth > 0)
                    meta["video_width"] = tagFile.Properties.VideoWidth;
                if (tagFile.Properties.VideoHeight > 0)
                    meta["video_height"] = tagFile.Properties.VideoHeight;
            }
            else if (type == ClipboardContentType.Audio)
            {
                if (!string.IsNullOrEmpty(tagFile.Tag.FirstAlbumArtist))
                    meta["artist"] = tagFile.Tag.FirstAlbumArtist;
                if (!string.IsNullOrEmpty(tagFile.Tag.Title))
                    meta["title"] = tagFile.Tag.Title;
                if (!string.IsNullOrEmpty(tagFile.Tag.Album))
                    meta["album"] = tagFile.Tag.Album;
            }
        }
        catch { /* Ignore metadata extraction failures */ }
    }

    private static string DetectImageFormat(byte[] imageData)
    {
        if (imageData.Length < 4) return "jpg"; // Default fallback

        // Check PNG signature (89 50 4E 47)
        if (imageData[0] == 0x89 && imageData[1] == 0x50 && imageData[2] == 0x4E && imageData[3] == 0x47)
            return "png";

        // Check JPEG signature (FF D8 FF)
        if (imageData[0] == 0xFF && imageData[1] == 0xD8 && imageData[2] == 0xFF)
            return "jpg";

        // Check WebP signature (RIFF ... WEBP)
        if (imageData.Length >= 12 &&
            imageData[0] == 0x52 && imageData[1] == 0x49 && imageData[2] == 0x46 && imageData[3] == 0x46 &&
            imageData[8] == 0x57 && imageData[9] == 0x45 && imageData[10] == 0x42 && imageData[11] == 0x50)
            return "webp";

        return "jpg"; // Default fallback
    }

    private static byte[]? ExtractAudioArtwork(string audioPath)
    {
        try
        {
            using var tagFile = TagLib.File.Create(audioPath);
            var pictures = tagFile.Tag.Pictures;

            if (pictures.Length == 0)
            {
                AppLogger.Info($"[ExtractAudioArtwork] No pictures found in {Path.GetFileName(audioPath)} (pictures.Length={pictures.Length})");
                return null;
            }

            var firstPicture = pictures[0];
            var data = firstPicture.Data?.Data;

            if (data == null || data.Length == 0)
            {
                AppLogger.Warn($"[ExtractAudioArtwork] Picture data is null or empty for {Path.GetFileName(audioPath)}");
                return null;
            }

            AppLogger.Info($"[ExtractAudioArtwork] Found picture with {data.Length} bytes, type={firstPicture.Type}");
            return data;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, $"[ExtractAudioArtwork] Failed to extract from {Path.GetFileName(audioPath)}");
            return null;
        }
    }

    private ClipboardItem? FindExistingItem(ClipboardItem current, string? currentHash)
    {
        // For images, we need to search by hash in metadata
        if (current.Type == ClipboardContentType.Image && currentHash != null)
        {
            return FindExistingImageByHash(currentHash);
        }

        // For non-image types, search by content and type
        return repository.FindByContentAndType(current.Content, current.Type);
    }

    // Use indexed ContentHash column for O(1) lookup instead of scanning all images
    private ClipboardItem? FindExistingImageByHash(string currentHash) =>
        repository.FindByContentHash(currentHash);

    public IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? query = null, bool? isPinned = null) =>
        GetHistoryAdvanced(limit, skip, query, null, null, isPinned);

    /// <summary>
    /// Advanced history retrieval with all filters at DB level.
    /// When any filter is active (query, types, or colors), searches ALL items (PRO mode).
    /// </summary>
    public IEnumerable<ClipboardItem> GetHistoryAdvanced(
        int limit,
        int skip,
        string? query,
        IReadOnlyCollection<ClipboardContentType>? types,
        IReadOnlyCollection<CardColor>? colors,
        bool? isPinned) =>
        repository.SearchAdvanced(query, types, colors, isPinned, limit, skip);

    public void RemoveItem(Guid id)
    {
        var item = repository.GetById(id);
        if (item == null) return;

        // The repository.Delete() method handles cleanup of app-generated files
        // (backup images and thumbnails stored in LocalAppData)
        // It NEVER deletes the original files
        repository.Delete(id);
    }

    public void UpdatePin(Guid id, bool isPinned)
    {
        var item = repository.GetById(id);
        if (item == null) return;

        item.IsPinned = isPinned;
        item.ModifiedAt = DateTime.UtcNow;
        repository.Update(item);
    }

    public void UpdateLabelAndColor(Guid id, string? label, CardColor color)
    {
        var item = repository.GetById(id);
        if (item == null) return;

        item.Label = label;
        item.CardColor = color;
        item.ModifiedAt = DateTime.UtcNow;
        repository.Update(item);
    }

    /// <summary>
    /// Marks an item as used by updating its ModifiedAt timestamp and incrementing PasteCount.
    /// This moves the item to the top of the list when sorted by modification date.
    /// </summary>
    /// <param name="id">The ID of the item to mark as used.</param>
    /// <returns>The updated item, or null if not found.</returns>
    public ClipboardItem? MarkItemUsed(Guid id)
    {
        var item = repository.GetById(id);
        if (item == null) return null;

        item.ModifiedAt = DateTime.UtcNow;
        item.PasteCount++;

        _ = Task.Run(() => repository.Update(item));
        return item;
    }

    /// <summary>
    /// Generates a thumbnail from a bitmap and saves it to disk.
    /// Returns the width and height of the generated thumbnail.
    /// </summary>
    private static (int Width, int Height) GenerateThumbnail(SKBitmap sourceBitmap, string outputPath, SKEncodedImageFormat format)
    {
        var config = ConfigLoader.Config;
        int targetHeight = (int)(sourceBitmap.Height * (config.ThumbnailWidth / (double)sourceBitmap.Width));
        var sampling = new SKSamplingOptions(SKCubicResampler.CatmullRom);

        using var resized = new SKBitmap(config.ThumbnailWidth, targetHeight);
        using (var canvas = new SKCanvas(resized))
        {
            canvas.Clear(SKColors.Transparent);
            using var imageToDraw = SKImage.FromBitmap(sourceBitmap);
            using var paint = new SKPaint { IsAntialias = true };
            canvas.DrawImage(imageToDraw, SKRect.Create(config.ThumbnailWidth, targetHeight), sampling, paint);
        }

        using var thumbImage = SKImage.FromBitmap(resized);
        int quality = format == SKEncodedImageFormat.Png ? config.ThumbnailQualityPng : config.ThumbnailQualityJpeg;
        using var data = thumbImage.Encode(format, quality);

        using (var stream = File.Create(outputPath))
        {
            data.SaveTo(stream);
        }

        return (config.ThumbnailWidth, targetHeight);
    }

    private static byte[]? ConvertDibToBmp(byte[] dibData)
    {
        if (dibData.Length < 40) return null;

        int headerSize = BitConverter.ToInt32(dibData, 0);
        int bitCount = BitConverter.ToInt16(dibData, 14);
        int compression = BitConverter.ToInt32(dibData, 16);
        int colorsUsed = BitConverter.ToInt32(dibData, 32);

        int paletteSize = 0;
        if (headerSize == 40 && compression == 3) paletteSize = 12;
        else if (colorsUsed > 0) paletteSize = colorsUsed * 4;
        else if (bitCount <= 8) paletteSize = (1 << bitCount) * 4;

        int pixelOffset = 14 + headerSize + paletteSize;
        int fileSize = 14 + dibData.Length;

        byte[] fileHeader = new byte[14];
        fileHeader[0] = 0x42; // 'B'
        fileHeader[1] = 0x4D; // 'M'
        BitConverter.TryWriteBytes(fileHeader.AsSpan(2), fileSize);
        BitConverter.TryWriteBytes(fileHeader.AsSpan(10), pixelOffset);

        byte[] bmp = new byte[fileSize];
        Buffer.BlockCopy(fileHeader, 0, bmp, 0, 14);
        Buffer.BlockCopy(dibData, 0, bmp, 14, dibData.Length);

        return bmp;
    }
}
