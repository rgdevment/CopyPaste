using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Core.Tests;

public class StorageConfigTests : IDisposable
{
    private readonly string _basePath;

    public StorageConfigTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
    }

    [Fact]
    public void CleanOrphanImages_RemovesFilesNotInValidSet()
    {
        var validImage = Path.Combine(StorageConfig.ImagesPath, "valid.png");
        var validThumb = Path.Combine(StorageConfig.ThumbnailsPath, "valid_t.png");
        var orphanImage = Path.Combine(StorageConfig.ImagesPath, "orphan.png");
        var orphanThumb = Path.Combine(StorageConfig.ThumbnailsPath, "orphan_t.png");

        File.WriteAllText(validImage, "valid");
        File.WriteAllText(validThumb, "valid thumb");
        File.WriteAllText(orphanImage, "orphan");
        File.WriteAllText(orphanThumb, "orphan thumb");

        StorageConfig.CleanOrphanImages([validImage]);

        Assert.True(File.Exists(validImage));
        Assert.True(File.Exists(validThumb));
        Assert.False(File.Exists(orphanImage));
        Assert.False(File.Exists(orphanThumb));
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
}
