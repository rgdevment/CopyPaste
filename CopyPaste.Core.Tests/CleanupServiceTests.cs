using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using CopyPaste.Core;
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

    #region RunCleanupIfNeeded Tests

    [Fact]
    public void RunCleanupIfNeeded_CallsRepository_WhenDue()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
        Assert.True(File.Exists(GetLastCleanupFile()));
    }

    [Fact]
    public void RunCleanupIfNeeded_Skips_WhenRetentionIsZero()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 0, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(0, repo.ClearCalls);
        Assert.False(File.Exists(GetLastCleanupFile()));
    }

    [Fact]
    public void RunCleanupIfNeeded_Skips_WhenAlreadyCleanedToday()
    {
        var repo = new StubClipboardRepository();
        File.WriteAllText(GetLastCleanupFile(), DateTime.UtcNow.ToString("O"));
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(0, repo.ClearCalls);
    }

    [Fact]
    public void RunCleanupIfNeeded_Runs_WhenLastCleanupIsOld()
    {
        var repo = new StubClipboardRepository();
        var yesterday = DateTime.UtcNow.AddDays(-1);
        File.WriteAllText(GetLastCleanupFile(), yesterday.ToString("O"));
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
    }

    [Fact]
    public void RunCleanupIfNeeded_Skips_WhenNegativeRetention()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => -1, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(0, repo.ClearCalls);
        Assert.False(File.Exists(GetLastCleanupFile()));
    }

    [Fact]
    public void RunCleanupIfNeeded_HandlesInvalidLastCleanupFile()
    {
        var repo = new StubClipboardRepository();
        File.WriteAllText(GetLastCleanupFile(), "invalid-date");
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
        Assert.True(File.Exists(GetLastCleanupFile()));
    }

    [Theory]
    [InlineData(1)]
    [InlineData(7)]
    [InlineData(30)]
    [InlineData(90)]
    [InlineData(365)]
    public void RunCleanupIfNeeded_PassesCorrectRetentionDays(int days)
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => days, startTimer: false);

        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
        Assert.Equal(days, repo.LastRetentionDays);
    }

    [Fact]
    public void RunCleanupIfNeeded_CreatesDirectoryIfNotExists()
    {
        var repo = new StubClipboardRepository();
        var directory = Path.GetDirectoryName(StorageConfig.DatabasePath)!;
        if (Directory.Exists(directory))
        {
            Directory.Delete(directory, true);
        }

        using var service = new CleanupService(repo, () => 7, startTimer: false);
        service.RunCleanupIfNeeded();

        // Wait longer and retry for filesystem operations to complete
        var cleanupFile = GetLastCleanupFile();
        for (int i = 0; i < 20; i++)
        {
            if (Directory.Exists(directory) && File.Exists(cleanupFile))
                break;
            Thread.Sleep(100);
        }

        // Use cleanup file existence as proof that directory was created
        Assert.True(File.Exists(cleanupFile),
            $"Cleanup file should exist: {cleanupFile} (directory: {directory}, exists: {Directory.Exists(directory)})");
    }

    [Fact]
    public void RunCleanupIfNeeded_UpdatesLastCleanupFile()
    {
        var repo = new StubClipboardRepository();
        var beforeTime = DateTime.UtcNow.AddSeconds(-1); // Add tolerance

        using var service = new CleanupService(repo, () => 7, startTimer: false);
        service.RunCleanupIfNeeded();

        var afterTime = DateTime.UtcNow.AddSeconds(1); // Add tolerance
        var fileContent = File.ReadAllText(GetLastCleanupFile());
        var lastCleanupTime = DateTime.Parse(fileContent, System.Globalization.CultureInfo.InvariantCulture).ToUniversalTime();

        Assert.True(lastCleanupTime >= beforeTime && lastCleanupTime <= afterTime,
            $"Cleanup time {lastCleanupTime:O} should be between {beforeTime:O} and {afterTime:O}");
    }

    [Fact]
    public void RunCleanupIfNeeded_MultipleCallsSameDay_OnlyCleanOnce()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        service.RunCleanupIfNeeded();
        service.RunCleanupIfNeeded();
        service.RunCleanupIfNeeded();

        Assert.Equal(1, repo.ClearCalls);
    }

    #endregion

    #region Dispose Tests

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        var repo = new StubClipboardRepository();
        var service = new CleanupService(repo, () => 7, startTimer: false);

        service.Dispose();
        service.Dispose();
        service.Dispose();

        // Should not throw
        Assert.True(true);
    }

    [Fact]
    public void Constructor_WithTimerDisabled_DoesNotStartTimer()
    {
        var repo = new StubClipboardRepository();
        using var service = new CleanupService(repo, () => 7, startTimer: false);

        // Wait a bit to ensure timer doesn't trigger
        Thread.Sleep(100);

        Assert.Equal(0, repo.ClearCalls);
    }

    #endregion

    private static string GetLastCleanupFile()
    {
        var directory = Path.GetDirectoryName(StorageConfig.DatabasePath)!;
        return Path.Combine(directory, "last_cleanup.txt");
    }

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
