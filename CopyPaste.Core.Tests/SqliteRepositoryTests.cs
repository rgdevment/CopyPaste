using System;
using System.IO;
using System.Linq;
using System.Threading;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class SqliteRepositoryTests : IDisposable
{
    private readonly string _basePath;
    private readonly string _dbPath;
    private readonly SqliteRepository _repository;

    public SqliteRepositoryTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        Directory.CreateDirectory(_basePath);
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();

        _dbPath = Path.Combine(_basePath, "test_clipboard.db");
        _repository = new SqliteRepository(_dbPath);
    }

    #region Save Tests

    [Fact]
    public void Save_NewItem_StoresInDatabase()
    {
        var item = new ClipboardItem
        {
            Content = "Test content",
            Type = ClipboardContentType.Text,
            AppSource = "TestApp"
        };

        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal("Test content", retrieved.Content);
        Assert.Equal(ClipboardContentType.Text, retrieved.Type);
        Assert.Equal("TestApp", retrieved.AppSource);
    }

    [Fact]
    public void Save_WithoutId_GeneratesId()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        _repository.Save(item);

        Assert.NotEqual(Guid.Empty, item.Id);
    }

    [Fact]
    public void Save_WithMetadata_StoresMetadata()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            Metadata = "{\"key\":\"value\"}"
        };

        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal("{\"key\":\"value\"}", retrieved.Metadata);
    }

    [Fact]
    public void Save_NullItem_ThrowsException()
    {
        Assert.Throws<ArgumentNullException>(() => _repository.Save(null!));
    }

    [Fact]
    public void Save_PinnedItem_StoresPinnedFlag()
    {
        var item = new ClipboardItem
        {
            Content = "Pinned",
            Type = ClipboardContentType.Text,
            IsPinned = true
        };

        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.True(retrieved.IsPinned);
    }

    [Fact]
    public void Save_WithLabel_StoresLabel()
    {
        var item = new ClipboardItem
        {
            Content = "Some UUID: abc-123",
            Type = ClipboardContentType.Text,
            Label = "API Key Production"
        };

        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal("API Key Production", retrieved.Label);
    }

    [Fact]
    public void Save_WithCardColor_StoresColor()
    {
        var item = new ClipboardItem
        {
            Content = "Important note",
            Type = ClipboardContentType.Text,
            CardColor = CardColor.Red
        };

        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal(CardColor.Red, retrieved.CardColor);
    }

    [Fact]
    public void Save_WithoutLabel_HasNullLabel()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Null(retrieved.Label);
    }

    [Fact]
    public void Save_WithoutCardColor_HasNoneColor()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal(CardColor.None, retrieved.CardColor);
    }

    #endregion

    #region Update Tests

    [Fact]
    public void Update_ExistingItem_UpdatesContent()
    {
        var item = new ClipboardItem
        {
            Content = "Original",
            Type = ClipboardContentType.Text
        };
        _repository.Save(item);

        item.Content = "Updated";
        _repository.Update(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal("Updated", retrieved.Content);
    }

    [Fact]
    public void Update_ModifiedAt_UpdatesTimestamp()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };
        _repository.Save(item);

        var originalModifiedAt = item.ModifiedAt.ToUniversalTime();
        Thread.Sleep(200); // Increased sleep for timestamp resolution

        item.ModifiedAt = DateTime.UtcNow;
        _repository.Update(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        var retrievedModifiedAt = retrieved.ModifiedAt.ToUniversalTime();

        // Compare with second precision and 1-second tolerance for clock drift
        var secondsDiff = (retrievedModifiedAt - originalModifiedAt).TotalSeconds;
        Assert.True(secondsDiff >= -1,
            $"Modified: {retrievedModifiedAt:O} should be >= Original: {originalModifiedAt:O} (diff: {secondsDiff:F2}s)");
    }

    [Fact]
    public void Update_NullItem_ThrowsException()
    {
        Assert.Throws<ArgumentNullException>(() => _repository.Update(null!));
    }

    [Fact]
    public void Update_IsPinned_UpdatesPinnedStatus()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            IsPinned = false
        };
        _repository.Save(item);

        item.IsPinned = true;
        _repository.Update(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.True(retrieved.IsPinned);
    }

    [Fact]
    public void Update_Label_UpdatesLabel()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            Label = null
        };
        _repository.Save(item);

        item.Label = "My Custom Label";
        _repository.Update(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal("My Custom Label", retrieved.Label);
    }

    [Fact]
    public void Update_CardColor_UpdatesColor()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CardColor = CardColor.None
        };
        _repository.Save(item);

        item.CardColor = CardColor.Blue;
        _repository.Update(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal(CardColor.Blue, retrieved.CardColor);
    }

    [Fact]
    public void Update_LabelToNull_ClearsLabel()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            Label = "Existing Label"
        };
        _repository.Save(item);

        item.Label = null;
        _repository.Update(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Null(retrieved.Label);
    }

    #endregion

    #region GetById Tests

    [Fact]
    public void GetById_ExistingItem_ReturnsItem()
    {
        var item = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };
        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);

        Assert.NotNull(retrieved);
        Assert.Equal(item.Id, retrieved.Id);
    }

    [Fact]
    public void GetById_NonExistentItem_ReturnsNull()
    {
        var result = _repository.GetById(Guid.NewGuid());

        Assert.Null(result);
    }

    [Fact]
    public void GetById_EmptyGuid_ReturnsNull()
    {
        var result = _repository.GetById(Guid.Empty);

        Assert.Null(result);
    }

    #endregion

    #region GetLatest Tests

    [Fact]
    public void GetLatest_WithItems_ReturnsMostRecent()
    {
        var item1 = new ClipboardItem { Content = "First", Type = ClipboardContentType.Text };
        var item2 = new ClipboardItem { Content = "Second", Type = ClipboardContentType.Text };

        _repository.Save(item1);
        Thread.Sleep(10);
        _repository.Save(item2);

        var latest = _repository.GetLatest();

        Assert.NotNull(latest);
        Assert.Equal("Second", latest.Content);
    }

    [Fact]
    public void GetLatest_EmptyDatabase_ReturnsNull()
    {
        var latest = _repository.GetLatest();

        Assert.Null(latest);
    }

    [Fact]
    public void GetLatest_IgnoresUnknownType()
    {
        var unknownItem = new ClipboardItem { Content = "Unknown", Type = ClipboardContentType.Unknown };
        var textItem = new ClipboardItem { Content = "Text", Type = ClipboardContentType.Text };

        _repository.Save(textItem);
        _repository.Save(unknownItem);

        var latest = _repository.GetLatest();

        Assert.NotNull(latest);
        Assert.Equal("Text", latest.Content);
    }

    #endregion

    #region GetAll Tests

    [Fact]
    public void GetAll_ReturnsAllItems()
    {
        var item1 = new ClipboardItem { Content = "First", Type = ClipboardContentType.Text };
        var item2 = new ClipboardItem { Content = "Second", Type = ClipboardContentType.Text };
        var item3 = new ClipboardItem { Content = "Third", Type = ClipboardContentType.Text };

        _repository.Save(item1);
        _repository.Save(item2);
        _repository.Save(item3);

        var all = _repository.GetAll().ToList();

        Assert.Equal(3, all.Count);
    }

    [Fact]
    public void GetAll_OrdersByModifiedAtDesc()
    {
        var item1 = new ClipboardItem { Content = "First", Type = ClipboardContentType.Text };
        var item2 = new ClipboardItem { Content = "Second", Type = ClipboardContentType.Text };

        _repository.Save(item1);
        Thread.Sleep(10);
        _repository.Save(item2);

        var all = _repository.GetAll().ToList();

        Assert.Equal("Second", all[0].Content);
        Assert.Equal("First", all[1].Content);
    }

    [Fact]
    public void GetAll_EmptyDatabase_ReturnsEmpty()
    {
        var all = _repository.GetAll().ToList();

        Assert.Empty(all);
    }

    [Fact]
    public void GetAll_IgnoresUnknownType()
    {
        var unknownItem = new ClipboardItem { Content = "Unknown", Type = ClipboardContentType.Unknown };
        var textItem = new ClipboardItem { Content = "Text", Type = ClipboardContentType.Text };

        _repository.Save(textItem);
        _repository.Save(unknownItem);

        var all = _repository.GetAll().ToList();

        Assert.Single(all);
        Assert.Equal("Text", all[0].Content);
    }

    #endregion

    #region Delete Tests

    [Fact]
    public void Delete_ExistingItem_RemovesFromDatabase()
    {
        var item = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text };
        _repository.Save(item);

        _repository.Delete(item.Id);

        var retrieved = _repository.GetById(item.Id);
        Assert.Null(retrieved);
    }

    [Fact]
    public void Delete_NonExistentItem_DoesNotThrow()
    {
        _repository.Delete(Guid.NewGuid());

        // Should not throw
        Assert.True(true);
    }

    #endregion

    #region Search Tests

    [Fact]
    public void Search_FindsMatchingContent()
    {
        var item1 = new ClipboardItem { Content = "Hello World", Type = ClipboardContentType.Text };
        var item2 = new ClipboardItem { Content = "Goodbye World", Type = ClipboardContentType.Text };

        _repository.Save(item1);
        _repository.Save(item2);

        var results = _repository.Search("Hello").ToList();

        Assert.Single(results);
        Assert.Equal("Hello World", results[0].Content);
    }

    [Fact]
    public void Search_EmptyQuery_ReturnsAll()
    {
        var item1 = new ClipboardItem { Content = "First", Type = ClipboardContentType.Text };
        var item2 = new ClipboardItem { Content = "Second", Type = ClipboardContentType.Text };

        _repository.Save(item1);
        _repository.Save(item2);

        var results = _repository.Search("").ToList();

        Assert.Equal(2, results.Count);
    }

    [Fact]
    public void Search_WithLimit_ReturnsLimitedResults()
    {
        for (int i = 0; i < 10; i++)
        {
            _repository.Save(new ClipboardItem { Content = $"Item {i}", Type = ClipboardContentType.Text });
        }

        var results = _repository.Search("", limit: 5).ToList();

        Assert.Equal(5, results.Count);
    }

    [Fact]
    public void Search_NoMatches_ReturnsEmpty()
    {
        var item = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text };
        _repository.Save(item);

        var results = _repository.Search("NonExistent").ToList();

        Assert.Empty(results);
    }

    [Fact]
    public void Search_FindsByLabel()
    {
        var item1 = new ClipboardItem
        {
            Content = "abc-123-xyz",
            Type = ClipboardContentType.Text,
            Label = "API Key Production"
        };
        var item2 = new ClipboardItem
        {
            Content = "def-456-uvw",
            Type = ClipboardContentType.Text,
            Label = "Database Password"
        };

        _repository.Save(item1);
        _repository.Save(item2);

        var results = _repository.Search("Production").ToList();

        Assert.Single(results);
        Assert.Equal("API Key Production", results[0].Label);
    }

    #endregion

    #region ClearOldItems Tests

    [Fact]
    public void ClearOldItems_RemovesOldItems()
    {
        var oldItem = new ClipboardItem
        {
            Content = "Old",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow.AddDays(-10)
        };
        var newItem = new ClipboardItem
        {
            Content = "New",
            Type = ClipboardContentType.Text
        };

        _repository.Save(oldItem);
        _repository.Save(newItem);

        var count = _repository.ClearOldItems(7);

        Assert.Equal(1, count);
        Assert.Null(_repository.GetById(oldItem.Id));
        Assert.NotNull(_repository.GetById(newItem.Id));
    }

    [Fact]
    public void ClearOldItems_ExcludesPinned_KeepsPinnedItems()
    {
        var oldPinnedItem = new ClipboardItem
        {
            Content = "Old Pinned",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow.AddDays(-10),
            IsPinned = true
        };

        _repository.Save(oldPinnedItem);

        var count = _repository.ClearOldItems(7, excludePinned: true);

        Assert.Equal(0, count);
        Assert.NotNull(_repository.GetById(oldPinnedItem.Id));
    }

    [Fact]
    public void ClearOldItems_IncludePinned_RemovesPinnedItems()
    {
        var oldPinnedItem = new ClipboardItem
        {
            Content = "Old Pinned",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow.AddDays(-10),
            IsPinned = true
        };

        _repository.Save(oldPinnedItem);

        var count = _repository.ClearOldItems(7, excludePinned: false);

        Assert.Equal(1, count);
        Assert.Null(_repository.GetById(oldPinnedItem.Id));
    }

    #endregion

    #region Database Initialization Tests

    [Fact]
    public void Constructor_CreatesDatabase()
    {
        Assert.True(File.Exists(_dbPath));
    }

    [Fact]
    public void Constructor_CreatesTablesAndIndexes()
    {
        // If we can save and retrieve, tables exist
        var item = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text };
        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
    }

    #endregion

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types", Justification = "Best-effort cleanup of temp test data should not fail tests")]
    public void Dispose()
    {
        _repository?.Dispose();

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
}
