namespace CopyPaste.Core;

/// <summary>
/// Service responsible for automatic cleanup of old clipboard items.
/// Runs once per day, checking every 18 hours.
/// </summary>
public sealed class CleanupService : IDisposable
{
    private static readonly TimeSpan _checkInterval = TimeSpan.FromHours(18);
    private const string _lastCleanupFileName = "last_cleanup.txt";

    private readonly IClipboardRepository _repository;
    private readonly Func<int> _getRetentionDays;
    private readonly Timer _timer;
    private bool _isDisposed;

    private static string CleanupFilePath => Path.Combine(
        Path.GetDirectoryName(StorageConfig.DatabasePath) ?? string.Empty,
        _lastCleanupFileName
    );

    public CleanupService(IClipboardRepository repository, Func<int> getRetentionDays, bool startTimer = true)
    {
        ArgumentNullException.ThrowIfNull(repository);
        ArgumentNullException.ThrowIfNull(getRetentionDays);

        _repository = repository;
        _getRetentionDays = getRetentionDays;

        _timer = new Timer(
            _ => RunCleanupIfNeeded(),
            null,
            startTimer ? TimeSpan.Zero : Timeout.InfiniteTimeSpan,
            startTimer ? _checkInterval : Timeout.InfiniteTimeSpan
        );
    }

    internal void RunCleanupIfNeeded()
    {
        if (_isDisposed) return;

        int retentionDays = _getRetentionDays();
        if (retentionDays <= 0) return;

        var lastCleanupDate = LoadLastCleanupDate();
        if (lastCleanupDate.Date == DateTime.UtcNow.Date) return;

        try
        {
            int deletedCount = _repository.ClearOldItems(retentionDays, excludePinned: true);
            SaveLastCleanupDate(DateTime.UtcNow);

            if (deletedCount > 0)
            {
                AppLogger.Info($"Cleanup: {deletedCount} items older than {retentionDays} days removed");
            }
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            AppLogger.Exception(ex, "Cleanup failed");
        }
    }

    private static DateTime LoadLastCleanupDate()
    {
        try
        {
            if (File.Exists(CleanupFilePath))
            {
                string content = File.ReadAllText(CleanupFilePath);
                if (DateTime.TryParse(content, out var date))
                {
                    return date;
                }
            }
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            AppLogger.Exception(ex, "Failed to load last cleanup date");
        }

        return DateTime.MinValue;
    }

    private static void SaveLastCleanupDate(DateTime date)
    {
        try
        {
            var directory = Path.GetDirectoryName(CleanupFilePath);
            if (!string.IsNullOrEmpty(directory) && !Directory.Exists(directory))
            {
                Directory.CreateDirectory(directory);
            }
            File.WriteAllText(CleanupFilePath, date.ToString("O"));
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            AppLogger.Exception(ex, "Failed to save last cleanup date");
        }
    }

    public void Dispose()
    {
        if (_isDisposed) return;
        _timer.Dispose();
        _isDisposed = true;
    }
}
