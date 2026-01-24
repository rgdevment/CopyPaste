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

        var latest = repository.GetLatest();

        if (IsDuplicate(latest, item, rawData))
        {
            latest!.CreatedAt = DateTime.Now;
            repository.Update(latest);
            return;
        }

        // Ensure ID is ready for file naming
        if (item.Id == Guid.Empty) item.Id = Guid.NewGuid();

        // 1. Save record immediately to DB
        repository.Save(item);

        // 2. Fire and forget heavy assets processing
        if (item.Type == ClipboardContentType.Image && rawData != null)
        {
            _ = Task.Run(() => ProcessImageAssetsBackground(item, rawData));
        }
    }

    private void ProcessImageAssetsBackground(ClipboardItem item, byte[] rawData)
    {
        try
        {
            string originalPath = Path.Combine(StorageConfig.ImagesPath, $"{item.Id}.png");
            File.WriteAllBytes(originalPath, rawData);
            item.Content = originalPath;

            string thumbPath = Path.Combine(StorageConfig.ThumbnailsPath, $"{item.Id}_t.png");

            using var managedSrc = new MemoryStream(rawData);
            using var bitmap = SKBitmap.Decode(managedSrc);

            // Exit if image data is invalid
            if (bitmap == null) return;

            int width = 200;
            int height = (int)(bitmap.Height * (200.0 / bitmap.Width));

            string hash = BitConverter.ToString(SHA256.HashData(rawData));

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
                { "hash", hash },
            };

            item.Metadata = JsonSerializer.Serialize(dataMap, MetadataJsonContext.Default.DictionaryStringObject);

            // 3. Update DB record with paths and metadata once ready
            repository.Update(item);
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or ArgumentException)
        {
            Debug.WriteLine($"Asset processing failed: {ex.Message}");
        }
    }

    private static bool IsDuplicate(ClipboardItem? last, ClipboardItem current, byte[]? currentImageBuffer)
    {
        if (last == null || last.Type != current.Type) return false;

        return current.Type switch
        {
            ClipboardContentType.Text or
            ClipboardContentType.Html or
            ClipboardContentType.File => last.Content == current.Content,

            ClipboardContentType.Image => CompareImages(last, currentImageBuffer),

            _ => false
        };
    }

    private static bool CompareImages(ClipboardItem last, byte[]? currentBuffer)
    {
        if (currentBuffer == null || string.IsNullOrEmpty(last.Metadata)) return false;

        string currentHash = BitConverter.ToString(SHA256.HashData(currentBuffer));

        using var doc = JsonDocument.Parse(last.Metadata);
        if (doc.RootElement.TryGetProperty("hash", out var hashProp))
        {
            return hashProp.GetString() == currentHash;
        }

        return false;
    }

    public IEnumerable<ClipboardItem> GetHistory(string? query = null) =>
        string.IsNullOrWhiteSpace(query) ? repository.GetAll() : repository.Search(query);

    public void RemoveItem(Guid id) => repository.Delete(id);
}
