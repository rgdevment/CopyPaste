using System.Net.Http;
using System.Text.Json;

namespace CopyPaste.Core;

/// <summary>
/// Event args for update available notification.
/// </summary>
[System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1054:URI parameters should not be strings",
    Justification = "URL comes from JSON API response and is consumed as string by Process.Start")]
public sealed class UpdateAvailableEventArgs(string newVersion, string downloadUrl) : EventArgs
{
    public string NewVersion { get; } = newVersion;

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1056:URI properties should not be strings",
        Justification = "URL comes from JSON API response and is consumed as string by Process.Start")]
    public string DownloadUrl { get; } = downloadUrl;
}

/// <summary>
/// Checks for updates by comparing current version against the latest GitHub release.
/// Only runs for standalone (unpackaged) builds — Store builds use Store updates.
/// </summary>
public sealed class UpdateChecker : IDisposable
{
    private const string _gitHubReleasesUrl = "https://api.github.com/repos/rgdevment/CopyPaste/releases/latest";
    private static readonly Uri _gitHubReleasesUri = new(_gitHubReleasesUrl);
    private const string _dismissedVersionFileName = "dismissed_update.txt";

    private static readonly TimeSpan _checkInterval = TimeSpan.FromHours(12);
    private static readonly TimeSpan _httpTimeout = TimeSpan.FromSeconds(15);

    private readonly HttpClient _httpClient;
    private readonly Timer _timer;
    private bool _isDisposed;

    /// <summary>
    /// Raised when a new version is available.
    /// </summary>
    public event EventHandler<UpdateAvailableEventArgs>? OnUpdateAvailable;

    public UpdateChecker()
    {
        _httpClient = new HttpClient { Timeout = _httpTimeout };
        _httpClient.DefaultRequestHeaders.Add("User-Agent", "CopyPaste-UpdateChecker");
        _httpClient.DefaultRequestHeaders.Add("Accept", "application/vnd.github.v3+json");

        _timer = new Timer(
            _ => CheckForUpdateAsync().ConfigureAwait(false),
            null,
            TimeSpan.FromSeconds(30), // First check 30s after startup
            _checkInterval
        );
    }

    /// <summary>
    /// Gets the current app version from the assembly.
    /// </summary>
    public static string GetCurrentVersion()
    {
        var assembly = System.Reflection.Assembly.GetEntryAssembly();
        var informational = assembly?
            .GetCustomAttributes(typeof(System.Reflection.AssemblyInformationalVersionAttribute), false)
            .OfType<System.Reflection.AssemblyInformationalVersionAttribute>()
            .FirstOrDefault()?.InformationalVersion;

        if (!string.IsNullOrEmpty(informational))
        {
            // Remove metadata after '+' (e.g., "1.2.0+abc123" -> "1.2.0")
            var plusIndex = informational.IndexOf('+', StringComparison.Ordinal);
            return plusIndex > 0 ? informational[..plusIndex] : informational;
        }

        var version = assembly?.GetName().Version;
        return version != null ? $"{version.Major}.{version.Minor}.{version.Build}" : "0.0.0";
    }

    internal async Task CheckForUpdateAsync()
    {
        if (_isDisposed) return;

        try
        {
            using var response = await _httpClient.GetAsync(_gitHubReleasesUri).ConfigureAwait(false);

            if (!response.IsSuccessStatusCode)
            {
                AppLogger.Warn($"Update check failed: HTTP {response.StatusCode}");
                return;
            }

            var json = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
            var release = JsonSerializer.Deserialize(json, GitHubReleaseJsonContext.Default.GitHubRelease);

            if (release == null || string.IsNullOrEmpty(release.TagName))
            {
                AppLogger.Warn("Update check: invalid response");
                return;
            }

            var latestVersion = release.TagName.TrimStart('v', 'V');
            var currentVersion = GetCurrentVersion();

            if (IsNewerVersion(latestVersion, currentVersion))
            {
                if (IsVersionDismissed(latestVersion))
                {
                    AppLogger.Info($"Update {latestVersion} available but dismissed by user");
                    return;
                }

                var downloadUrl = release.HtmlUrl ?? $"https://github.com/rgdevment/CopyPaste/releases/tag/{release.TagName}";
                AppLogger.Info($"Update available: {currentVersion} → {latestVersion}");
                OnUpdateAvailable?.Invoke(this, new UpdateAvailableEventArgs(latestVersion, downloadUrl));
            }
            else
            {
                AppLogger.Info($"App is up to date ({currentVersion})");
            }
        }
        catch (HttpRequestException ex)
        {
            AppLogger.Warn($"Update check failed (network): {ex.Message}");
        }
        catch (TaskCanceledException)
        {
            AppLogger.Warn("Update check timed out");
        }
        catch (JsonException ex)
        {
            AppLogger.Warn($"Update check failed (parse): {ex.Message}");
        }
    }

