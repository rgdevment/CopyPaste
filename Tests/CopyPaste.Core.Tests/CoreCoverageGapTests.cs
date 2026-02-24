using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using Microsoft.Data.Sqlite;
using Xunit;

namespace CopyPaste.Core.Tests;

// ─────────────────────────────────────────────────────────────────────────────
// SqliteRepository – uncovered edge cases
// ─────────────────────────────────────────────────────────────────────────────

public sealed class SqliteRepositoryEdgeCaseTests : IDisposable
{
    private readonly string _basePath;
    private readonly string _dbPath;
    private readonly SqliteRepository _repository;

    public SqliteRepositoryEdgeCaseTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        Directory.CreateDirectory(_basePath);
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();

        _dbPath = Path.Combine(_basePath, "edge.db");
        _repository = new SqliteRepository(_dbPath);
    }

    public void Dispose()
    {
        _repository.Dispose();
        SqliteConnection.ClearAllPools();
        try
        {
            if (Directory.Exists(_basePath))
                Directory.Delete(_basePath, recursive: true);
        }
        catch { }
    }

    private static ClipboardItem MakeItem(string content = "test",
        ClipboardContentType type = ClipboardContentType.Text,
        bool isPinned = false,
        DateTime? createdAt = null)
    {
        var now = createdAt ?? DateTime.UtcNow;
        return new ClipboardItem { Content = content, Type = type, IsPinned = isPinned, CreatedAt = now, ModifiedAt = now };
    }

    [Fact]
    public void FindByContentHash_WithNullHash_ReturnsNull()
    {
        var result = _repository.FindByContentHash(null!);
        Assert.Null(result);
    }

    [Fact]
    public void FindByContentHash_WithEmptyHash_ReturnsNull()
    {
        var result = _repository.FindByContentHash(string.Empty);
        Assert.Null(result);
    }

    [Fact]
    public void ClearOldItems_WithMoreThan50OldItems_DeletesAllAndRunsVacuum()
    {
        var oldDate = DateTime.UtcNow.AddDays(-30);
        for (int i = 0; i < 52; i++)
        {
            _repository.Save(MakeItem($"old {i}", createdAt: oldDate));
        }

        var deleted = _repository.ClearOldItems(days: 7);

        Assert.Equal(52, deleted);
    }

    [Fact]
    public void ClearOldItems_ExcludePinnedFalse_DeletesPinnedItems()
    {
        var oldDate = DateTime.UtcNow.AddDays(-30);
        _repository.Save(MakeItem("pinned old", isPinned: true, createdAt: oldDate));
        _repository.Save(MakeItem("unpinned old", isPinned: false, createdAt: oldDate));

        var deleted = _repository.ClearOldItems(days: 7, excludePinned: false);

        Assert.Equal(2, deleted);
    }

    [Fact]
    public void GetLatest_EmptyRepository_ReturnsNull()
    {
        var latest = _repository.GetLatest();
        Assert.Null(latest);
    }

    [Fact]
    public void Search_WithNullQuery_ReturnsAllItems()
    {
        _repository.Save(MakeItem("item1"));
        _repository.Save(MakeItem("item2"));

        var results = _repository.Search(null!, limit: 100, skip: 0);

        Assert.Equal(2, results.Count());
    }

    [Fact]
    public void Search_WithWhitespaceQuery_ReturnsAllItems()
    {
        _repository.Save(MakeItem("item1"));

        var results = _repository.Search("   ", limit: 100, skip: 0);

        Assert.Single(results);
    }

    [Fact]
    public void Search_WithTextQuery_FindsMatchingItems()
    {
        _repository.Save(MakeItem("unique_edge_term_99999"));
        _repository.Save(MakeItem("something unrelated"));

        var results = _repository.Search("unique_edge_term_99999", limit: 100, skip: 0);

        Assert.Single(results);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CleanupService – LoadLastCleanupDate uncovered branches
// ─────────────────────────────────────────────────────────────────────────────

public sealed class CleanupServiceCoverageTests : IDisposable
{
    private readonly string _basePath;

    public CleanupServiceCoverageTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        Directory.CreateDirectory(_basePath);
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
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

    private static string GetCleanupFilePath()
    {
        var directory = Path.GetDirectoryName(StorageConfig.DatabasePath)!;
        return Path.Combine(directory, "last_cleanup.txt");
    }

    [Fact]
    public void RunCleanupIfNeeded_WithInvalidDateInFile_RunsCleanupAnyway()
    {
        var repo = new CleanupStub();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        File.WriteAllText(GetCleanupFilePath(), "this-is-not-a-date");

        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
    }

    [Fact]
    public void RunCleanupIfNeeded_WithYesterdayDate_RunsCleanup()
    {
        var repo = new CleanupStub();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        var yesterday = DateTime.UtcNow.AddDays(-1);
        File.WriteAllText(GetCleanupFilePath(), yesterday.ToString("O"));

        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
    }

    private sealed class CleanupStub : IClipboardRepository
    {
        public int ClearCalls;

        public void Save(ClipboardItem item) { }
        public void Update(ClipboardItem item) { }
        public ClipboardItem? GetById(Guid id) => null;
        public ClipboardItem? GetLatest() => null;
        public ClipboardItem? FindByContentAndType(string c, ClipboardContentType t) => null;
        public ClipboardItem? FindByContentHash(string h) => null;
        public IEnumerable<ClipboardItem> GetAll() => [];
        public void Delete(Guid id) { }
        public IEnumerable<ClipboardItem> Search(string q, int l = 50, int s = 0) => [];
        public IEnumerable<ClipboardItem> SearchAdvanced(
            string? q, IReadOnlyCollection<ClipboardContentType>? t,
            IReadOnlyCollection<CardColor>? c, bool? p, int l, int s) => [];

        public int ClearOldItems(int days, bool excludePinned = true)
        {
            ClearCalls++;
            return 0;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UpdateChecker – IsVersionDismissed (private static method via reflection)
// ─────────────────────────────────────────────────────────────────────────────

public sealed class UpdateCheckerCoverageTests : IDisposable
{
    private readonly string _basePath;

    private static readonly MethodInfo _isVersionDismissed =
        typeof(UpdateChecker).GetMethod("IsVersionDismissed",
            BindingFlags.NonPublic | BindingFlags.Static)!;

    public UpdateCheckerCoverageTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        Directory.CreateDirectory(_basePath);
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
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

    private static bool IsVersionDismissed(string version) =>
        (bool)_isVersionDismissed.Invoke(null, new object[] { version })!;

    [Fact]
    public void IsVersionDismissed_WhenNoDismissedFile_ReturnsFalse()
    {
        var result = IsVersionDismissed("1.0.0");
        Assert.False(result);
    }

    [Fact]
    public void IsVersionDismissed_AfterDismissingThatVersion_ReturnsTrue()
    {
        UpdateChecker.DismissVersion("2.5.0");
        Assert.True(IsVersionDismissed("2.5.0"));
    }

    [Fact]
    public void IsVersionDismissed_WhenDifferentVersionDismissed_ReturnsFalse()
    {
        UpdateChecker.DismissVersion("2.5.0");
        Assert.False(IsVersionDismissed("3.0.0"));
    }

    [Fact]
    public void IsVersionDismissed_CaseInsensitiveMatch_ReturnsTrue()
    {
        UpdateChecker.DismissVersion("v1.2.3");
        Assert.True(IsVersionDismissed("V1.2.3"));
    }
}
