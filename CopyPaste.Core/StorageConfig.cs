namespace CopyPaste.Core;

public static class StorageConfig
{
    private static string _appDataPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "CopyPaste"
    );

    public static string DatabasePath => Path.Combine(_appDataPath, "clipboard.db");
    public static string ImagesPath => Path.Combine(_appDataPath, "images");
    public static string ThumbnailsPath => Path.Combine(_appDataPath, "thumbs");
    public static string ConfigPath => Path.Combine(_appDataPath, "config");
    public static string ThemesPath => Path.Combine(_appDataPath, "themes");
    private static string FirstRunFlagPath => Path.Combine(_appDataPath, ".initialized");

    internal static void SetBasePath(string basePath) => _appDataPath = basePath;

    public static bool IsFirstRun => !File.Exists(FirstRunFlagPath);

    public static void Initialize()
    {
        Directory.CreateDirectory(_appDataPath);
        Directory.CreateDirectory(ImagesPath);
        Directory.CreateDirectory(ThumbnailsPath);
        Directory.CreateDirectory(ConfigPath);
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage(
        "Design",
        "CA1031:Do not catch general exception types",
        Justification = "First-run flag is non-critical - any failure should not prevent app from running")]
    public static void MarkAsInitialized()
    {
        try
        {
            File.WriteAllText(FirstRunFlagPath, DateTime.UtcNow.ToString("O"));
        }
        catch
        {
            // Non-critical, ignore errors
        }
    }

    public static void CleanOrphanImages(IEnumerable<string> validPaths)
    {
        var validSet = new HashSet<string>(validPaths);

        CleanDirectory(ImagesPath, validSet);

        var validThumbs = validSet.Select(p =>
            Path.Combine(ThumbnailsPath, $"{Path.GetFileNameWithoutExtension(p)}_t.png")
        ).ToHashSet();

        CleanDirectory(ThumbnailsPath, validThumbs);
    }

    private static void CleanDirectory(string path, HashSet<string> validFiles)
    {
        if (!Directory.Exists(path)) return;

        foreach (var file in Directory.GetFiles(path))
        {
            if (!validFiles.Contains(file))
            {
                File.Delete(file);
            }
        }
    }
}