    /// <summary>
    /// Compares two semver-style versions. Returns true if <paramref name="latest"/> is newer.
    /// Supports pre-release tags (e.g., "1.0.0-beta.1" &lt; "1.0.0").
    /// </summary>
    internal static bool IsNewerVersion(string latest, string current)
    {
        // Split off pre-release suffix
        var (latestBase, latestPre) = SplitVersion(latest);
        var (currentBase, currentPre) = SplitVersion(current);

        if (!Version.TryParse(NormalizeVersion(latestBase), out var latestVer) ||
            !Version.TryParse(NormalizeVersion(currentBase), out var currentVer))
        {
            return false;
        }

        var baseComparison = latestVer.CompareTo(currentVer);
        if (baseComparison != 0)
            return baseComparison > 0;

        // Same base version: stable > pre-release
        if (string.IsNullOrEmpty(latestPre) && !string.IsNullOrEmpty(currentPre))
            return true; // latest is stable, current is pre-release

        if (!string.IsNullOrEmpty(latestPre) && string.IsNullOrEmpty(currentPre))
            return false; // latest is pre-release, current is stable

        // Both pre-release: compare numerically (beta.2 > beta.1)
        if (!string.IsNullOrEmpty(latestPre) && !string.IsNullOrEmpty(currentPre))
            return string.Compare(latestPre, currentPre, StringComparison.OrdinalIgnoreCase) > 0;

        return false;
    }

    private static (string baseVersion, string preRelease) SplitVersion(string version)
    {
        var dashIndex = version.IndexOf('-', StringComparison.Ordinal);
        return dashIndex > 0
            ? (version[..dashIndex], version[(dashIndex + 1)..])
            : (version, string.Empty);
    }

    private static string NormalizeVersion(string version)
    {
        var parts = version.Split('.');
        return parts.Length switch
        {
            1 => $"{parts[0]}.0.0",
            2 => $"{parts[0]}.{parts[1]}.0",
            _ => version
        };
    }

    /// <summary>
    /// Persists the user's decision to dismiss a specific version update.
    /// </summary>
    public static void DismissVersion(string version)
    {
        try
        {
            var filePath = GetDismissedFilePath();
            File.WriteAllText(filePath, version);
            AppLogger.Info($"User dismissed update notification for version {version}");
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            AppLogger.Warn($"Failed to save dismissed version: {ex.Message}");
        }
    }

    /// <summary>
    /// Checks if the user has dismissed a specific version notification.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Dismissed version check is non-critical - any failure should return false")]
    private static bool IsVersionDismissed(string version)
    {
        try
        {
            var filePath = GetDismissedFilePath();
            if (!File.Exists(filePath)) return false;

            var dismissed = File.ReadAllText(filePath).Trim();
            return string.Equals(dismissed, version, StringComparison.OrdinalIgnoreCase);
        }
        catch
        {
            return false;
        }
    }

    private static string GetDismissedFilePath() =>
        Path.Combine(StorageConfig.ConfigPath, _dismissedVersionFileName);

    public void Dispose()
    {
        if (_isDisposed) return;
        _isDisposed = true;
        _timer.Dispose();
        _httpClient.Dispose();
    }
}

/// <summary>
/// Minimal model for GitHub release API response.
/// </summary>
public sealed class GitHubRelease
{
    [System.Text.Json.Serialization.JsonPropertyName("tag_name")]
    public string? TagName { get; set; }

    [System.Text.Json.Serialization.JsonPropertyName("html_url")]
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1056:URI properties should not be strings",
        Justification = "Mapped from GitHub API JSON response")]
    public string? HtmlUrl { get; set; }

    [System.Text.Json.Serialization.JsonPropertyName("prerelease")]
    public bool Prerelease { get; set; }
}

/// <summary>
/// JSON serialization context for GitHub release (required for trimming).
/// </summary>
[System.Text.Json.Serialization.JsonSerializable(typeof(GitHubRelease))]
[System.Text.Json.Serialization.JsonSourceGenerationOptions(PropertyNameCaseInsensitive = true)]
public partial class GitHubReleaseJsonContext : System.Text.Json.Serialization.JsonSerializerContext
{
}
