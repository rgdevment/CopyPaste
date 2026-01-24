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
        // CreateDirectory handles existence checks internally
        Directory.CreateDirectory(_appDataPath);
        Directory.CreateDirectory(ImagesPath);
        Directory.CreateDirectory(ThumbnailsPath);
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
