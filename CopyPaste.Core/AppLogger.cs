using System.Collections.Concurrent;
using System.Globalization;
using System.Runtime.CompilerServices;
using System.Text;

namespace CopyPaste.Core;

/// <summary>
/// Simple file logger optimized for AOT and minimal allocations.
/// Logs are written to the application data folder.
/// </summary>
public static class AppLogger
{
    private static readonly string _logDirectory = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "CopyPaste", "logs");

    private static readonly string _logFilePath = Path.Combine(
        _logDirectory,
        $"copypaste_{DateTime.Now:yyyy-MM-dd}.log");

    private static readonly ConcurrentQueue<string> _logQueue = new();
    private static readonly Lock _writeLock = new();
    private static bool _isInitialized;
    private static bool _isEnabled = true;

    private const int _maxLogAgeDays = 7;
    private const int _maxLogSizeMb = 10;

    /// <summary>
    /// Enable or disable logging at runtime.
    /// </summary>
    public static bool IsEnabled
    {
        get => _isEnabled;
        set => _isEnabled = value;
    }

    /// <summary>
    /// Initialize the logger and clean old log files.
    /// </summary>
    public static void Initialize()
    {
        if (_isInitialized) return;

        try
        {
            Directory.CreateDirectory(_logDirectory);
            CleanOldLogs();
            _isInitialized = true;

            Info("Logger initialized", "AppLogger");
        }
        catch
        {
            _isEnabled = false;
        }
    }

    /// <summary>
    /// Log an informational message.
    /// </summary>
    public static void Info(string message, [CallerMemberName] string caller = "") =>
        Log("INFO", message, caller);

    /// <summary>
    /// Log a warning message.
    /// </summary>
    public static void Warn(string message, [CallerMemberName] string caller = "") =>
        Log("WARN", message, caller);

    /// <summary>
    /// Log an error message.
    /// </summary>
    public static void Error(string message, [CallerMemberName] string caller = "") =>
        Log("ERROR", message, caller);

    /// <summary>
    /// Log an exception with full details.
    /// </summary>
    public static void Exception(Exception? ex, string context = "", [CallerMemberName] string caller = "")
    {
        if (!_isEnabled || ex is null) return;

        var sb = new StringBuilder();
        sb.Append(context);
        if (!string.IsNullOrEmpty(context)) sb.Append(" - ");
        sb.Append(ex.GetType().Name);
        sb.Append(": ");
        sb.Append(ex.Message);

        if (ex.StackTrace is not null)
        {
            sb.AppendLine();
            sb.Append("  StackTrace: ");
            sb.Append(ex.StackTrace);
        }

        if (ex.InnerException is not null)
        {
            sb.AppendLine();
            sb.Append("  Inner: ");
            sb.Append(ex.InnerException.Message);
        }

        Log("ERROR", sb.ToString(), caller);
    }

    /// <summary>
    /// Gets the path to the current log file.
    /// </summary>
    public static string LogFilePath => _logFilePath;

    /// <summary>
    /// Gets the logs directory path.
    /// </summary>
    public static string LogDirectory => _logDirectory;

    private static void Log(string level, string message, string caller)
    {
        if (!_isEnabled || !_isInitialized) return;

        var timestamp = DateTime.Now.ToString("HH:mm:ss.fff", CultureInfo.InvariantCulture);
        var logEntry = $"[{timestamp}] [{level}] [{caller}] {message}";

        _logQueue.Enqueue(logEntry);
        FlushQueue();
    }

    private static void FlushQueue()
    {
        if (!_logQueue.TryPeek(out _)) return;

        lock (_writeLock)
        {
            try
            {
                // Check file size before writing
                if (File.Exists(_logFilePath))
                {
                    var fileInfo = new FileInfo(_logFilePath);
                    if (fileInfo.Length > _maxLogSizeMb * 1024 * 1024)
                    {
                        RotateLog();
                    }
                }

                using var writer = new StreamWriter(_logFilePath, append: true, Encoding.UTF8);
                while (_logQueue.TryDequeue(out var entry))
                {
                    writer.WriteLine(entry);
                }
            }
            catch
            {
                // Silently fail - logging should never crash the app
            }
        }
    }

    private static void RotateLog()
    {
        var rotatedPath = Path.Combine(_logDirectory,
            $"copypaste_{DateTime.Now:yyyy-MM-dd_HHmmss}.log");

        try
        {
            File.Move(_logFilePath, rotatedPath);
        }
        catch
        {
            // If rotation fails, just delete the old log
            File.Delete(_logFilePath);
        }
    }

    private static void CleanOldLogs()
    {
        try
        {
            var cutoffDate = DateTime.Now.AddDays(-_maxLogAgeDays);
            var logFiles = Directory.GetFiles(_logDirectory, "copypaste_*.log");

            foreach (var file in logFiles)
            {
                var fileInfo = new FileInfo(file);
                if (fileInfo.LastWriteTime < cutoffDate)
                {
                    File.Delete(file);
                }
            }
        }
        catch
        {
            // Ignore cleanup errors
        }
    }
}
