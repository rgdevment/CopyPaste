using SkiaSharp;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Security.Cryptography;
using System.Text.Json;

namespace CopyPaste.Core;

[System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:No capture tipos de excepción generales.")]
public class ClipboardService(IClipboardRepository repository)
{
    private const int _thumbnailWidth = 300;

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1003: primitive object")]
    public event Action<ClipboardItem>? OnThumbnailReady;

    public void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null)
    {
        if (string.IsNullOrWhiteSpace(text)) return;

        string? json = null;
        if (rtfBytes != null)
        {
            var meta = new Dictionary<string, object> { { "rtf", Convert.ToBase64String(rtfBytes) } };
            json = JsonSerializer.Serialize(meta, MetadataJsonContext.Default.DictionaryStringObject);
        }

        AddItem(new ClipboardItem { Content = text, Type = type, AppSource = source, Metadata = json });
    }

    public void AddImage(byte[]? dibData, string? source)
    {
        if (dibData == null) return;

        byte[]? bmp = ConvertDibToBmp(dibData);
        if (bmp != null)
        {
            AddItem(new ClipboardItem { Type = ClipboardContentType.Image, AppSource = source }, bmp);
        }
    }

    public void AddFiles(Collection<string>? files, ClipboardContentType type, string? source)
    {
        if (files == null || files.Count == 0) return;

        string firstFile = files[0];
        string paths = string.Join(Environment.NewLine, files);

        var meta = new Dictionary<string, object>
        {
            { "file_count", files.Count },
            { "file_name", Path.GetFileName(firstFile) },
            { "first_ext", Path.GetExtension(firstFile) }
        };

        if (File.Exists(firstFile))
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
            latest!.CreatedAt = DateTime.Now;
            repository.Update(latest);
            return;
        }

        if (item.Id == Guid.Empty) item.Id = Guid.NewGuid();

        if (item.Type == ClipboardContentType.Image && currentHash != null)
        {
            var initialMeta = new Dictionary<string, object> { { "hash", currentHash } };
            item.Metadata = JsonSerializer.Serialize(initialMeta, MetadataJsonContext.Default.DictionaryStringObject);
        }

        repository.Save(item);

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

            int targetHeight = (int)(bitmap.Height * (_thumbnailWidth / (double)bitmap.Width));
            var sampling = new SKSamplingOptions(SKCubicResampler.CatmullRom);

            using var resized = new SKBitmap(_thumbnailWidth, targetHeight);
            using (var canvas = new SKCanvas(resized))
            {
                canvas.Clear(SKColors.Transparent);
                using var imageToDraw = SKImage.FromBitmap(bitmap);
                using var paint = new SKPaint { IsAntialias = true };
                canvas.DrawImage(imageToDraw, SKRect.Create(_thumbnailWidth, targetHeight), sampling, paint);
            }

            using var thumbImage = SKImage.FromBitmap(resized);
            using var data = thumbImage.Encode(SKEncodedImageFormat.Png, 90);

            using (var stream = File.Create(thumbPath))
            {
                data.SaveTo(stream);
            }

            var dataMap = new Dictionary<string, object>
            {
                { "thumb_path", thumbPath },
                { "thumb_width", _thumbnailWidth },
                { "thumb_height", targetHeight },
                { "width", bitmap.Width },
                { "height", bitmap.Height },
                { "size", (long)rawData.Length },
                { "hash", preCalculatedHash ?? string.Empty }
            };

            item.Metadata = JsonSerializer.Serialize(dataMap, MetadataJsonContext.Default.DictionaryStringObject);
            repository.Update(item);

            OnThumbnailReady?.Invoke(item);
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or ArgumentException)
        {
            Debug.WriteLine($"Asset processing failed: {ex.Message}");
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
                    thumbData = WindowsThumbnailExtractor.GetThumbnail(filePath, _thumbnailWidth);
                }
                else if (type == ClipboardContentType.Audio)
                {
                    thumbData = ExtractAudioArtwork(filePath);
                }
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"Thumbnail extraction failed: {ex.Message}");
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
                        int targetHeight = (int)(bitmap.Height * (_thumbnailWidth / (double)bitmap.Width));
                        var sampling = new SKSamplingOptions(SKCubicResampler.CatmullRom);

                        using var resized = new SKBitmap(_thumbnailWidth, targetHeight);
                        using (var canvas = new SKCanvas(resized))
                        {
                            canvas.Clear(SKColors.Transparent);
                            using var imageToDraw = SKImage.FromBitmap(bitmap);
                            using var paint = new SKPaint { IsAntialias = true };
                            canvas.DrawImage(imageToDraw, SKRect.Create(_thumbnailWidth, targetHeight), sampling, paint);
                        }

                        using var thumbImage = SKImage.FromBitmap(resized);
                        using var data = thumbImage.Encode(SKEncodedImageFormat.Png, 90);

                        using (var stream = File.Create(thumbPath))
                        {
                            data.SaveTo(stream);
                        }

                        meta["thumb_path"] = thumbPath;
                        meta["thumb_width"] = _thumbnailWidth;
                        meta["thumb_height"] = targetHeight;
                        meta["original_width"] = bitmap.Width;
                        meta["original_height"] = bitmap.Height;
                    }
                }
                catch (Exception ex)
                {
                    Debug.WriteLine($"Thumbnail save failed: {ex.Message}");
                }
            }

            // Extract metadata (duration, etc.) - always try even if thumbnail failed
            ExtractMediaMetadata(filePath, type, meta);
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Media processing failed: {ex.Message}");
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
                Debug.WriteLine($"Failed to save metadata: {ex.Message}");
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
            Debug.WriteLine($"Audio artwork extraction failed: {ex.Message}");
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

    public IEnumerable<ClipboardItem> GetHistory(int limit = 50, string? query = null)
    {
        var items = string.IsNullOrWhiteSpace(query)
            ? repository.GetAll()
            : repository.Search(query);

        return items
            .OrderByDescending(x => x.CreatedAt)
            .Take(limit);
    }

    public void RemoveItem(Guid id)
    {
        var item = repository.GetAll().FirstOrDefault(x => x.Id == id);
        if (item == null) return;

        repository.Delete(id);

        _ = Task.Run(() =>
        {
            try
            {
                // Only delete original file for images (we store a copy)
                if (item.Type == ClipboardContentType.Image
                    && !string.IsNullOrEmpty(item.Content)
                    && File.Exists(item.Content))
                {
                    File.Delete(item.Content);
                }

                // Always delete generated thumbnails
                if (!string.IsNullOrEmpty(item.Metadata))
                {
                    using var doc = JsonDocument.Parse(item.Metadata);
                    if (doc.RootElement.TryGetProperty("thumb_path", out var pathProp))
                    {
                        string? thumbPath = pathProp.GetString();
                        if (!string.IsNullOrEmpty(thumbPath) && File.Exists(thumbPath))
                        {
                            File.Delete(thumbPath);
                        }
                    }
                }
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or ArgumentException or JsonException)
            {
                Debug.WriteLine($"Failed to delete physical files: {ex.Message}");
            }
        });
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
