using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class CleanupServiceTests : IDisposable
{
    private readonly string _basePath;

    public CleanupServiceTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
    }

    #region Constructor Tests

    [Fact]
    public void Constructor_NullRepository_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() =>
            new CleanupService(null!, () => 7, startTimer: false));
    }

    [Fact]
    public void Constructor_NullGetRetentionDays_ThrowsArgumentNullException()
    {
        var repo = new StubClipboardRepository();
        Assert.Throws<ArgumentNullException>(() =>
            new CleanupService(repo, null!, startTimer: false));
    }

    [Fact]
    public void Constructor_WithTimerDisabled_DoesNotRunCleanupImmediately()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        Thread.Sleep(100);

        Assert.Equal(0, repo.ClearCalls);
    }

    #endregion

    #region Dispose Tests

    [Fact]
    public void Dispose_CanBeCalledOnce()
    {
        var repo = new StubClipboardRepository();
        var service = new CleanupService(repo, () => 7, startTimer: false);

        service.Dispose();

        Assert.True(true);
    }

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        var repo = new StubClipboardRepository();
        var service = new CleanupService(repo, () => 7, startTimer: false);

        service.Dispose();
        service.Dispose();
        service.Dispose();

        Assert.True(true);
    }

    #endregion

    #region RunCleanupIfNeeded Tests

    [Fact]
    public void RunCleanupIfNeeded_RetentionDaysZero_DoesNotCallRepository()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 0, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(0, repo.ClearCalls);
    }

    [Fact]
    public void RunCleanupIfNeeded_RetentionDaysNegative_DoesNotCallRepository()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => -1, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(0, repo.ClearCalls);
    }

    [Fact]
    public void RunCleanupIfNeeded_NoPreviousCleanup_CallsRepository()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
    }

    [Fact]
    public void RunCleanupIfNeeded_AfterDispose_DoesNothing()
    {
        var repo = new StubClipboardRepository();
        var service = new CleanupService(repo, () => 7, startTimer: false);
        service.Dispose();

        service.RunCleanupIfNeeded();

        Assert.Equal(0, repo.ClearCalls);
    }

    [Fact]
    public void RunCleanupIfNeeded_WritesLastCleanupFile()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.True(File.Exists(GetLastCleanupFilePath()));
    }

    [Fact]
    public void RunCleanupIfNeeded_SameDay_DoesNotCleanupTwice()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        service.RunCleanupIfNeeded();
        service.RunCleanupIfNeeded();
        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
    }

    #endregion

    private static string GetLastCleanupFilePath()
    {
        var directory = Path.GetDirectoryName(StorageConfig.DatabasePath)!;
        return Path.Combine(directory, "last_cleanup.txt");
    }

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
            // Best-effort cleanup for temp test data.
        }
    }

    private sealed class StubClipboardRepository : IClipboardRepository
    {
        public int ClearCalls { get; private set; }
        public int LastRetentionDays { get; private set; }

        public int ClearOldItems(int days, bool excludePinned = true)
        {
            ClearCalls++;
            LastRetentionDays = days;
            return 0;
        }

        public void Delete(Guid id) => throw new NotImplementedException();
        public IEnumerable<ClipboardItem> GetAll() => throw new NotImplementedException();
        public ClipboardItem? GetById(Guid id) => throw new NotImplementedException();
        public ClipboardItem? GetLatest() => throw new NotImplementedException();
        public ClipboardItem? FindByContentAndType(string content, ClipboardContentType type) => throw new NotImplementedException();
        public ClipboardItem? FindByContentHash(string contentHash) => throw new NotImplementedException();
        public void Save(ClipboardItem item) => throw new NotImplementedException();
        public IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0) => throw new NotImplementedException();
        public IEnumerable<ClipboardItem> SearchAdvanced(
            string? query,
            IReadOnlyCollection<ClipboardContentType>? types,
            IReadOnlyCollection<CardColor>? colors,
            bool? isPinned,
            int limit,
            int skip) => throw new NotImplementedException();
        public void Update(ClipboardItem item) => throw new NotImplementedException();
    }
}
