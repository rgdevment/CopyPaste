using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Json;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class ClipboardServiceAdditionalTests : IDisposable
{
    private readonly string _basePath;
    private readonly StubClipboardRepository _repository;
    private readonly ClipboardService _service;

    public ClipboardServiceAdditionalTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();

        _repository = new StubClipboardRepository();
        _service = new ClipboardService(_repository);
    }

    #region ConvertDibToBmp via AddImage

    [Fact]
    public void AddImage_WithValidDibData_SavesItem()
    {
        byte[] dibData = CreateValidDibData(2, 2, 24);

        _service.AddImage(dibData, "TestApp");

        Assert.Single(_repository.SavedItems);
        Assert.Equal(ClipboardContentType.Image, _repository.SavedItems[0].Type);
    }

    [Fact]
    public void AddImage_WithValidDibData_SetsContentHash()
    {
        byte[] dibData = CreateValidDibData(2, 2, 24);

        _service.AddImage(dibData, "TestApp");

        Assert.Single(_repository.SavedItems);
        var hash = _repository.SavedItems[0].ContentHash;
        Assert.NotNull(hash);
        Assert.NotEmpty(hash);
    }

    [Fact]
    public void AddImage_WithValidDibData_SetsMetadataWithHash()
    {
        byte[] dibData = CreateValidDibData(2, 2, 24);

        _service.AddImage(dibData, "TestApp");

        Assert.Single(_repository.SavedItems);
        Assert.NotNull(_repository.SavedItems[0].Metadata);
        Assert.Contains("hash", _repository.SavedItems[0].Metadata, StringComparison.Ordinal);
    }

    [Fact]
    public void AddImage_DibDataTooShort_DoesNotSave()
    {
        _service.AddImage(new byte[39], "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void AddImage_With8BitDib_HandlesColorPalette()
    {
        byte[] dibData = CreateValidDibData(2, 2, 8);

        _service.AddImage(dibData, "TestApp");

        Assert.Single(_repository.SavedItems);
    }

    [Fact]
    public void AddImage_WithCompression3_HandlesBitmasks()
    {
        byte[] dibData = CreateValidDibData(2, 2, 32, compression: 3);

        _service.AddImage(dibData, "TestApp");

        Assert.Single(_repository.SavedItems);
    }

    [Fact]
    public void AddImage_SameImageTwice_ReactivatesExisting()
    {
        byte[] dibData = CreateValidDibData(1, 1, 24);

        _service.AddImage(dibData, "TestApp");
        Assert.Single(_repository.SavedItems);

        var savedHash = _repository.SavedItems[0].ContentHash;
        _repository.ItemsByHash[savedHash!] = _repository.SavedItems[0];

        ClipboardItem? reactivated = null;
        _service.OnItemReactivated += item => reactivated = item;

        _service.AddImage(dibData, "TestApp");

        Assert.NotNull(reactivated);
        Assert.Single(_repository.SavedItems);
    }

    [Fact]
    public void AddImage_FiresOnItemAddedEvent()
    {
        ClipboardItem? added = null;
        _service.OnItemAdded += item => added = item;

        byte[] dibData = CreateValidDibData(1, 1, 24);
        _service.AddImage(dibData, "TestApp");

        Assert.NotNull(added);
        Assert.Equal(ClipboardContentType.Image, added.Type);
    }

    [Fact]
    public void AddImage_SetsAppSource()
    {
        byte[] dibData = CreateValidDibData(1, 1, 24);
        _service.AddImage(dibData, "Photoshop");

        Assert.Single(_repository.SavedItems);
        Assert.Equal("Photoshop", _repository.SavedItems[0].AppSource);
    }

    #endregion

    #region GetHistory and GetHistoryAdvanced

    [Fact]
    public void GetHistory_WithDefaultParams_CallsRepository()
    {
        var items = _service.GetHistory().ToList();
        Assert.Empty(items);
    }

    [Fact]
    public void GetHistory_PassesParametersCorrectly()
    {
        _repository.SearchAdvancedCallback = (query, types, colors, isPinned, limit, skip) =>
        {
            Assert.Equal("search", query);
            Assert.Null(types);
            Assert.Null(colors);
            Assert.True(isPinned);
            Assert.Equal(10, limit);
            Assert.Equal(5, skip);
        };

        var result = _service.GetHistory(10, 5, "search", true).ToList();
        Assert.NotNull(result);
    }

    [Fact]
    public void GetHistoryAdvanced_PassesAllParameters()
    {
        var typeFilter = new List<ClipboardContentType> { ClipboardContentType.Text, ClipboardContentType.Link };
        var colorFilter = new List<CardColor> { CardColor.Red };

        _repository.SearchAdvancedCallback = (query, types, colors, isPinned, limit, skip) =>
        {
            Assert.Equal("test", query);
            Assert.NotNull(types);
            Assert.Equal(2, types!.Count);
            Assert.NotNull(colors);
            Assert.Single(colors!);
            Assert.False(isPinned);
            Assert.Equal(20, limit);
            Assert.Equal(0, skip);
        };

        var result = _service.GetHistoryAdvanced(20, 0, "test", typeFilter, colorFilter, false).ToList();
        Assert.NotNull(result);
    }

    [Fact]
    public void GetHistoryAdvanced_WithNullFilters_PassesNulls()
    {
        _repository.SearchAdvancedCallback = (query, types, colors, isPinned, limit, skip) =>
        {
            Assert.Null(query);
            Assert.Null(types);
            Assert.Null(colors);
            Assert.Null(isPinned);
        };

        var result = _service.GetHistoryAdvanced(50, 0, null, null, null, null).ToList();
        Assert.NotNull(result);
    }

    #endregion

    #region AddText with HTML bytes

    [Fact]
    public void AddText_WithHtmlBytes_SavesMetadata()
    {
        var htmlBytes = new byte[] { 60, 104, 49, 62, 72, 105, 60, 47, 104, 49, 62 }; // <h1>Hi</h1>
        _service.AddText("Hi", ClipboardContentType.Text, "TestApp", htmlBytes: htmlBytes);

        Assert.Single(_repository.SavedItems);
        Assert.NotNull(_repository.SavedItems[0].Metadata);
        Assert.Contains("html", _repository.SavedItems[0].Metadata, StringComparison.Ordinal);
    }

    [Fact]
    public void AddText_WithBothRtfAndHtml_SavesBothInMetadata()
    {
        var rtfBytes = new byte[] { 1, 2, 3 };
        var htmlBytes = new byte[] { 4, 5, 6 };
        _service.AddText("Test", ClipboardContentType.Text, "TestApp", rtfBytes, htmlBytes);

        Assert.Single(_repository.SavedItems);
        var metadata = _repository.SavedItems[0].Metadata!;
        Assert.Contains("rtf", metadata, StringComparison.Ordinal);
        Assert.Contains("html", metadata, StringComparison.Ordinal);
    }

    #endregion

    #region AddFiles with metadata details

    [Fact]
    public void AddFiles_WithSingleFile_IncludesFileExtensionInMetadata()
    {
        var testFile = Path.Combine(_basePath, "document.pdf");
        File.WriteAllText(testFile, "fake pdf");

        _service.AddFiles(new Collection<string> { testFile }, ClipboardContentType.File, "Explorer");

        Assert.Single(_repository.SavedItems);
        Assert.Contains(".pdf", _repository.SavedItems[0].Metadata!, StringComparison.Ordinal);
    }

    [Fact]
    public void AddFiles_WithSingleFile_IncludesFileNameInMetadata()
    {
        var testFile = Path.Combine(_basePath, "readme.txt");
        File.WriteAllText(testFile, "content");

        _service.AddFiles(new Collection<string> { testFile }, ClipboardContentType.File, "Explorer");

        Assert.Contains("readme.txt", _repository.SavedItems[0].Metadata!, StringComparison.Ordinal);
    }

    [Fact]
    public void AddFiles_WithDirectory_DoesNotIncludeFileSize()
    {
        var testDir = Path.Combine(_basePath, "myFolder");
        Directory.CreateDirectory(testDir);

        _service.AddFiles(new Collection<string> { testDir }, ClipboardContentType.Folder, "Explorer");

        Assert.DoesNotContain("file_size", _repository.SavedItems[0].Metadata!, StringComparison.Ordinal);
    }

    [Fact]
    public void AddFiles_MetadataIsValidJson()
    {
        var testFile = Path.Combine(_basePath, "jsontest.txt");
        File.WriteAllText(testFile, "test");

        _service.AddFiles(new Collection<string> { testFile }, ClipboardContentType.File, "Explorer");

        var metadata = _repository.SavedItems[0].Metadata!;
        using var doc = JsonDocument.Parse(metadata);
        Assert.NotNull(doc);
    }

    [Fact]
    public void AddFiles_WithThreeFiles_FileCountIsCorrect()
    {
        var files = new Collection<string>();
        for (int i = 0; i < 3; i++)
        {
            var path = Path.Combine(_basePath, $"file{i}.txt");
            File.WriteAllText(path, "content");
            files.Add(path);
        }

        _service.AddFiles(files, ClipboardContentType.File, "Explorer");

        Assert.Contains("3", _repository.SavedItems[0].Metadata!, StringComparison.Ordinal);
    }

    #endregion

    #region Duplicate Detection for Non-Image Types

    [Fact]
    public void AddText_DuplicateByContentAndType_ReactivatesItem()
    {
        var existing = new ClipboardItem
        {
            Id = Guid.NewGuid(),
            Content = "Same content",
            Type = ClipboardContentType.Text
        };
        _repository.ByContentAndType[(existing.Content, existing.Type)] = existing;

        ClipboardItem? reactivated = null;
        _service.OnItemReactivated += item => reactivated = item;

        _service.AddText("Same content", ClipboardContentType.Text, "App");

        Assert.NotNull(reactivated);
        Assert.Equal(existing.Id, reactivated.Id);
        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void AddText_SameContentDifferentType_SavesNewItem()
    {
        var existing = new ClipboardItem
        {
            Id = Guid.NewGuid(),
            Content = "https://example.com",
            Type = ClipboardContentType.Text
        };
        _repository.ByContentAndType[(existing.Content, existing.Type)] = existing;

        _service.AddText("https://example.com", ClipboardContentType.Link, "App");

        Assert.Single(_repository.SavedItems);
    }

    #endregion

    #region RemoveItem and UpdatePin edge cases

    [Fact]
    public void RemoveItem_ExistingItem_CallsDelete()
    {
        var id = Guid.NewGuid();
        _repository.ItemsById[id] = new ClipboardItem { Id = id, Content = "test" };

        _service.RemoveItem(id);

        Assert.Single(_repository.DeletedIds);
    }

    [Fact]
    public void RemoveItem_NonExistent_NoDelete()
    {
        _service.RemoveItem(Guid.NewGuid());
        Assert.Empty(_repository.DeletedIds);
    }

    [Fact]
    public void UpdatePin_ExistingItem_PinsSuccessfully()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "test", IsPinned = false };
        _repository.ItemsById[id] = item;

        _service.UpdatePin(id, true);

        Assert.True(item.IsPinned);
    }

    [Fact]
    public void UpdateLabelAndColor_SetsValuesCorrectly()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "test" };
        _repository.ItemsById[id] = item;

        _service.UpdateLabelAndColor(id, "Important", CardColor.Red);

        Assert.Equal("Important", item.Label);
        Assert.Equal(CardColor.Red, item.CardColor);
    }

    [Fact]
    public void MarkItemUsed_ReturnsUpdatedItem()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "test", PasteCount = 3 };
        _repository.ItemsById[id] = item;

        var result = _service.MarkItemUsed(id);

        Assert.NotNull(result);
        Assert.Equal(4, result.PasteCount);
    }

    #endregion

    #region PasteIgnoreWindowMs

    [Fact]
    public void PasteIgnoreWindowMs_DefaultIs450()
    {
        Assert.Equal(450, _service.PasteIgnoreWindowMs);
    }

    [Fact]
    public void PasteIgnoreWindowMs_CanBeSet()
    {
        _service.PasteIgnoreWindowMs = 1000;
        Assert.Equal(1000, _service.PasteIgnoreWindowMs);
    }

    #endregion

    private static byte[] CreateValidDibData(int width, int height, int bitCount, int compression = 0)
    {
        int headerSize = 40;
        int bytesPerPixel = bitCount / 8;
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

        if (compression == 3)
        {
            BitConverter.GetBytes(0x00FF0000).CopyTo(data, headerSize);
            BitConverter.GetBytes(0x0000FF00).CopyTo(data, headerSize + 4);
            BitConverter.GetBytes(0x000000FF).CopyTo(data, headerSize + 8);
        }

        return data;
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

    private sealed class StubClipboardRepository : IClipboardRepository
    {
        public List<ClipboardItem> SavedItems { get; } = [];
        public List<ClipboardItem> UpdatedItems { get; } = [];
        public Dictionary<Guid, ClipboardItem> ItemsById { get; } = [];
        public Dictionary<string, ClipboardItem> ItemsByHash { get; } = [];
        public Dictionary<(string, ClipboardContentType), ClipboardItem> ByContentAndType { get; } = [];
        public List<Guid> DeletedIds { get; } = [];

        public Action<string?, IReadOnlyCollection<ClipboardContentType>?, IReadOnlyCollection<CardColor>?, bool?, int, int>? SearchAdvancedCallback { get; set; }

        public void Save(ClipboardItem item) => SavedItems.Add(item);
        public void Update(ClipboardItem item) => UpdatedItems.Add(item);
        public ClipboardItem? GetById(Guid id) => ItemsById.GetValueOrDefault(id);
        public ClipboardItem? GetLatest() => null;

        public ClipboardItem? FindByContentAndType(string content, ClipboardContentType type) =>
            ByContentAndType.GetValueOrDefault((content, type));

        public ClipboardItem? FindByContentHash(string contentHash) =>
            ItemsByHash.GetValueOrDefault(contentHash);

        public IEnumerable<ClipboardItem> GetAll() => [];
        public void Delete(Guid id) => DeletedIds.Add(id);
        public int ClearOldItems(int days, bool excludePinned = true) => 0;
        public IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0) => [];

        public IEnumerable<ClipboardItem> SearchAdvanced(
            string? query,
            IReadOnlyCollection<ClipboardContentType>? types,
            IReadOnlyCollection<CardColor>? colors,
            bool? isPinned,
            int limit,
            int skip)
        {
            SearchAdvancedCallback?.Invoke(query, types, colors, isPinned, limit, skip);
            return [];
        }
    }
}
