using CopyPaste.Core;
using CopyPaste.UI.Themes;
using Microsoft.UI.Xaml;
using System;
using System.IO;
using Xunit;

namespace CopyPaste.UI.Tests;

/// <summary>
/// Tests for ClipboardItemViewModel menu text properties and media-related members
/// (ThumbnailPath, ImagePath, IsFileAvailable, etc.) that are not covered elsewhere.
/// </summary>
public sealed class ClipboardItemViewModelMenuTextTests
{
    private static ClipboardItemViewModel Create(
        ClipboardItem model,
        Action<ClipboardItemViewModel>? editAction = null) =>
        new(model, _ => { }, (_, _) => { }, _ => { }, editAction);

    [Fact]
    public void PasteText_ReturnsNonEmptyString()
    {
        var vm = Create(new ClipboardItem { Content = "x", Type = ClipboardContentType.Text });

        Assert.NotEmpty(vm.PasteText);
    }

    [Fact]
    public void PastePlainText_ReturnsNonEmptyString()
    {
        var vm = Create(new ClipboardItem { Content = "x", Type = ClipboardContentType.Text });

        Assert.NotEmpty(vm.PastePlainText);
    }

    [Fact]
    public void DeleteText_ReturnsNonEmptyString()
    {
        var vm = Create(new ClipboardItem { Content = "x", Type = ClipboardContentType.Text });

        Assert.NotEmpty(vm.DeleteText);
    }

    [Fact]
    public void EditText_ReturnsNonEmptyString()
    {
        var vm = Create(new ClipboardItem { Content = "x", Type = ClipboardContentType.Text });

        Assert.NotEmpty(vm.EditText);
    }

    [Fact]
    public void FileWarningText_ReturnsNonEmptyString()
    {
        var vm = Create(new ClipboardItem { Content = "x", Type = ClipboardContentType.File });

        Assert.NotEmpty(vm.FileWarningText);
    }

    [Fact]
    public void HeaderTitle_ForTextType_ReturnsNonEmpty()
    {
        var vm = Create(new ClipboardItem { Content = "x", Type = ClipboardContentType.Text });

        Assert.NotEmpty(vm.HeaderTitle);
    }

    [Fact]
    public void PinMenuText_WhenUnpinned_ReturnsNonEmpty()
    {
        var model = new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, IsPinned = false };
        var vm = Create(model);

        Assert.NotEmpty(vm.PinMenuText);
    }

    [Fact]
    public void PinMenuText_WhenPinned_ReturnsNonEmpty()
    {
        var model = new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, IsPinned = true };
        var vm = Create(model);

        Assert.NotEmpty(vm.PinMenuText);
    }
}

public sealed class ClipboardItemViewModelFileAvailabilityTests
{
    [Fact]
    public void IsFileAvailable_ForTextItem_AlwaysTrue()
    {
        var model = new ClipboardItem { Content = "hello", Type = ClipboardContentType.Text };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.True(vm.IsFileAvailable);
    }

