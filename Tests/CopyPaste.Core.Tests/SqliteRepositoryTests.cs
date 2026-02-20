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

    private static ClipboardItem CreateItem(
        string content = "test",
        ClipboardContentType type = ClipboardContentType.Text,
        bool isPinned = false,
        CardColor color = CardColor.None,
        string? label = null,
        string? contentHash = null,
        DateTime? createdAt = null,
        DateTime? modifiedAt = null)
    {
        return new ClipboardItem
        {
            Content = content,
            Type = type,
            IsPinned = isPinned,
            CardColor = color,
            Label = label,
            ContentHash = contentHash,
            CreatedAt = createdAt ?? DateTime.UtcNow,
            ModifiedAt = modifiedAt ?? DateTime.UtcNow
        };
    }

    #region Save and GetById

    [Fact]
    public void Save_AssignsGuidIfEmpty()
    {
        var item = CreateItem();
        item.Id = Guid.Empty;

        _repository.Save(item);

        Assert.NotEqual(Guid.Empty, item.Id);
    }

    [Fact]
    public void Save_PersistsItem()
    {
        var item = CreateItem("hello");

        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal("hello", retrieved.Content);
    }

    [Fact]
    public void GetById_ExistingItem_ReturnsItem()
    {
        var item = CreateItem();
        _repository.Save(item);

        var retrieved = _repository.GetById(item.Id);

        Assert.NotNull(retrieved);
        Assert.Equal(item.Id, retrieved.Id);
    }

    [Fact]
    public void GetById_NonExistent_ReturnsNull()
    {
        var result = _repository.GetById(Guid.NewGuid());

        Assert.Null(result);
    }

    [Fact]
    public void Save_AllFieldsPersisted()
    {
        var id = Guid.NewGuid();
        var createdAt = new DateTime(2024, 1, 15, 10, 30, 0, DateTimeKind.Utc);
        var modifiedAt = new DateTime(2024, 6, 20, 14, 45, 0, DateTimeKind.Utc);
        var item = new ClipboardItem
        {
            Id = id,
            Content = "full content",
            Type = ClipboardContentType.Link,
            AppSource = "TestApp",
            IsPinned = true,
            Metadata = "{\"key\":\"value\"}",
            Label = "my label",
            CardColor = CardColor.Blue,
            PasteCount = 7,
            ContentHash = "abc123hash",
            CreatedAt = createdAt,
            ModifiedAt = modifiedAt
        };

        _repository.Save(item);

        var r = _repository.GetById(id);
        Assert.NotNull(r);
        Assert.Equal(id, r.Id);
        Assert.Equal("full content", r.Content);
        Assert.Equal(ClipboardContentType.Link, r.Type);
        Assert.Equal("TestApp", r.AppSource);
        Assert.True(r.IsPinned);
        Assert.Equal("{\"key\":\"value\"}", r.Metadata);
        Assert.Equal("my label", r.Label);
        Assert.Equal(CardColor.Blue, r.CardColor);
        Assert.Equal(7, r.PasteCount);
        Assert.Equal("abc123hash", r.ContentHash);
        Assert.Equal(createdAt, r.CreatedAt.ToUniversalTime());
        Assert.Equal(modifiedAt, r.ModifiedAt.ToUniversalTime());
    }

    #endregion

    #region Update

    [Fact]
    public void Update_ModifiesExistingItem()
    {
        var item = CreateItem("original");
        _repository.Save(item);

        item.Content = "updated";
        _repository.Update(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.Equal("updated", retrieved.Content);
    }

    [Fact]
    public void Update_ChangesIsPinned()
    {
        var item = CreateItem(isPinned: false);
        _repository.Save(item);

        item.IsPinned = true;
        _repository.Update(item);

        var retrieved = _repository.GetById(item.Id);
        Assert.NotNull(retrieved);
        Assert.True(retrieved.IsPinned);
    }

    #endregion

    #region GetLatest

    [Fact]
    public void GetLatest_ReturnsNewestByModifiedAt()
    {
        var older = CreateItem("older", modifiedAt: DateTime.UtcNow.AddSeconds(-10));
        var newer = CreateItem("newer", modifiedAt: DateTime.UtcNow);
        _repository.Save(older);
        _repository.Save(newer);

        var latest = _repository.GetLatest();

        Assert.NotNull(latest);
        Assert.Equal("newer", latest.Content);
    }

    [Fact]
    public void GetLatest_EmptyDb_ReturnsNull()
    {
        var result = _repository.GetLatest();

        Assert.Null(result);
    }

    [Fact]
    public void GetLatest_ExcludesUnknownType()
    {
        var textItem = CreateItem("text item");
        var unknownItem = CreateItem("unknown item", type: ClipboardContentType.Unknown, modifiedAt: DateTime.UtcNow.AddSeconds(5));
        _repository.Save(textItem);
        _repository.Save(unknownItem);

        var latest = _repository.GetLatest();

        Assert.NotNull(latest);
        Assert.Equal("text item", latest.Content);
    }

    #endregion

    #region FindByContentHash

    [Fact]
    public void FindByContentHash_ExistingHash_ReturnsItem()
    {
        var item = CreateItem(contentHash: "hash_xyz");
        _repository.Save(item);

        var result = _repository.FindByContentHash("hash_xyz");

        Assert.NotNull(result);
        Assert.Equal(item.Id, result.Id);
    }

    [Fact]
    public void FindByContentHash_NoMatch_ReturnsNull()
    {
        var item = CreateItem(contentHash: "hash_abc");
        _repository.Save(item);

        var result = _repository.FindByContentHash("hash_not_found");

        Assert.Null(result);
    }

    [Fact]
    public void FindByContentHash_NullOrEmpty_ReturnsNull()
    {
        var item = CreateItem(contentHash: "some_hash");
        _repository.Save(item);

        Assert.Null(_repository.FindByContentHash(null!));
        Assert.Null(_repository.FindByContentHash(string.Empty));
    }

    #endregion

    #region FindByContentAndType

    [Fact]
    public void FindByContentAndType_Match_ReturnsItem()
    {
        var item = CreateItem("find me", ClipboardContentType.Text);
        _repository.Save(item);

        var result = _repository.FindByContentAndType("find me", ClipboardContentType.Text);

        Assert.NotNull(result);
        Assert.Equal(item.Id, result.Id);
    }

    [Fact]
    public void FindByContentAndType_NoMatch_ReturnsNull()
    {
        var item = CreateItem("something", ClipboardContentType.Text);
        _repository.Save(item);

        var result = _repository.FindByContentAndType("not found", ClipboardContentType.Text);

        Assert.Null(result);
    }

    [Fact]
    public void FindByContentAndType_SameContentDifferentType_ReturnsNull()
    {
        var item = CreateItem("shared content", ClipboardContentType.Text);
        _repository.Save(item);

        var result = _repository.FindByContentAndType("shared content", ClipboardContentType.Link);

        Assert.Null(result);
    }

    #endregion

    #region GetAll

    [Fact]
    public void GetAll_ReturnsAllItems()
    {
        _repository.Save(CreateItem("a"));
        _repository.Save(CreateItem("b"));
        _repository.Save(CreateItem("c"));

        var all = _repository.GetAll().ToList();

        Assert.Equal(3, all.Count);
    }

    [Fact]
    public void GetAll_ExcludesUnknownType()
    {
        _repository.Save(CreateItem("visible", ClipboardContentType.Text));
        _repository.Save(CreateItem("hidden", ClipboardContentType.Unknown));

        var all = _repository.GetAll().ToList();

        Assert.Single(all);
        Assert.Equal("visible", all[0].Content);
    }

    [Fact]
    public void GetAll_OrderedByModifiedAtDesc()
    {
        var first = CreateItem("first", modifiedAt: DateTime.UtcNow.AddSeconds(-20));
        var second = CreateItem("second", modifiedAt: DateTime.UtcNow.AddSeconds(-10));
        var third = CreateItem("third", modifiedAt: DateTime.UtcNow);
        _repository.Save(first);
        _repository.Save(second);
        _repository.Save(third);

        var all = _repository.GetAll().ToList();

        Assert.Equal("third", all[0].Content);
        Assert.Equal("second", all[1].Content);
        Assert.Equal("first", all[2].Content);
    }

    #endregion

    #region Delete

    [Fact]
    public void Delete_RemovesItem()
    {
        var item = CreateItem();
        _repository.Save(item);

        _repository.Delete(item.Id);

        Assert.Null(_repository.GetById(item.Id));
    }

    [Fact]
    public void Delete_NonExistent_DoesNotThrow()
    {
        var ex = Record.Exception(() => _repository.Delete(Guid.NewGuid()));

        Assert.Null(ex);
    }

    #endregion

    #region ClearOldItems

    [Fact]
    public void ClearOldItems_DeletesOlderThanDays()
    {
        var oldItem = CreateItem("old", createdAt: DateTime.UtcNow.AddDays(-10));
        var newItem = CreateItem("new");
        _repository.Save(oldItem);
        _repository.Save(newItem);

        _repository.ClearOldItems(7);

        Assert.Null(_repository.GetById(oldItem.Id));
        Assert.NotNull(_repository.GetById(newItem.Id));
    }

    [Fact]
    public void ClearOldItems_ExcludesPinned_WhenTrue()
    {
        var oldPinned = CreateItem("old pinned", isPinned: true, createdAt: DateTime.UtcNow.AddDays(-10));
        _repository.Save(oldPinned);

        var deleted = _repository.ClearOldItems(7, excludePinned: true);

        Assert.Equal(0, deleted);
        Assert.NotNull(_repository.GetById(oldPinned.Id));
    }

    [Fact]
    public void ClearOldItems_IncludesPinned_WhenFalse()
    {
        var oldPinned = CreateItem("old pinned", isPinned: true, createdAt: DateTime.UtcNow.AddDays(-10));
        _repository.Save(oldPinned);

        var deleted = _repository.ClearOldItems(7, excludePinned: false);

        Assert.Equal(1, deleted);
        Assert.Null(_repository.GetById(oldPinned.Id));
    }

    [Fact]
    public void ClearOldItems_ReturnsDeletedCount()
    {
        _repository.Save(CreateItem("old1", createdAt: DateTime.UtcNow.AddDays(-10)));
        _repository.Save(CreateItem("old2", createdAt: DateTime.UtcNow.AddDays(-15)));
        _repository.Save(CreateItem("new"));

        var count = _repository.ClearOldItems(7);

        Assert.Equal(2, count);
    }

    #endregion

    #region Search

    [Fact]
    public void Search_EmptyQuery_ReturnsAllOrdered()
    {
        _repository.Save(CreateItem("alpha"));
        _repository.Save(CreateItem("beta"));

        var results = _repository.Search(string.Empty).ToList();

        Assert.Equal(2, results.Count);
    }

    [Fact]
    public void Search_WithQuery_FindsMatchingContent()
    {
        _repository.Save(CreateItem("Hello World"));
        _repository.Save(CreateItem("Goodbye"));

        var results = _repository.Search("Hello").ToList();

        Assert.Single(results);
        Assert.Equal("Hello World", results[0].Content);
    }

    [Fact]
    public void Search_RespectsLimitAndSkip()
    {
        for (int i = 0; i < 10; i++)
            _repository.Save(CreateItem($"item {i}"));

        var page1 = _repository.Search(string.Empty, limit: 3, skip: 0).ToList();
        var page2 = _repository.Search(string.Empty, limit: 3, skip: 3).ToList();

        Assert.Equal(3, page1.Count);
        Assert.Equal(3, page2.Count);
        Assert.DoesNotContain(page2, i => page1.Any(j => j.Id == i.Id));
    }

    [Fact]
    public void Search_NoMatch_ReturnsEmpty()
    {
        _repository.Save(CreateItem("some text"));

        var results = _repository.Search("zzznomatch").ToList();

        Assert.Empty(results);
    }

    #endregion

    #region SearchAdvanced

    [Fact]
    public void SearchAdvanced_NoFilters_ReturnsAll()
    {
        _repository.Save(CreateItem("first"));
        _repository.Save(CreateItem("second"));

        var results = _repository.SearchAdvanced(null, null, null, null, 100, 0).ToList();

        Assert.Equal(2, results.Count);
    }

    [Fact]
    public void SearchAdvanced_TypeFilter_FiltersCorrectly()
    {
        _repository.Save(CreateItem("text item", ClipboardContentType.Text));
        _repository.Save(CreateItem("link item", ClipboardContentType.Link));
        _repository.Save(CreateItem("image item", ClipboardContentType.Image));

        var results = _repository.SearchAdvanced(
            null,
            new[] { ClipboardContentType.Link },
            null,
            null,
            100,
            0).ToList();

        Assert.Single(results);
        Assert.Equal("link item", results[0].Content);
    }

    [Fact]
    public void SearchAdvanced_ColorFilter_FiltersCorrectly()
    {
        _repository.Save(CreateItem("red item", color: CardColor.Red));
        _repository.Save(CreateItem("blue item", color: CardColor.Blue));
        _repository.Save(CreateItem("no color", color: CardColor.None));

        var results = _repository.SearchAdvanced(
            null,
            null,
            new[] { CardColor.Red },
            null,
            100,
            0).ToList();

        Assert.Single(results);
        Assert.Equal("red item", results[0].Content);
    }

    [Fact]
    public void SearchAdvanced_IsPinnedFilter_FiltersCorrectly()
    {
        _repository.Save(CreateItem("pinned", isPinned: true));
        _repository.Save(CreateItem("not pinned", isPinned: false));

        var results = _repository.SearchAdvanced(null, null, null, true, 100, 0).ToList();

        Assert.Single(results);
        Assert.Equal("pinned", results[0].Content);
    }

    [Fact]
    public void SearchAdvanced_TextQuery_FindsMatches()
    {
        _repository.Save(CreateItem("unique phrase here"));
        _repository.Save(CreateItem("something else"));

        var results = _repository.SearchAdvanced("unique phrase", null, null, null, 100, 0).ToList();

        Assert.Single(results);
        Assert.Equal("unique phrase here", results[0].Content);
    }

    [Fact]
    public void SearchAdvanced_CombinedFilters_WorksTogether()
    {
        _repository.Save(CreateItem("text red", ClipboardContentType.Text, color: CardColor.Red));
        _repository.Save(CreateItem("text blue", ClipboardContentType.Text, color: CardColor.Blue));
        _repository.Save(CreateItem("link red", ClipboardContentType.Link, color: CardColor.Red));

        var results = _repository.SearchAdvanced(
            null,
            new[] { ClipboardContentType.Text },
            new[] { CardColor.Red },
            null,
            100,
            0).ToList();

        Assert.Single(results);
        Assert.Equal("text red", results[0].Content);
    }

    #endregion

    #region Dispose

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        var basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        Directory.CreateDirectory(basePath);
        StorageConfig.SetBasePath(basePath);
        StorageConfig.Initialize();

        var dbPath = Path.Combine(basePath, "dispose_test.db");

        Exception? ex;
        using (var repo = new SqliteRepository(dbPath))
        {
            ex = Record.Exception(() =>
            {
                repo.Dispose();
                repo.Dispose();
            });
        }

        Assert.Null(ex);

        try { Directory.Delete(basePath, recursive: true); } catch { }
    }

    #endregion

    #region Database resilience

    [Fact]
    public void Constructor_CreatesNewDatabase()
    {
        Assert.True(File.Exists(_dbPath));
    }

    #endregion

    public void Dispose()
    {
        _repository?.Dispose();

        try
        {
            if (Directory.Exists(_basePath))
                Directory.Delete(_basePath, recursive: true);
        }
        catch
        {
            // Best-effort cleanup
        }
    }
}
