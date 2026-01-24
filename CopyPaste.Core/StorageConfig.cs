namespace CopyPaste.Core;

public static class StorageConfig
{
    private static readonly string _appDataPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "CopyPaste"
    );

    public static string DatabasePath => Path.Combine(_appDataPath, "history.db");

    public static string ImagesPath => Path.Combine(_appDataPath, "images");

    public static string ThumbnailsPath => Path.Combine(_appDataPath, "thumbs");

    public static void Initialize()
    {
        // Ensure folders exist on startup
        if (!Directory.Exists(_appDataPath)) Directory.CreateDirectory(_appDataPath);
        if (!Directory.Exists(ImagesPath)) Directory.CreateDirectory(ImagesPath);
        if (!Directory.Exists(ThumbnailsPath)) Directory.CreateDirectory(ThumbnailsPath);
    }

    public static void CleanOrphanImages(IEnumerable<string> validPaths)
    {
        if (!Directory.Exists(ImagesPath)) return;

        var existingFiles = Directory.GetFiles(ImagesPath);
        var validSet = new HashSet<string>(validPaths);

        foreach (var file in existingFiles)
        {
            if (!validSet.Contains(file))
            {
                File.Delete(file);
            }
        }
    }
}