    [Fact]
    public void FileWarningVisibility_ForTextItem_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "hello", Type = ClipboardContentType.Text };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.Equal(Visibility.Collapsed, vm.FileWarningVisibility);
    }

    [Fact]
    public void IsFileAvailable_ForFileItemWithMissingPath_IsFalse()
    {
        var model = new ClipboardItem
        {
            Content = @"C:\this_path_does_not_exist_xyz\file.txt",
            Type = ClipboardContentType.File
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.False(vm.IsFileAvailable);
    }

    [Fact]
    public void FileWarningVisibility_ForMissingFile_IsVisible()
    {
        var model = new ClipboardItem
        {
            Content = @"C:\this_path_does_not_exist_xyz\file.txt",
            Type = ClipboardContentType.File
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.Equal(Visibility.Visible, vm.FileWarningVisibility);
    }
}

public sealed class ClipboardItemViewModelImageTests
{
    [Fact]
    public void HasValidImagePath_ForTextItem_IsFalse()
    {
        var model = new ClipboardItem { Content = "hello", Type = ClipboardContentType.Text };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.False(vm.HasValidImagePath);
    }

    [Fact]
    public void ImageVisibility_ForTextItem_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "hello", Type = ClipboardContentType.Text };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.Equal(Visibility.Collapsed, vm.ImageVisibility);
    }

    [Fact]
    public void HasValidImagePath_ForImageItemWithNoMetadata_IsTrue()
    {
        // For Image type, GetImagePathOrThumbnail returns the ms-appx placeholder,
        // so HasValidImagePath = true even without a real file on disk.
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\image.png",
            Type = ClipboardContentType.Image,
            Metadata = null
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.True(vm.HasValidImagePath);
    }

    [Fact]
    public void ImageVisibility_ForImageItem_IsVisible()
    {
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\image.png",
            Type = ClipboardContentType.Image,
            Metadata = null
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.Equal(Visibility.Visible, vm.ImageVisibility);
    }

    [Fact]
    public void ImagePath_ForImageItemWithNoMetadata_ReturnsPlaceholder()
    {
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\file.png",
            Type = ClipboardContentType.Image,
            Metadata = null
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var path = vm.ImagePath;

        // Should return ms-appx placeholder for Image type when no valid path
        Assert.NotEmpty(path);
    }

    [Fact]
    public void ImagePath_SecondAccess_UsesCachedValue()
    {
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\file.png",
            Type = ClipboardContentType.Image,
            Metadata = null
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var first = vm.ImagePath;
        var second = vm.ImagePath;

        Assert.Equal(first, second);
    }

    [Fact]
    public void ImagePath_ForImageItemWithThumbPathMetadata_ReturnsExpected()
    {
        var metadata = """{"thumb_path": "/nonexistent/thumb.jpg"}""";
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\image.png",
            Type = ClipboardContentType.Image,
            Metadata = metadata
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var path = vm.ImagePath;

        // thumb_path doesn't exist, so falls back to placeholder
        Assert.NotEmpty(path);
    }

    [Fact]
    public void ImagePath_ForImageItemWithExistingContentFile_ReturnsContentPath()
    {
        var tempFile = Path.GetTempFileName();
        try
        {
            var model = new ClipboardItem
            {
                Content = tempFile,
                Type = ClipboardContentType.Image,
                Metadata = null
            };
            var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

            var path = vm.ImagePath;

            Assert.Equal(tempFile, path);
        }
        finally
        {
            File.Delete(tempFile);
        }
    }
}

