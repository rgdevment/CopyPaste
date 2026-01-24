using SkiaSharp;
using System.Text.Json;

namespace CopyPaste.Core;

public class ClipboardService(IClipboardRepository repository)
{
    public void AddItem(ClipboardItem item, byte[]? rawData = null)
    {
        ArgumentNullException.ThrowIfNull(item);

        if (item.Type == ClipboardContentType.Image && rawData != null)
        {
            ProcessImageStorage(item, rawData);
        }

        repository.Save(item);
    }

    private static void ProcessImageStorage(ClipboardItem item, byte[] rawData)
    {
        // Save original file
        string originalPath = Path.Combine(StorageConfig.ImagesPath, $"{item.Id}.png");
        File.WriteAllBytes(originalPath, rawData);
        item.Content = originalPath;

        // Generate and save thumbnail
        string thumbPath = Path.Combine(StorageConfig.ThumbnailsPath, $"{item.Id}_t.png");

        using var managedSrc = new MemoryStream(rawData);
        using var bitmap = SKBitmap.Decode(managedSrc);

        int width = 200;
        int height = (int)(bitmap.Height * (200.0 / bitmap.Width));

        var sampling = new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.None);
        using var resized = bitmap.Resize(new SKImageInfo(width, height), sampling);
        using var image = SKImage.FromBitmap(resized);
        using var data = image.Encode(SKEncodedImageFormat.Png, 80);
        using var stream = File.Create(thumbPath);
        data.SaveTo(stream);

        var dataMap = new Dictionary<string, object>
        {
            { "thumb_path", thumbPath },
            { "thumb_width", width },
            { "thumb_height", height },
            { "width", bitmap.Width },
            { "height", bitmap.Height },
            { "size", rawData.Length },
        };

        item.Metadata = JsonSerializer.Serialize(dataMap);
    }

    public IEnumerable<ClipboardItem> GetHistory(string? query = null) =>
        string.IsNullOrWhiteSpace(query) ? repository.GetAll() : repository.Search(query);

    public void RemoveItem(Guid id) => repository.Delete(id);
}
