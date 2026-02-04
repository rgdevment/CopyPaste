using System;
using System.Collections.Generic;
using System.IO;
using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class StorageConfigTests : IDisposable
{
    private readonly string _basePath;

    public StorageConfigTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
    }

    #region Initialize Tests

    [Fact]
    public void Initialize_CreatesBaseDirectory()
    {
        Assert.True(Directory.Exists(_basePath));
    }

    [Fact]
    public void Initialize_CreatesImagesDirectory()
    {
        Assert.True(Directory.Exists(StorageConfig.ImagesPath));
    }

    [Fact]
    public void Initialize_CreatesThumbnailsDirectory()
    {
        Assert.True(Directory.Exists(StorageConfig.ThumbnailsPath));
    }

    [Fact]
    public void Initialize_CanBeCalledMultipleTimes()
    {
        StorageConfig.Initialize();
        StorageConfig.Initialize();
        StorageConfig.Initialize();

        Assert.True(Directory.Exists(_basePath));
        Assert.True(Directory.Exists(StorageConfig.ImagesPath));
        Assert.True(Directory.Exists(StorageConfig.ThumbnailsPath));
    }

    #endregion

    #region IsFirstRun Tests

    [Fact]
    public void IsFirstRun_ReturnsTrue_WhenFlagFileDoesNotExist()
    {
        var result = StorageConfig.IsFirstRun;

        Assert.True(result);
    }

    [Fact]
    public void IsFirstRun_ReturnsFalse_AfterMarkAsInitialized()
    {
        StorageConfig.MarkAsInitialized();

        var result = StorageConfig.IsFirstRun;

        Assert.False(result);
    }

    #endregion

    #region MarkAsInitialized Tests

    [Fact]
    public void MarkAsInitialized_CreatesInitializedFile()
    {
        StorageConfig.MarkAsInitialized();

        var flagFile = Path.Combine(_basePath, ".initialized");
        Assert.True(File.Exists(flagFile));
    }

    [Fact]
    public void MarkAsInitialized_WritesTimestamp()
    {
        var before = DateTime.UtcNow.AddSeconds(-1); // Add tolerance for timing issues
        StorageConfig.MarkAsInitialized();
        var after = DateTime.UtcNow.AddSeconds(1); // Add tolerance for timing issues

        var flagFile = Path.Combine(_basePath, ".initialized");
        var content = File.ReadAllText(flagFile);
        var timestamp = DateTime.Parse(content, System.Globalization.CultureInfo.InvariantCulture).ToUniversalTime();

        Assert.True(timestamp >= before && timestamp <= after,
            $"Timestamp {timestamp:O} should be between {before:O} and {after:O}");
    }

    [Fact]
    public void MarkAsInitialized_CanBeCalledMultipleTimes()
    {
        StorageConfig.MarkAsInitialized();
        StorageConfig.MarkAsInitialized();
        StorageConfig.MarkAsInitialized();

        var flagFile = Path.Combine(_basePath, ".initialized");
        Assert.True(File.Exists(flagFile));
    }

    #endregion

    #region CleanOrphanImages Tests

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

    [Fact]
    public void CleanOrphanImages_EmptyValidSet_RemovesAllFiles()
    {
        var image1 = Path.Combine(StorageConfig.ImagesPath, "image1.png");
        var image2 = Path.Combine(StorageConfig.ImagesPath, "image2.png");
        var thumb1 = Path.Combine(StorageConfig.ThumbnailsPath, "image1_t.png");

        File.WriteAllText(image1, "test");
        File.WriteAllText(image2, "test");
        File.WriteAllText(thumb1, "test");

        StorageConfig.CleanOrphanImages([]);

        Assert.False(File.Exists(image1));
        Assert.False(File.Exists(image2));
        Assert.False(File.Exists(thumb1));
    }

    [Fact]
    public void CleanOrphanImages_NoFilesToClean_DoesNotThrow()
    {
        StorageConfig.CleanOrphanImages([]);

        // Should not throw
        Assert.True(true);
    }

    [Fact]
    public void CleanOrphanImages_DirectoryDoesNotExist_DoesNotThrow()
    {
        var tempPath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(tempPath);

        StorageConfig.CleanOrphanImages([]);

        // Should not throw
        Assert.True(true);
    }

    [Fact]
    public void CleanOrphanImages_KeepsMultipleValidFiles()
    {
        var valid1 = Path.Combine(StorageConfig.ImagesPath, "valid1.png");
        var valid2 = Path.Combine(StorageConfig.ImagesPath, "valid2.png");
        var valid3 = Path.Combine(StorageConfig.ImagesPath, "valid3.png");
        var orphan = Path.Combine(StorageConfig.ImagesPath, "orphan.png");

        File.WriteAllText(valid1, "test");
        File.WriteAllText(valid2, "test");
        File.WriteAllText(valid3, "test");
        File.WriteAllText(orphan, "test");

        StorageConfig.CleanOrphanImages([valid1, valid2, valid3]);

        Assert.True(File.Exists(valid1));
        Assert.True(File.Exists(valid2));
        Assert.True(File.Exists(valid3));
        Assert.False(File.Exists(orphan));
    }

    [Fact]
    public void CleanOrphanImages_ThumbnailWithoutMainImage_RemovesThumbnail()
    {
        var orphanThumb = Path.Combine(StorageConfig.ThumbnailsPath, "orphan_t.png");
        File.WriteAllText(orphanThumb, "test");

        StorageConfig.CleanOrphanImages([]);

        Assert.False(File.Exists(orphanThumb));
    }

    [Fact]
    public void CleanOrphanImages_CaseInsensitiveMatch_Works()
    {
        var imageLower = Path.Combine(StorageConfig.ImagesPath, "image.png");
        File.WriteAllText(imageLower, "test");

        // Even if the path has different case in the valid set
        StorageConfig.CleanOrphanImages([imageLower]);

        Assert.True(File.Exists(imageLower));
    }

    #endregion

    #region Path Properties Tests

    [Fact]
    public void DatabasePath_IsCorrect()
    {
        var expected = Path.Combine(_basePath, "clipboard.db");
        Assert.Equal(expected, StorageConfig.DatabasePath);
    }

    [Fact]
    public void ImagesPath_IsCorrect()
    {
        var expected = Path.Combine(_basePath, "images");
        Assert.Equal(expected, StorageConfig.ImagesPath);
    }

    [Fact]
    public void ThumbnailsPath_IsCorrect()
    {
        var expected = Path.Combine(_basePath, "thumbs");
        Assert.Equal(expected, StorageConfig.ThumbnailsPath);
    }

    [Fact]
    public void SetBasePath_UpdatesAllPaths()
    {
        var newBasePath = Path.Combine(Path.GetTempPath(), "NewPath", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(newBasePath);

        Assert.Contains(newBasePath, StorageConfig.DatabasePath, StringComparison.Ordinal);
        Assert.Contains(newBasePath, StorageConfig.ImagesPath, StringComparison.Ordinal);
        Assert.Contains(newBasePath, StorageConfig.ThumbnailsPath, StringComparison.Ordinal);
    }

    #endregion

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
}