public sealed class ClipboardItemViewModelThumbnailTests
{
    [Fact]
    public void ThumbnailPath_ForVideoItemWithNoMetadata_ReturnsVideoPlaceholder()
    {
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = null
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var path = vm.ThumbnailPath;

        Assert.Contains("video", path, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ThumbnailPath_ForAudioItemWithNoMetadata_ReturnsAudioPlaceholder()
    {
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\audio.mp3",
            Type = ClipboardContentType.Audio,
            Metadata = null
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var path = vm.ThumbnailPath;

        Assert.Contains("audio", path, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ThumbnailPath_SecondAccess_UsesCachedValue()
    {
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = null
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var first = vm.ThumbnailPath;
        var second = vm.ThumbnailPath;

        Assert.Equal(first, second);
    }

    [Fact]
    public void ThumbnailPath_WithThumbPathMetadataButMissingFile_ReturnsPlaceholder()
    {
        var metadata = """{"thumb_path": "/nonexistent/path/thumb.jpg"}""";
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = metadata
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var path = vm.ThumbnailPath;

        // File doesn't exist, so falls back to placeholder
        Assert.NotEmpty(path);
        Assert.Contains("ms-appx", path, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ThumbnailPath_WithExistingThumbFile_ReturnsThumbnailPath()
    {
        var thumbFile = Path.GetTempFileName();
        try
        {
            var metadata = $$"""{"thumb_path": "{{thumbFile.Replace("\\", "\\\\", StringComparison.Ordinal)}}"}""";
            var model = new ClipboardItem
            {
                Content = @"C:\nonexistent\video.mp4",
                Type = ClipboardContentType.Video,
                Metadata = metadata
            };
            var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

            var path = vm.ThumbnailPath;

            Assert.Equal(thumbFile, path);
        }
        finally
        {
            File.Delete(thumbFile);
        }
    }

    [Fact]
    public void ThumbnailPath_WithMetadataNoThumbPathProperty_ReturnsPlaceholder()
    {
        var metadata = """{"duration": 120, "file_size": 50000}""";
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = metadata
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var path = vm.ThumbnailPath;

        Assert.Contains("ms-appx", path, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ThumbnailPath_WithMalformedMetadata_ReturnsPlaceholder()
    {
        // Malformed JSON triggers JsonException catch in GetThumbnailPath, returns null,
        // then GetThumbnailPathOrPlaceholder returns the placeholder.
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = "{ this is not valid json {{{"
        };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        var path = vm.ThumbnailPath;

        Assert.NotEmpty(path);
        Assert.Contains("ms-appx", path, StringComparison.OrdinalIgnoreCase);
    }
}

/// <summary>
/// Tests for exception-path coverage in GetThumbnailPath, GetImagePathOrThumbnail,
/// GetImageDimensions, and GetMediaDuration (malformed JSON metadata).
/// </summary>
public sealed class ClipboardItemViewModelMetadataExceptionTests
{
    private static ClipboardItemViewModel Create(ClipboardItem model) =>
        new(model, _ => { }, (_, _) => { }, _ => { });

    [Fact]
    public void ImagePath_WithMalformedJson_DoesNotThrow()
    {
        // Malformed JSON triggers JsonException in GetThumbnailPath (caught),
        // falls through to Image placeholder in GetImagePathOrThumbnail.
        var model = new ClipboardItem
        {
            Content = @"C:\nonexistent\img.png",
            Type = ClipboardContentType.Image,
            Metadata = "not valid json at all {{{"
        };
        var vm = Create(model);

        var exception = Record.Exception(() => _ = vm.ImagePath);

        Assert.Null(exception);
    }

    [Fact]
    public void ImagePath_ForNonImageTypeWithNoValidContent_ReturnsEmpty()
    {
        // Link type with non-existent content → GetImagePathOrThumbnail returns string.Empty
        var model = new ClipboardItem
        {
            Content = @"C:\this\does\not\exist.lnk",
            Type = ClipboardContentType.Link,
            Metadata = null
        };
        var vm = Create(model);

        var path = vm.ImagePath;

        Assert.Equal(string.Empty, path);
    }

    [Fact]
    public void ImageDimensions_WithMalformedJson_ReturnsNull()
    {
        var model = new ClipboardItem
        {
            Content = "img.png",
            Type = ClipboardContentType.Image,
            Metadata = "not valid json {{{"
        };
        var vm = Create(model);

        var dims = vm.ImageDimensions;

        Assert.Null(dims);
    }

    [Fact]
    public void MediaDuration_WithMalformedJson_ReturnsNull()
    {
        var model = new ClipboardItem
        {
            Content = "video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = "{ badJson"
        };
        var vm = Create(model);

        var duration = vm.MediaDuration;

        Assert.Null(duration);
    }

    [Fact]
    public void FileSize_WithMalformedJson_ReturnsNull()
    {
        var model = new ClipboardItem
        {
            Content = "file.txt",
            Type = ClipboardContentType.File,
            Metadata = "INVALID"
        };
        var vm = Create(model);

        var size = vm.FileSize;

        Assert.Null(size);
    }
}

/// <summary>
/// Tests for missing branch coverage in GetImageDimensions, GetMediaDuration,
/// GetImagePathOrThumbnail, and GetThumbnailPathOrPlaceholder.
/// </summary>
public sealed class ClipboardItemViewModelMissingBranchTests
{
    private static ClipboardItemViewModel Create(ClipboardItem model) =>
        new(model, _ => { }, (_, _) => { }, _ => { });

    [Fact]
    public void ImageDimensions_WithOnlyWidthProperty_ReturnsNull()
    {
        // TryGetProperty("height") fails → falls through the if → returns null
        var model = new ClipboardItem
        {
            Content = "img.png",
            Type = ClipboardContentType.Image,
            Metadata = """{"width": 1920}"""
        };
        var vm = Create(model);

        Assert.Null(vm.ImageDimensions);
    }

    [Fact]
    public void ImageDimensions_WithOnlyHeightProperty_ReturnsNull()
    {
        // TryGetProperty("width") fails → returns null
        var model = new ClipboardItem
        {
            Content = "img.png",
            Type = ClipboardContentType.Image,
            Metadata = """{"height": 1080}"""
        };
        var vm = Create(model);

        Assert.Null(vm.ImageDimensions);
    }

    [Fact]
    public void MediaDuration_WithMetadataButNoDurationKey_ReturnsNull()
    {
        // Metadata is valid JSON but has no "duration" property → TryGetProperty returns false → null
        var model = new ClipboardItem
        {
            Content = "video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = """{"file_size": 50000}"""
        };
        var vm = Create(model);

        Assert.Null(vm.MediaDuration);
    }

    [Fact]
    public void GetThumbnailPathOrPlaceholder_ForAudioType_ReturnsAudioPlaceholder()
    {
        // Covers the Audio branch in the switch expression
        var model = new ClipboardItem
        {
            Content = "audio.mp3",
            Type = ClipboardContentType.Audio,
            Metadata = null
        };
        var vm = Create(model);

        // ThumbnailPath goes through GetThumbnailPathOrPlaceholder
        Assert.Contains("audio", vm.ThumbnailPath, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void GetImagePathOrThumbnail_ForNonImageNonExistentContent_ReturnsEmpty()
    {
        // Type != Image, Content path doesn't exist → returns string.Empty
        var model = new ClipboardItem
        {
            Content = @"C:\does\not\exist\file.link",
            Type = ClipboardContentType.Link,
            Metadata = null
        };
        var vm = Create(model);

        Assert.Equal(string.Empty, vm.ImagePath);
    }
}
