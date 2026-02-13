using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading;
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

    #region RemoveItem Tests

    [Fact]
    public void RemoveItem_WithExistingItem_DeletesFromRepository()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text };
        _repository.ItemsById[id] = item;

        _service.RemoveItem(id);

        Assert.Single(_repository.DeletedIds);
        Assert.Equal(id, _repository.DeletedIds[0]);
    }

    [Fact]
    public void RemoveItem_WithNonExistentItem_DoesNotCallDelete()
    {
        _service.RemoveItem(Guid.NewGuid());

        Assert.Empty(_repository.DeletedIds);
    }

    #endregion

    #region UpdatePin Tests

    [Fact]
    public void UpdatePin_WithExistingItem_UpdatesPinStatus()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text, IsPinned = false };
        _repository.ItemsById[id] = item;

        _service.UpdatePin(id, true);

        Assert.True(item.IsPinned);
        Assert.Single(_repository.UpdatedItems);
    }

    [Fact]
    public void UpdatePin_WithExistingItem_UnpinsItem()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text, IsPinned = true };
        _repository.ItemsById[id] = item;

        _service.UpdatePin(id, false);

        Assert.False(item.IsPinned);
        Assert.Single(_repository.UpdatedItems);
    }

    [Fact]
    public void UpdatePin_WithNonExistentItem_DoesNotUpdate()
    {
        _service.UpdatePin(Guid.NewGuid(), true);

        Assert.Empty(_repository.UpdatedItems);
    }

    [Fact]
    public void UpdatePin_UpdatesModifiedAt()
    {
        var id = Guid.NewGuid();
        var originalTime = DateTime.UtcNow.AddHours(-1);
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text, ModifiedAt = originalTime };
        _repository.ItemsById[id] = item;

        _service.UpdatePin(id, true);

        Assert.True(item.ModifiedAt > originalTime);
    }

    #endregion

    #region UpdateLabelAndColor Tests

    [Fact]
    public void UpdateLabelAndColor_WithExistingItem_UpdatesBoth()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text };
        _repository.ItemsById[id] = item;

        _service.UpdateLabelAndColor(id, "My Label", CardColor.Red);

        Assert.Equal("My Label", item.Label);
        Assert.Equal(CardColor.Red, item.CardColor);
        Assert.Single(_repository.UpdatedItems);
    }

    [Fact]
    public void UpdateLabelAndColor_WithNullLabel_SetsNullLabel()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text, Label = "Old Label" };
        _repository.ItemsById[id] = item;

        _service.UpdateLabelAndColor(id, null, CardColor.None);

        Assert.Null(item.Label);
        Assert.Equal(CardColor.None, item.CardColor);
    }

    [Fact]
    public void UpdateLabelAndColor_WithNonExistentItem_DoesNotUpdate()
    {
        _service.UpdateLabelAndColor(Guid.NewGuid(), "Label", CardColor.Blue);

        Assert.Empty(_repository.UpdatedItems);
    }

    [Fact]
    public void UpdateLabelAndColor_UpdatesModifiedAt()
    {
        var id = Guid.NewGuid();
        var originalTime = DateTime.UtcNow.AddHours(-1);
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text, ModifiedAt = originalTime };
        _repository.ItemsById[id] = item;

        _service.UpdateLabelAndColor(id, "Label", CardColor.Green);

        Assert.True(item.ModifiedAt > originalTime);
    }

    [Theory]
    [InlineData(CardColor.None)]
    [InlineData(CardColor.Red)]
    [InlineData(CardColor.Green)]
    [InlineData(CardColor.Purple)]
    [InlineData(CardColor.Yellow)]
    [InlineData(CardColor.Blue)]
    [InlineData(CardColor.Orange)]
    public void UpdateLabelAndColor_AllColors_AreAccepted(CardColor color)
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text };
        _repository.ItemsById[id] = item;

        _service.UpdateLabelAndColor(id, "Label", color);

        Assert.Equal(color, item.CardColor);
    }

    #endregion

    #region MarkItemUsed Tests

    [Fact]
    public void MarkItemUsed_WithExistingItem_IncrementsPasteCount()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text, PasteCount = 0 };
        _repository.ItemsById[id] = item;

        var result = _service.MarkItemUsed(id);

        Assert.NotNull(result);
        Assert.Equal(1, result.PasteCount);
    }

    [Fact]
    public void MarkItemUsed_CalledMultipleTimes_IncrementsPasteCountEachTime()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text, PasteCount = 0 };
        _repository.ItemsById[id] = item;

        _service.MarkItemUsed(id);
        _service.MarkItemUsed(id);
        _service.MarkItemUsed(id);

        Assert.Equal(3, item.PasteCount);
    }

    [Fact]
    public void MarkItemUsed_WithNonExistentItem_ReturnsNull()
    {
        var result = _service.MarkItemUsed(Guid.NewGuid());

        Assert.Null(result);
    }

    [Fact]
    public void MarkItemUsed_UpdatesModifiedAt()
    {
        var id = Guid.NewGuid();
        var originalTime = DateTime.UtcNow.AddHours(-1);
        var item = new ClipboardItem { Id = id, Content = "Test", Type = ClipboardContentType.Text, ModifiedAt = originalTime };
        _repository.ItemsById[id] = item;

        _service.MarkItemUsed(id);

        Assert.True(item.ModifiedAt > originalTime);
    }

    [Fact]
    public void MarkItemUsed_ReturnsUpdatedItem()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "Hello", Type = ClipboardContentType.Text, PasteCount = 5 };
        _repository.ItemsById[id] = item;

        var result = _service.MarkItemUsed(id);

        Assert.NotNull(result);
        Assert.Equal("Hello", result.Content);
        Assert.Equal(6, result.PasteCount);
    }

    #endregion

    #region AddImage Tests

    [Fact]
    public void AddImage_WithNullData_DoesNotSave()
    {
        _service.AddImage(null, "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void AddImage_WithShortDibData_DoesNotSave()
    {
        // DIB data less than 40 bytes is invalid — ConvertDibToBmp returns null
        _service.AddImage(new byte[10], "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void AddImage_DuringPasteWindow_IsIgnored()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "img.png", Type = ClipboardContentType.Image };
        _repository.ItemsById[id] = item;
        _service.PasteIgnoreWindowMs = 2000;

        _service.NotifyPasteInitiated(id);

        // Should be ignored during paste window
        _service.AddImage(new byte[100], "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    #endregion

    #region GetHistory Tests

    [Fact]
    public void GetHistory_DelegatesToGetHistoryAdvanced()
    {
        var result = _service.GetHistory(10, 0, "query", true);

        Assert.NotNull(result);
    }

    [Fact]
    public void GetHistory_DefaultParameters_ReturnsEmptyFromStub()
    {
        var result = _service.GetHistory();

        Assert.Empty(result);
    }

    #endregion

    #region NotifyPasteInitiated Advanced Tests

    [Fact]
    public void NotifyPasteInitiated_DifferentContent_NotIgnoredAfterTimeWindow()
    {
        _service.PasteIgnoreWindowMs = 50;
        var itemId = Guid.NewGuid();
        var item = new ClipboardItem { Id = itemId, Content = "Original", Type = ClipboardContentType.Text };
        _repository.ItemsById[itemId] = item;

        _service.NotifyPasteInitiated(itemId);
        Thread.Sleep(100);

        // Different content should be added after initial window expires
        _service.AddText("Totally different content", ClipboardContentType.Text, "TestApp");

        Assert.Single(_repository.SavedItems);
        Assert.Equal("Totally different content", _repository.SavedItems[0].Content);
    }

    [Fact]
    public void NotifyPasteInitiated_WithNonExistentItem_StillIgnoresWithinWindow()
    {
        _service.PasteIgnoreWindowMs = 500;
        var itemId = Guid.NewGuid();
        // item not added to repository — GetById returns null

        _service.NotifyPasteInitiated(itemId);

        _service.AddText("New text", ClipboardContentType.Text, "TestApp");

        Assert.Empty(_repository.SavedItems);
    }

    #endregion

    #region AddText Edge Cases

    [Fact]
    public void AddText_FiresOnItemAddedWithCorrectAppSource()
    {
        ClipboardItem? addedItem = null;
        _service.OnItemAdded += item => addedItem = item;

        _service.AddText("Test", ClipboardContentType.Text, "Chrome.exe");

        Assert.NotNull(addedItem);
        Assert.Equal("Chrome.exe", addedItem.AppSource);
    }

    [Fact]
    public void AddText_WithNullAppSource_SavesNullAppSource()
    {
        _service.AddText("Test", ClipboardContentType.Text, null);

        Assert.Single(_repository.SavedItems);
        Assert.Null(_repository.SavedItems[0].AppSource);
    }

    [Fact]
    public void AddText_WithAllContentTypes_SavesCorrectType()
    {
        _service.AddText("test", ClipboardContentType.Text, "App");
        _service.AddText("https://x.com", ClipboardContentType.Link, "App");

        Assert.Equal(2, _repository.SavedItems.Count);
        Assert.Equal(ClipboardContentType.Text, _repository.SavedItems[0].Type);
        Assert.Equal(ClipboardContentType.Link, _repository.SavedItems[1].Type);
    }

    [Fact]
    public void AddText_GeneratesUniqueIds()
    {
        _service.AddText("First", ClipboardContentType.Text, "App");
        _service.AddText("Second", ClipboardContentType.Text, "App");

        Assert.Equal(2, _repository.SavedItems.Count);
        Assert.NotEqual(_repository.SavedItems[0].Id, _repository.SavedItems[1].Id);
        Assert.NotEqual(Guid.Empty, _repository.SavedItems[0].Id);
    }

    #endregion

    #region AddFiles Edge Cases

    [Fact]
    public void AddFiles_FiresOnItemAddedEvent()
    {
        ClipboardItem? addedItem = null;
        _service.OnItemAdded += item => addedItem = item;

        var testFile = Path.Combine(_basePath, "event_test.txt");
        File.WriteAllText(testFile, "content");
        _service.AddFiles(new Collection<string> { testFile }, ClipboardContentType.File, "Explorer");

        Assert.NotNull(addedItem);
    }

    [Fact]
    public void AddFiles_DuringPasteWindow_IsIgnored()
    {
        var id = Guid.NewGuid();
        var item = new ClipboardItem { Id = id, Content = "file.txt", Type = ClipboardContentType.File };
        _repository.ItemsById[id] = item;
        _service.PasteIgnoreWindowMs = 2000;

        _service.NotifyPasteInitiated(id);

        _service.AddFiles(new Collection<string> { "file.txt" }, ClipboardContentType.File, "Explorer");

        Assert.Empty(_repository.SavedItems);
    }

    [Fact]
    public void AddFiles_WithFileSize_IncludesFileSizeInMetadata()
    {
        var testFile = Path.Combine(_basePath, "sized_file.txt");
        File.WriteAllText(testFile, "Hello, World!");

        _service.AddFiles(new Collection<string> { testFile }, ClipboardContentType.File, "Explorer");

        Assert.Single(_repository.SavedItems);
        Assert.Contains("file_size", _repository.SavedItems[0].Metadata!, StringComparison.Ordinal);
    }

    #endregion

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

        public List<Guid> DeletedIds { get; } = new();
        public void Delete(Guid id) => DeletedIds.Add(id);

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
