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
            using var bitmap = SKBitmap.Decode(managedSrc) ?? throw new ArgumentException(
                    $"Failed to decode bitmap. Size: {rawData.Length} bytes");

            int width = 200;
            int height = (int)(bitmap.Height * (200.0 / bitmap.Width));

            var sampling = new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.None);
            using var resized = bitmap.Resize(new SKImageInfo(width, height), sampling);
            using var image = SKImage.FromBitmap(resized);
            using var data = image.Encode(SKEncodedImageFormat.Png, 80);

            using (var stream = File.Create(thumbPath))
            {
                data.SaveTo(stream);
            }

            var dataMap = new Dictionary<string, object>
        {
            { "thumb_path", thumbPath },
            { "thumb_width", width },
            { "thumb_height", height },
            { "width", bitmap.Width },
            { "height", bitmap.Height },
            { "size", rawData.Length },
            { "hash", preCalculatedHash ?? string.Empty },
        };

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
