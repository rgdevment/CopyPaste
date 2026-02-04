using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading;
using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class ClipboardServiceTests : IDisposable
{
    private readonly string _basePath;
    private readonly StubClipboardRepository _repository;
    private readonly ClipboardService _service;

    public ClipboardServiceTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();

        _repository = new StubClipboardRepository();
        _service = new ClipboardService(_repository);
    }

    #region AddText Tests

    [Fact]
    public void AddText_WithValidText_SavesItem()
    {
        _service.AddText("Hello World", ClipboardContentType.Text, "TestApp");

        Assert.Single(_repository.SavedItems);
        Assert.Equal("Hello World", _repository.SavedItems[0].Content);
        Assert.Equal(ClipboardContentType.Text, _repository.SavedItems[0].Type);
        Assert.Equal("TestApp", _repository.SavedItems[0].AppSource);
    }

    [Fact]
    public void AddText_WithNullText_SavesEmptyString()
    {
        _service.AddText(null, ClipboardContentType.Text, "TestApp");

        Assert.Single(_repository.SavedItems);
        Assert.Equal(string.Empty, _repository.SavedItems[0].Content);
    }

    [Fact]
    public void AddText_WithEmptyText_SavesEmptyString()
    {
        _service.AddText(string.Empty, ClipboardContentType.Text, "TestApp");

        Assert.Single(_repository.SavedItems);
        Assert.Equal(string.Empty, _repository.SavedItems[0].Content);
    }

    [Fact]
    public void AddText_WithLink_SavesAsLink()
    {
        _service.AddText("https://example.com", ClipboardContentType.Link, "Browser");

        Assert.Single(_repository.SavedItems);
        Assert.Equal(ClipboardContentType.Link, _repository.SavedItems[0].Type);
    }

    [Fact]
    public void AddText_WithRtfBytes_SavesMetadata()
    {
        var rtfBytes = new byte[] { 1, 2, 3, 4 };
        _service.AddText("Test", ClipboardContentType.Text, "TestApp", rtfBytes);

        Assert.Single(_repository.SavedItems);
        Assert.NotNull(_repository.SavedItems[0].Metadata);
        Assert.Contains("rtf", _repository.SavedItems[0].Metadata, StringComparison.Ordinal);
    }

    [Fact]
    public void AddText_WithoutRtfBytes_HasNullMetadata()
    {
        _service.AddText("Test", ClipboardContentType.Text, "TestApp");

        Assert.Single(_repository.SavedItems);
        Assert.Null(_repository.SavedItems[0].Metadata);
    }

    [Fact]
    public void AddText_FiresOnItemAddedEvent()
    {
        ClipboardItem? addedItem = null;
        _service.OnItemAdded += item => addedItem = item;

        _service.AddText("Test", ClipboardContentType.Text, "TestApp");

        Assert.NotNull(addedItem);
        Assert.Equal("Test", addedItem.Content);
    }

    #endregion

    #region AddFiles Tests

    [Fact]
    public void AddFiles_WithNullFiles_DoesNotSave()
    {
        _service.AddFiles(null, ClipboardContentType.File, "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void AddFiles_WithEmptyFiles_DoesNotSave()
    {
        _service.AddFiles(new Collection<string>(), ClipboardContentType.File, "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void AddFiles_WithSingleFile_SavesWithMetadata()
    {
        var testFile = Path.Combine(_basePath, "test.txt");
        File.WriteAllText(testFile, "content");

        _service.AddFiles(new Collection<string> { testFile }, ClipboardContentType.File, "Explorer");

        Assert.Single(_repository.SavedItems);
        Assert.Equal(testFile, _repository.SavedItems[0].Content);
        Assert.NotNull(_repository.SavedItems[0].Metadata);
        Assert.Contains("file_count", _repository.SavedItems[0].Metadata, StringComparison.Ordinal);
        Assert.Contains("file_name", _repository.SavedItems[0].Metadata, StringComparison.Ordinal);
    }

    [Fact]
    public void AddFiles_WithMultipleFiles_JoinsWithNewLine()
    {
        var file1 = Path.Combine(_basePath, "file1.txt");
        var file2 = Path.Combine(_basePath, "file2.txt");
        File.WriteAllText(file1, "content1");
        File.WriteAllText(file2, "content2");

        _service.AddFiles(new Collection<string> { file1, file2 }, ClipboardContentType.File, "Explorer");

        Assert.Single(_repository.SavedItems);
        Assert.Contains(file1, _repository.SavedItems[0].Content, StringComparison.Ordinal);
        Assert.Contains(file2, _repository.SavedItems[0].Content, StringComparison.Ordinal);
        Assert.Contains(Environment.NewLine, _repository.SavedItems[0].Content, StringComparison.Ordinal);
    }

    [Fact]
    public void AddFiles_WithDirectory_MarksAsDirectory()
    {
        var testDir = Path.Combine(_basePath, "testFolder");
        Directory.CreateDirectory(testDir);

        _service.AddFiles(new Collection<string> { testDir }, ClipboardContentType.Folder, "Explorer");

        Assert.Single(_repository.SavedItems);
        Assert.Contains("is_directory", _repository.SavedItems[0].Metadata!, StringComparison.Ordinal);
    }

    [Fact]
    public void AddFiles_WithNonExistentFile_StillSaves()
    {
        var nonExistent = Path.Combine(_basePath, "nonexistent.txt");

        _service.AddFiles(new Collection<string> { nonExistent }, ClipboardContentType.File, "Explorer");

        Assert.Single(_repository.SavedItems);
        Assert.Equal(nonExistent, _repository.SavedItems[0].Content);
    }

    #endregion

    #region NotifyPasteInitiated Tests

    [Fact]
    public void NotifyPasteInitiated_PreventsAddText()
    {
        var itemId = Guid.NewGuid();
        var item = new ClipboardItem { Id = itemId, Content = "Test", Type = ClipboardContentType.Text };
        _repository.ItemsById[itemId] = item;

        _service.NotifyPasteInitiated(itemId);

        _service.AddText("Test", ClipboardContentType.Text, "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void NotifyPasteInitiated_ExpiresAfterWindow()
    {
        _service.PasteIgnoreWindowMs = 50;
        var itemId = Guid.NewGuid();
        var item = new ClipboardItem { Id = itemId, Content = "Original", Type = ClipboardContentType.Text };
        _repository.ItemsById[itemId] = item;

        _service.NotifyPasteInitiated(itemId);

        Thread.Sleep(100);

        // Different content should be added after timeout
        _service.AddText("Different", ClipboardContentType.Text, "TestApp");

        Assert.Single(_repository.SavedItems);
    }

    [Fact]
    public void NotifyPasteInitiated_SameContentIgnoredForExtendedWindow()
    {
        _service.PasteIgnoreWindowMs = 50;
        var itemId = Guid.NewGuid();
        var item = new ClipboardItem { Id = itemId, Content = "Test", Type = ClipboardContentType.Text };
        _repository.ItemsById[itemId] = item;

        _service.NotifyPasteInitiated(itemId);

        Thread.Sleep(100);

        // Same content should still be ignored within extended 2-second window
        _service.AddText("Test", ClipboardContentType.Text, "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void PasteIgnoreWindowMs_CanBeConfigured()
    {
        _service.PasteIgnoreWindowMs = 1000;

        Assert.Equal(1000, _service.PasteIgnoreWindowMs);
    }

    #endregion

    #region Duplicate Detection Tests

    [Fact]
    public void AddText_DuplicateContent_FiresOnItemReactivated()
    {
        var existingItem = new ClipboardItem
        {
            Id = Guid.NewGuid(),
            Content = "Duplicate",
            Type = ClipboardContentType.Text
        };
        _repository.LatestItem = existingItem;

        ClipboardItem? reactivatedItem = null;
        _service.OnItemReactivated += item => reactivatedItem = item;

        _service.AddText("Duplicate", ClipboardContentType.Text, "TestApp");

        Assert.NotNull(reactivatedItem);
        Assert.Equal(existingItem.Id, reactivatedItem.Id);
        Assert.Single(_repository.UpdatedItems);
        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void AddText_DifferentContent_AddsNewItem()
    {
        var existingItem = new ClipboardItem
        {
            Id = Guid.NewGuid(),
            Content = "Original",
            Type = ClipboardContentType.Text
        };
        _repository.LatestItem = existingItem;

        _service.AddText("Different", ClipboardContentType.Text, "TestApp");

        Assert.Single(_repository.SavedItems);
        Assert.Empty(_repository.UpdatedItems);
    }

    #endregion

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types", Justification = "Best-effort cleanup of temp test data should not fail tests")]
    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_basePath))
            {
                Directory.Delete(_basePath, recursive: true);
            }
        }
        catch
        {
            // Best-effort cleanup
        }
    }

    private sealed class StubClipboardRepository : IClipboardRepository
    {
        public List<ClipboardItem> SavedItems { get; } = new();
        public List<ClipboardItem> UpdatedItems { get; } = new();
        public ClipboardItem? LatestItem { get; set; }
        public Dictionary<Guid, ClipboardItem> ItemsById { get; } = new();

        public void Save(ClipboardItem item) => SavedItems.Add(item);

        public void Update(ClipboardItem item) => UpdatedItems.Add(item);

        public ClipboardItem? GetLatest() => LatestItem;

        public ClipboardItem? FindByContentAndType(string content, ClipboardContentType type)
        {
            // Check LatestItem first (for backward compatibility with tests)
            if (LatestItem != null && LatestItem.Content == content && LatestItem.Type == type)
                return LatestItem;

            // Check ItemsById collection
            return ItemsById.Values.FirstOrDefault(i => i.Content == content && i.Type == type);
        }

        public int ClearOldItems(int days, bool excludePinned = true) => 0;

        public void Delete(Guid id) => throw new NotImplementedException();

        public IEnumerable<ClipboardItem> GetAll() => [];

        public ClipboardItem? GetById(Guid id) => ItemsById.TryGetValue(id, out var item) ? item : null;

        public IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0) => throw new NotImplementedException();

        public ClipboardItem? FindByContentHash(string contentHash) =>
            ItemsById.Values.FirstOrDefault(i => i.ContentHash == contentHash);

        public IEnumerable<ClipboardItem> SearchAdvanced(
            string? query,
            IReadOnlyCollection<ClipboardContentType>? types,
            IReadOnlyCollection<CardColor>? colors,
            bool? isPinned,
            int limit,
            int skip) => [];
    }
}
