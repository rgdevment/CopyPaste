using SkiaSharp;
using System.Diagnostics;
using System.Security.Cryptography;
using System.Text.Json;

namespace CopyPaste.Core;

public class ClipboardService(IClipboardRepository repository)
{
    public void AddItem(ClipboardItem item, byte[]? rawData = null)
    {
        ArgumentNullException.ThrowIfNull(item);

        // Pre-calculate hash for images to ensure reliable deduplication
        string? currentHash = null;
        if (item.Type == ClipboardContentType.Image && rawData != null)
        {
            currentHash = BitConverter.ToString(SHA256.HashData(rawData));
        }

        var latest = repository.GetLatest();

        if (IsDuplicate(latest, item, currentHash))
        {
            latest!.CreatedAt = DateTime.Now;
            repository.Update(latest);
            return;
        }

        if (item.Id == Guid.Empty) item.Id = Guid.NewGuid();

        // Store hash immediately in metadata to block future duplicates during async processing
        if (item.Type == ClipboardContentType.Image && currentHash != null)
        {
            var initialMeta = new Dictionary<string, object> { { "hash", currentHash } };
            item.Metadata = JsonSerializer.Serialize(initialMeta, MetadataJsonContext.Default.DictionaryStringObject);
        }

        repository.Save(item);

        if (item.Type == ClipboardContentType.Image && rawData != null)
        {
            _ = Task.Run(() => ProcessImageAssetsBackground(item, rawData, currentHash));
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

            // High-quality "Retina" resize settings
            int targetWidth = 300;
            int targetHeight = (int)(bitmap.Height * (targetWidth / (double)bitmap.Width));
            var sampling = new SKSamplingOptions(SKCubicResampler.CatmullRom);

            using var resized = new SKBitmap(targetWidth, targetHeight);
            using (var canvas = new SKCanvas(resized))
            {
                canvas.Clear(SKColors.Transparent);

                // DrawImage is required to use SKSamplingOptions
                using var imageToDraw = SKImage.FromBitmap(bitmap);
                using var paint = new SKPaint { IsAntialias = true };

                canvas.DrawImage(imageToDraw, SKRect.Create(targetWidth, targetHeight), sampling, paint);
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
            { "thumb_width", targetWidth },
            { "thumb_height", targetHeight },
            { "width", bitmap.Width },
            { "height", bitmap.Height },
            { "size", (long)rawData.Length },
            { "hash", preCalculatedHash ?? string.Empty }
        };

            // Serialize and finalize DB record
            item.Metadata = JsonSerializer.Serialize(dataMap, MetadataJsonContext.Default.DictionaryStringObject);
            repository.Update(item);
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or ArgumentException)
        {
            Debug.WriteLine($"Asset processing failed: {ex.Message}");
        }
    }

    private static bool IsDuplicate(ClipboardItem? last, ClipboardItem current, string? currentHash)
    {
        if (last == null || last.Type != current.Type) return false;

        return current.Type switch
        {
            ClipboardContentType.Text or
            ClipboardContentType.Html or
            ClipboardContentType.File => last.Content == current.Content,

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

    public IEnumerable<ClipboardItem> GetHistory(string? query = null) =>
        string.IsNullOrWhiteSpace(query) ? repository.GetAll() : repository.Search(query);

    public void RemoveItem(Guid id) => repository.Delete(id);
}
