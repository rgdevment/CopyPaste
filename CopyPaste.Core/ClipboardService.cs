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

    /// <summary>
    /// Configurable time window (in milliseconds) to ignore clipboard changes after paste.
    /// Default: 300ms. Can be adjusted based on system performance.
    /// </summary>
    public int PasteIgnoreWindowMs { get; set; } = 300;

    /// <summary>
    /// Notifies the service that a paste operation was initiated by the app.
    /// This prevents the clipboard listener from re-adding the same item.
    /// </summary>
    public void NotifyPasteInitiated(Guid itemId)
    {
        _lastPasteTime = DateTime.UtcNow;
        _lastPastedItemId = itemId;
    }

    /// <summary>
    /// Checks if a clipboard change should be ignored because we just pasted it.
    /// </summary>
    private bool ShouldIgnoreClipboardChange()
    {
        if (_lastPastedItemId == Guid.Empty) return false;
        return DateTime.UtcNow - _lastPasteTime < TimeSpan.FromMilliseconds(PasteIgnoreWindowMs);
    }

    public void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null)
    {
        if (ShouldIgnoreClipboardChange()) return;


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
        if (ShouldIgnoreClipboardChange()) return;
        if (dibData == null) return;

        byte[]? bmp = ConvertDibToBmp(dibData);
        if (bmp != null)
        {
            AddItem(new ClipboardItem { Type = ClipboardContentType.Image, AppSource = source }, bmp);
        }
    }

    public void AddFiles(Collection<string>? files, ClipboardContentType type, string? source)
    {
        if (ShouldIgnoreClipboardChange()) return;
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
        }

        var latest = repository.GetLatest();

        if (IsDuplicate(latest, item, currentHash))
        {
            // Item already exists - update timestamp and notify UI to move it to top
            latest!.ModifiedAt = DateTime.UtcNow;
            repository.Update(latest);
            OnItemReactivated?.Invoke(latest);
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
            if (rawData.Length >= ThumbnailConfig.GarbageCollectionThreshold)
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
                if (rawData.Length >= ThumbnailConfig.GarbageCollectionThreshold)
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
                    thumbData = WindowsThumbnailExtractor.GetThumbnail(filePath, ThumbnailConfig.Width);
                }
                else if (type == ClipboardContentType.Audio)
                {
                    thumbData = ExtractAudioArtwork(filePath);
                }
            }
            catch (Exception ex)
            {
                AppLogger.Exception(ex, "Thumbnail extraction failed");
            }

            // Save thumbnail if we got one
            if (thumbData != null && thumbData.Length > 0)
            {
                try
                {
                    string thumbPath = Path.Combine(StorageConfig.ThumbnailsPath, $"{item.Id}_t.png");

                    using var managedSrc = new MemoryStream(thumbData);
                    using var bitmap = SKBitmap.Decode(managedSrc);

                    if (bitmap != null)
                    {
                        var (width, height) = GenerateThumbnail(bitmap, thumbPath, SKEncodedImageFormat.Png);

                        meta["thumb_path"] = thumbPath;
                        meta["thumb_width"] = width;
                        meta["thumb_height"] = height;
                        meta["original_width"] = bitmap.Width;
                        meta["original_height"] = bitmap.Height;
                    }
                }
                catch (Exception ex)
                {
                    AppLogger.Exception(ex, "Thumbnail save failed");
                }
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

    private static byte[]? ExtractAudioArtwork(string audioPath)
    {
        try
        {
            using var tagFile = TagLib.File.Create(audioPath);
            var pictures = tagFile.Tag.Pictures;
            return pictures.Length > 0 ? pictures[0].Data.Data : null;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Audio artwork extraction failed");
            return null;
        }
    }

    private static bool IsDuplicate(ClipboardItem? last, ClipboardItem current, string? currentHash)
    {
        if (last == null || last.Type != current.Type) return false;

        return current.Type switch
        {
            ClipboardContentType.Text or
            ClipboardContentType.Link or
            ClipboardContentType.File or
            ClipboardContentType.Folder or
            ClipboardContentType.Audio or
            ClipboardContentType.Video => last.Content == current.Content,

            ClipboardContentType.Image => CompareImageHashes(last, currentHash),

            _ => false
        };
    }

    private static bool CompareImageHashes(ClipboardItem last, string? currentHash)
    {
        if (currentHash == null || string.IsNullOrEmpty(last.Metadata)) return false;

        try
        {
            using var doc = JsonDocument.Parse(last.Metadata);
            if (doc.RootElement.TryGetProperty("hash", out var hashProp))
            {
                return hashProp.GetString() == currentHash;
            }
        }
        catch (JsonException) { }

        return false;
    }

    public IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? query = null, bool? isPinned = null)
    {
        var items = string.IsNullOrWhiteSpace(query)
            ? repository.GetAll()
            : repository.Search(query, limit * 10, 0);

        if (isPinned.HasValue)
        {
            items = items.Where(x => x.IsPinned == isPinned.Value);
        }

        return items
            .OrderByDescending(x => x.ModifiedAt)
            .Skip(skip)
            .Take(limit);
    }

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

    /// <summary>
    /// Marks an item as used by updating its ModifiedAt timestamp.
    /// This moves the item to the top of the list when sorted by modification date.
    /// </summary>
    /// <param name="id">The ID of the item to mark as used.</param>
    /// <returns>The updated item, or null if not found.</returns>
    public ClipboardItem? MarkItemUsed(Guid id)
    {
        var item = repository.GetById(id);
        if (item == null) return null;

        item.ModifiedAt = DateTime.UtcNow;
        repository.Update(item);
        return item;
    }

    /// <summary>
    /// Generates a thumbnail from a bitmap and saves it to disk.
    /// Returns the width and height of the generated thumbnail.
    /// </summary>
    private static (int Width, int Height) GenerateThumbnail(SKBitmap sourceBitmap, string outputPath, SKEncodedImageFormat format)
    {
        int targetHeight = (int)(sourceBitmap.Height * (ThumbnailConfig.Width / (double)sourceBitmap.Width));
        var sampling = new SKSamplingOptions(SKCubicResampler.CatmullRom);

        using var resized = new SKBitmap(ThumbnailConfig.Width, targetHeight);
        using (var canvas = new SKCanvas(resized))
        {
            canvas.Clear(SKColors.Transparent);
            using var imageToDraw = SKImage.FromBitmap(sourceBitmap);
            using var paint = new SKPaint { IsAntialias = true };
            canvas.DrawImage(imageToDraw, SKRect.Create(ThumbnailConfig.Width, targetHeight), sampling, paint);
        }

        using var thumbImage = SKImage.FromBitmap(resized);
        int quality = format == SKEncodedImageFormat.Png ? ThumbnailConfig.QualityPng : ThumbnailConfig.QualityJpeg;
        using var data = thumbImage.Encode(format, quality);

        using (var stream = File.Create(outputPath))
        {
            data.SaveTo(stream);
        }

        return (ThumbnailConfig.Width, targetHeight);
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
