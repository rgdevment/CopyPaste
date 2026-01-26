using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Core.Tests;

public class CleanupServiceTests : IDisposable
{
    private readonly string _basePath;

    public CleanupServiceTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
    }

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

    private string GetLastCleanupFile()
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

        public int ClearOldItems(int days, bool excludePinned = true)
        {
            ClearCalls++;
            return 0;
        }

        public void Delete(Guid id) => throw new NotImplementedException();

        public IEnumerable<ClipboardItem> GetAll() => throw new NotImplementedException();

        public ClipboardItem? GetById(Guid id) => throw new NotImplementedException();

        public ClipboardItem? GetLatest() => throw new NotImplementedException();

        public void Save(ClipboardItem item) => throw new NotImplementedException();

        public IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0) => throw new NotImplementedException();

        public void Update(ClipboardItem item) => throw new NotImplementedException();
    }
}
