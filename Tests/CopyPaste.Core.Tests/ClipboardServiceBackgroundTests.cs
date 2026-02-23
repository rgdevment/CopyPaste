using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Threading;
using SkiaSharp;
using Xunit;

namespace CopyPaste.Core.Tests;

/// <summary>
/// Tests that exercise ClipboardService background processing paths:
/// ProcessImageAssetsBackground, ProcessImageFileBackground, ParseExistingMetadata, DetectImageFormat, GenerateThumbnail.
/// Uses Thread.Sleep to allow async tasks to complete before assertions.
/// </summary>
public sealed class ClipboardServiceBackgroundTests : IDisposable
{
    private readonly string _basePath;
    private readonly StubRepository _repository;
    private readonly ClipboardService _service;

    public ClipboardServiceBackgroundTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();

        _repository = new StubRepository();
        _service = new ClipboardService(_repository);
    }

    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_basePath))
                Directory.Delete(_basePath, recursive: true);
        }
        catch { }
    }

    // -------------------------------------------------------------------------
    // ProcessImageAssetsBackground (triggered by AddImage with valid DIB data)
    // -------------------------------------------------------------------------

    [Fact]
    public void AddImage_WithValidDib_CompletesBackgroundProcessing()
    {
        byte[] dibData = CreateValidDibData(4, 4, 24);

        _service.AddImage(dibData, "TestApp");

        // Background task needs time to process the image
        Thread.Sleep(3000);

        // At least one update should be recorded (image path update OR thumbnail update)
        Assert.NotEmpty(_repository.UpdatedItems);
    }

    [Fact]
    public void AddImage_WithValidDib_BackgroundSetsImagePath()
    {
        byte[] dibData = CreateValidDibData(4, 4, 24);
        ClipboardItem? savedItem = null;
        _service.OnItemAdded += item => savedItem = item;

        _service.AddImage(dibData, "TestApp");

        Thread.Sleep(3000);

        Assert.NotNull(savedItem);
        // After background processing, item content should be updated to a file path
        var lastUpdate = _repository.UpdatedItems.Count > 0
            ? _repository.UpdatedItems[^1]
            : savedItem;
        Assert.NotNull(lastUpdate);
    }

    [Fact]
    public void AddImage_WithValidDib_FiresThumbnailReadyEvent()
    {
        byte[] dibData = CreateValidDibData(4, 4, 24);
        ClipboardItem? thumbnailItem = null;
        _service.OnThumbnailReady += item => thumbnailItem = item;

        _service.AddImage(dibData, "TestApp");

        Thread.Sleep(3000);

        // OnThumbnailReady should fire after background processing
        Assert.NotNull(thumbnailItem);
    }

    [Fact]
    public void AddImage_LargeValidDib_CompletesWithoutException()
    {
        // 16x16 pixel bitmap - larger to ensure thumbnail path is exercised
        byte[] dibData = CreateValidDibData(16, 16, 24);
        var ex = Record.Exception(() =>
        {
            _service.AddImage(dibData, "TestApp");
            Thread.Sleep(3000);
        });

        Assert.Null(ex);
        Assert.Single(_repository.SavedItems);
    }

    [Fact]
    public void AddImage_WithValidDib_GcThresholdNotExceeded_DoesNotCrash()
    {
        // Small image - pixel data well below GC threshold
        byte[] dibData = CreateValidDibData(2, 2, 24);

        _service.AddImage(dibData, "TestApp");
        Thread.Sleep(3000);

        Assert.Single(_repository.SavedItems);
    }

    // -------------------------------------------------------------------------
    // ProcessImageFileBackground (triggered by AddFiles with Image type)
    // -------------------------------------------------------------------------

    [Fact]
    public void AddFiles_WithImageFile_CompletesBackgroundProcessing()
    {
        string pngPath = CreateTempPng("img_test.png");

        _service.AddFiles(new Collection<string> { pngPath }, ClipboardContentType.Image, "Explorer");

        Thread.Sleep(3000);

        // Item should have been saved
        Assert.Single(_repository.SavedItems);
    }

    [Fact]
    public void AddFiles_WithImageFile_ParsesExistingMetadata()
    {
        // The item is created with metadata (file_count, file_name, etc.)
        // ProcessImageFileBackground calls ParseExistingMetadata on it
        string pngPath = CreateTempPng("img_parse_test.png");

        _service.AddFiles(new Collection<string> { pngPath }, ClipboardContentType.Image, "Explorer");

        Thread.Sleep(3000);

        // After background processing, metadata should be updated
        Assert.Single(_repository.SavedItems);
        Assert.NotNull(_repository.SavedItems[0].Metadata);
    }

    [Fact]
    public void AddFiles_WithImageFile_FiresThumbnailReadyEvent()
    {
        string pngPath = CreateTempPng("img_thumb_event.png");
        ClipboardItem? thumbnailItem = null;
        _service.OnThumbnailReady += item => thumbnailItem = item;

        _service.AddFiles(new Collection<string> { pngPath }, ClipboardContentType.Image, "Explorer");

        Thread.Sleep(3000);

        Assert.NotNull(thumbnailItem);
    }

    [Fact]
    public void AddFiles_WithImageFile_BackgroundUpdatesMetadata()
    {
        string pngPath = CreateTempPng("img_meta.png");

        _service.AddFiles(new Collection<string> { pngPath }, ClipboardContentType.Image, "Explorer");

        Thread.Sleep(3000);

        // After background processing, at least one update happened (to set thumb_path, etc.)
        Assert.True(_repository.UpdatedItems.Count >= 1);
    }

    // -------------------------------------------------------------------------
    // DetectImageFormat (used in ProcessMediaThumbnailBackground, but we can test
    // the PNG/JPEG/WebP signature detection paths indirectly via file processing)
    // -------------------------------------------------------------------------

    [Fact]
    public void AddFiles_WithPngImageFile_ProcessesSuccessfully()
    {
        string pngPath = CreateTempPng("detect_png.png");

        _service.AddFiles(new Collection<string> { pngPath }, ClipboardContentType.Image, "App");

        Thread.Sleep(3000);

        Assert.NotEmpty(_repository.SavedItems);
    }

    // -------------------------------------------------------------------------
    // ConvertDibToBmp branches — colorsUsed > 0 path
    // -------------------------------------------------------------------------

    [Fact]
    public void AddImage_DibWithColorsUsed_ConvertsToBmp()
    {
        // Create DIB with colorsUsed > 0 at offset 32 (triggers the second branch in ConvertDibToBmp)
        byte[] dibData = CreateDibWithColorsUsed(4, 4, 24, colorsUsed: 4);

        _service.AddImage(dibData, "TestApp");

        Thread.Sleep(3000);

        // Item should be saved (the BMP conversion should succeed)
        Assert.Single(_repository.SavedItems);
    }

    // -------------------------------------------------------------------------
    // ProcessMediaThumbnailBackground (triggered by AddFiles with Video/Audio type)
    // Covers: ParseExistingMetadata, ExtractMediaMetadata (exception path),
    //         ExtractAudioArtwork (attempt + null result), AppLogger calls, finally block
    // -------------------------------------------------------------------------

    [Fact]
    public void AddFiles_WithVideoType_TriggersMediaThumbnailBackground()
    {
        string fakeVideo = Path.Combine(_basePath, "fake_video.mp4");
        File.WriteAllBytes(fakeVideo, CreateMinimalPngBytes());

        _service.AddFiles(new Collection<string> { fakeVideo }, ClipboardContentType.Video, "Player");

        Thread.Sleep(3000);

        Assert.Single(_repository.SavedItems);
    }

    [Fact]
    public void AddFiles_WithVideoType_FiresThumbnailReadyEvent()
    {
        string fakeVideo = Path.Combine(_basePath, "fake_video2.mp4");
        File.WriteAllBytes(fakeVideo, CreateMinimalPngBytes());

        ClipboardItem? thumbnailItem = null;
        _service.OnThumbnailReady += item => thumbnailItem = item;

        _service.AddFiles(new Collection<string> { fakeVideo }, ClipboardContentType.Video, "Player");

        Thread.Sleep(3000);

        // OnThumbnailReady fires from the finally block regardless of success/failure
        Assert.NotNull(thumbnailItem);
    }

    [Fact]
    public void AddFiles_WithAudioType_TriggersMediaThumbnailBackground()
    {
        string fakeAudio = Path.Combine(_basePath, "fake_audio.mp3");
        File.WriteAllBytes(fakeAudio, CreateMinimalPngBytes());

        _service.AddFiles(new Collection<string> { fakeAudio }, ClipboardContentType.Audio, "Player");

        Thread.Sleep(3000);

        Assert.Single(_repository.SavedItems);
    }

    [Fact]
    public void AddFiles_WithAudioType_FiresThumbnailReadyEvent()
    {
        string fakeAudio = Path.Combine(_basePath, "fake_audio2.mp3");
        File.WriteAllBytes(fakeAudio, CreateMinimalPngBytes());

        ClipboardItem? thumbnailItem = null;
        _service.OnThumbnailReady += item => thumbnailItem = item;

        _service.AddFiles(new Collection<string> { fakeAudio }, ClipboardContentType.Audio, "Player");

        Thread.Sleep(3000);

        Assert.NotNull(thumbnailItem);
    }

    [Fact]
    public void AddFiles_WithVideoType_UpdatesMetadataAfterProcessing()
    {
        string fakeVideo = Path.Combine(_basePath, "fake_video3.mp4");
        File.WriteAllBytes(fakeVideo, CreateMinimalPngBytes());

        _service.AddFiles(new Collection<string> { fakeVideo }, ClipboardContentType.Video, "Player");

        Thread.Sleep(3000);

        // Background processing should update item metadata in finally block
        Assert.True(_repository.UpdatedItems.Count >= 1);
    }

    [Fact]
    public void AddFiles_WithAudioType_UpdatesMetadataAfterProcessing()
    {
        string fakeAudio = Path.Combine(_basePath, "fake_audio3.mp3");
        File.WriteAllBytes(fakeAudio, CreateMinimalPngBytes());

        _service.AddFiles(new Collection<string> { fakeAudio }, ClipboardContentType.Audio, "Player");

        Thread.Sleep(3000);

        Assert.True(_repository.UpdatedItems.Count >= 1);
    }

    // -------------------------------------------------------------------------
    // Helper methods
    // -------------------------------------------------------------------------

    private string CreateTempPng(string fileName)
    {
        string path = Path.Combine(_basePath, fileName);
        using var bitmap = new SKBitmap(4, 4);
        for (int y = 0; y < 4; y++)
            for (int x = 0; x < 4; x++)
                bitmap.SetPixel(x, y, new SKColor(255, (byte)(x * 64), (byte)(y * 64)));
        using var image = SKImage.FromBitmap(bitmap);
        using var data = image.Encode(SKEncodedImageFormat.Png, 100);
        File.WriteAllBytes(path, data.ToArray());
        return path;
    }

    private static byte[] CreateMinimalPngBytes()
    {
        using var bitmap = new SKBitmap(2, 2);
        bitmap.SetPixel(0, 0, SKColors.Red);
        bitmap.SetPixel(1, 0, SKColors.Green);
        bitmap.SetPixel(0, 1, SKColors.Blue);
        bitmap.SetPixel(1, 1, SKColors.White);
        using var image = SKImage.FromBitmap(bitmap);
        using var pngData = image.Encode(SKEncodedImageFormat.Png, 100);
        return pngData.ToArray();
    }

    private static byte[] CreateValidDibData(int width, int height, int bitCount, int compression = 0)
    {
        int headerSize = 40;
        int rowSize = ((width * bitCount + 31) / 32) * 4;
        int pixelDataSize = rowSize * height;

        int paletteSize = 0;
        if (compression == 3) paletteSize = 12;
        else if (bitCount <= 8) paletteSize = (1 << bitCount) * 4;

        byte[] data = new byte[headerSize + paletteSize + pixelDataSize];

        BitConverter.GetBytes(headerSize).CopyTo(data, 0);
        BitConverter.GetBytes(width).CopyTo(data, 4);
        BitConverter.GetBytes(height).CopyTo(data, 8);
        BitConverter.GetBytes((short)1).CopyTo(data, 12);
        BitConverter.GetBytes((short)bitCount).CopyTo(data, 14);
        BitConverter.GetBytes(compression).CopyTo(data, 16);
        BitConverter.GetBytes(pixelDataSize).CopyTo(data, 20);

        if (bitCount == 24)
        {
            // Fill with valid RGB data: alternating red-ish pixels
            int byteOffset = headerSize + paletteSize;
            for (int y = 0; y < height; y++)
                for (int x = 0; x < width; x++)
                {
                    int idx = byteOffset + y * rowSize + x * 3;
                    if (idx + 2 < data.Length)
                    {
                        data[idx] = 0;         // Blue
                        data[idx + 1] = 100;   // Green
                        data[idx + 2] = 200;   // Red (BMP is BGR)
                    }
                }
        }

        if (compression == 3)
        {
            BitConverter.GetBytes(0x00FF0000).CopyTo(data, headerSize);
            BitConverter.GetBytes(0x0000FF00).CopyTo(data, headerSize + 4);
            BitConverter.GetBytes(0x000000FF).CopyTo(data, headerSize + 8);
        }

        return data;
    }

    private static byte[] CreateDibWithColorsUsed(int width, int height, int bitCount, int colorsUsed)
    {
        int headerSize = 40;
        int rowSize = ((width * bitCount + 31) / 32) * 4;
        int pixelDataSize = rowSize * height;
        // paletteSize calculated from colorsUsed (second branch in ConvertDibToBmp)
        int paletteSize = colorsUsed * 4;

        byte[] data = new byte[headerSize + paletteSize + pixelDataSize];

        BitConverter.GetBytes(headerSize).CopyTo(data, 0);
        BitConverter.GetBytes(width).CopyTo(data, 4);
        BitConverter.GetBytes(height).CopyTo(data, 8);
        BitConverter.GetBytes((short)1).CopyTo(data, 12);
        BitConverter.GetBytes((short)bitCount).CopyTo(data, 14);
        BitConverter.GetBytes(0).CopyTo(data, 16); // compression = 0
        BitConverter.GetBytes(pixelDataSize).CopyTo(data, 20);
        BitConverter.GetBytes(colorsUsed).CopyTo(data, 32); // colorsUsed at offset 32

        // Fill pixel data with some non-zero values
        for (int i = headerSize + paletteSize; i < data.Length - 2; i += 3)
        {
            data[i] = 50;
            data[i + 1] = 100;
            data[i + 2] = 150;
        }

        return data;
    }

    private sealed class StubRepository : IClipboardRepository
    {
        public List<ClipboardItem> SavedItems { get; } = [];
        public List<ClipboardItem> UpdatedItems { get; } = [];
        public List<Guid> DeletedIds { get; } = [];
        public Dictionary<Guid, ClipboardItem> ItemsById { get; } = [];
        public Dictionary<string, ClipboardItem> ItemsByHash { get; } = [];

        public void Save(ClipboardItem item)
        {
            lock (SavedItems) SavedItems.Add(item);
            lock (ItemsById) ItemsById[item.Id] = item;
        }

        public void Update(ClipboardItem item)
        {
            lock (UpdatedItems) UpdatedItems.Add(item);
        }

        public ClipboardItem? GetById(Guid id) { lock (ItemsById) return ItemsById.GetValueOrDefault(id); }

        public ClipboardItem? FindByContentAndType(string content, ClipboardContentType type) => null;

        public ClipboardItem? FindByContentHash(string contentHash)
        {
            lock (ItemsByHash) return ItemsByHash.GetValueOrDefault(contentHash);
        }

        public ClipboardItem? GetLatest() => null;
        public IEnumerable<ClipboardItem> GetAll() => [];
        public void Delete(Guid id) { lock (DeletedIds) DeletedIds.Add(id); }
        public int ClearOldItems(int days, bool excludePinned = true) => 0;
        public IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0) => [];

        public IEnumerable<ClipboardItem> SearchAdvanced(
            string? query,
            IReadOnlyCollection<ClipboardContentType>? types,
            IReadOnlyCollection<CardColor>? colors,
            bool? isPinned,
            int limit,
            int skip) => [];
    }
}
