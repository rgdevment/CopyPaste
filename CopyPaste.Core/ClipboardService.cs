using SkiaSharp;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace CopyPaste.Core;

public class ClipboardService(IClipboardRepository repository)
{
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1003: primitive object")]
    public event Action<ClipboardItem>? OnThumbnailReady;

    public void AddText(string? text)
    {
        if (string.IsNullOrWhiteSpace(text)) return;
        AddItem(new ClipboardItem { Content = text, Type = ClipboardContentType.Text });
    }

    public void AddHtml(byte[]? rawBytes)
    {
        if (rawBytes == null) return;

        string rawHtml = Encoding.UTF8.GetString(rawBytes).TrimEnd('\0');
        string html = ExtractHtmlFragment(rawHtml);

        if (!string.IsNullOrWhiteSpace(html))
        {
            AddItem(new ClipboardItem { Content = html, Type = ClipboardContentType.Html });
        }
    }

    public void AddImage(byte[]? dibData)
    {
        if (dibData == null) return;

        byte[]? bmp = ConvertDibToBmp(dibData);
        if (bmp != null)
        {
            AddItem(new ClipboardItem { Type = ClipboardContentType.Image }, bmp);
        }
    }

    public void AddFiles(Collection<string>? files)
    {
        if (files == null || files.Count == 0) return;

        string paths = string.Join(Environment.NewLine, files);
        var meta = new Dictionary<string, object>
        {
            { "file_count", files.Count },
            { "first_ext", Path.GetExtension(files[0]) }
        };
        string json = JsonSerializer.Serialize(meta, MetadataJsonContext.Default.DictionaryStringObject);

        AddItem(new ClipboardItem { Content = paths, Type = ClipboardContentType.File, Metadata = json });
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

            OnThumbnailReady?.Invoke(item);
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

    public void RemoveItem(Guid id)
    {
        var item = repository.GetAll().FirstOrDefault(x => x.Id == id);
        if (item == null) return;

        repository.Delete(id);

        _ = Task.Run(() =>
        {
            try
            {
                if (!string.IsNullOrEmpty(item.Content) && File.Exists(item.Content))
                {
                    File.Delete(item.Content);
                }
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
            catch (IOException ex)
            {
                Debug.WriteLine($"Failed to delete physical files: {ex.Message}");
            }
            catch (UnauthorizedAccessException ex)
            {
                Debug.WriteLine($"Failed to delete physical files: {ex.Message}");
            }
            catch (ArgumentException ex)
            {
                Debug.WriteLine($"Failed to delete physical files: {ex.Message}");
            }
        });
    }

    private static string ExtractHtmlFragment(string rawHtml)
    {
        if (string.IsNullOrWhiteSpace(rawHtml)) return string.Empty;

        const string startFragment = "<!--StartFragment-->";
        const string endFragment = "<!--EndFragment-->";

        int startIndex = rawHtml.IndexOf(startFragment, StringComparison.OrdinalIgnoreCase);
        int endIndex = rawHtml.LastIndexOf(endFragment, StringComparison.OrdinalIgnoreCase);

        if (startIndex != -1 && endIndex != -1)
        {
            startIndex += startFragment.Length;
            return rawHtml[startIndex..endIndex].Trim();
        }

        int htmlStart = rawHtml.IndexOf("<html", StringComparison.OrdinalIgnoreCase);
        return htmlStart != -1 ? rawHtml[htmlStart..].Trim() : rawHtml;
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
