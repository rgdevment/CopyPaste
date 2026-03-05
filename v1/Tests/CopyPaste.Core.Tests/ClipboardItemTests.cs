using System;
using System.IO;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class ClipboardItemTests : IDisposable
{
    private readonly string _basePath;

    public ClipboardItemTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        Directory.CreateDirectory(_basePath);
    }

    #region Default Values

    [Fact]
    public void NewItem_HasNonEmptyId()
    {
        var item = new ClipboardItem();
        Assert.NotEqual(Guid.Empty, item.Id);
    }

    [Fact]
    public void NewItem_HasEmptyContent()
    {
        var item = new ClipboardItem();
        Assert.Equal(string.Empty, item.Content);
    }

    [Fact]
    public void NewItem_HasDefaultType()
    {
        var item = new ClipboardItem();
        Assert.Equal(ClipboardContentType.Text, item.Type);
    }

    [Fact]
    public void NewItem_HasRecentCreatedAt()
    {
        var before = DateTime.UtcNow.AddSeconds(-1);
        var item = new ClipboardItem();
        var after = DateTime.UtcNow.AddSeconds(1);

        Assert.InRange(item.CreatedAt, before, after);
    }

    [Fact]
    public void NewItem_HasRecentModifiedAt()
    {
        var before = DateTime.UtcNow.AddSeconds(-1);
        var item = new ClipboardItem();
        var after = DateTime.UtcNow.AddSeconds(1);

        Assert.InRange(item.ModifiedAt, before, after);
    }

    [Fact]
    public void NewItem_IsNotPinned()
    {
        var item = new ClipboardItem();
        Assert.False(item.IsPinned);
    }

    [Fact]
    public void NewItem_HasNullLabel()
    {
        var item = new ClipboardItem();
        Assert.Null(item.Label);
    }

    [Fact]
    public void NewItem_HasNoCardColor()
    {
        var item = new ClipboardItem();
        Assert.Equal(CardColor.None, item.CardColor);
    }

    [Fact]
    public void NewItem_HasNullMetadata()
    {
        var item = new ClipboardItem();
        Assert.Null(item.Metadata);
    }

    [Fact]
    public void NewItem_HasZeroPasteCount()
    {
        var item = new ClipboardItem();
        Assert.Equal(0, item.PasteCount);
    }

    [Fact]
    public void NewItem_HasNullContentHash()
    {
        var item = new ClipboardItem();
        Assert.Null(item.ContentHash);
    }

    [Fact]
    public void NewItem_HasNullAppSource()
    {
        var item = new ClipboardItem();
        Assert.Null(item.AppSource);
    }

    [Fact]
    public void MaxLabelLength_Is40()
    {
        Assert.Equal(40, ClipboardItem.MaxLabelLength);
    }

    [Fact]
    public void TwoNewItems_HaveDifferentIds()
    {
        var item1 = new ClipboardItem();
        var item2 = new ClipboardItem();
        Assert.NotEqual(item1.Id, item2.Id);
    }

    #endregion

    #region IsFileBasedType Tests

    [Theory]
    [InlineData(ClipboardContentType.File, true)]
    [InlineData(ClipboardContentType.Folder, true)]
    [InlineData(ClipboardContentType.Audio, true)]
    [InlineData(ClipboardContentType.Video, true)]
    [InlineData(ClipboardContentType.Text, false)]
    [InlineData(ClipboardContentType.Image, false)]
    [InlineData(ClipboardContentType.Link, false)]
    [InlineData(ClipboardContentType.Unknown, false)]
    public void IsFileBasedType_ReturnsCorrectValue(ClipboardContentType type, bool expected)
    {
        var item = new ClipboardItem { Type = type };
        Assert.Equal(expected, item.IsFileBasedType);
    }

    #endregion

    #region IsFileAvailable Tests

    [Fact]
    public void IsFileAvailable_NonFileType_ReturnsTrue()
    {
        var item = new ClipboardItem { Type = ClipboardContentType.Text, Content = "any text" };
        Assert.True(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_ImageType_ReturnsTrue()
    {
        var item = new ClipboardItem { Type = ClipboardContentType.Image, Content = "some/path.png" };
        Assert.True(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_LinkType_ReturnsTrue()
    {
        var item = new ClipboardItem { Type = ClipboardContentType.Link, Content = "https://example.com" };
        Assert.True(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_FileType_WithEmptyContent_ReturnsFalse()
    {
        var item = new ClipboardItem { Type = ClipboardContentType.File, Content = string.Empty };
        Assert.False(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_FileType_WithNullContent_ReturnsFalse()
    {
        var item = new ClipboardItem { Type = ClipboardContentType.File, Content = null! };
        Assert.False(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_FileType_WithExistingFile_ReturnsTrue()
    {
        var filePath = Path.Combine(_basePath, "test.txt");
        File.WriteAllText(filePath, "content");

        var item = new ClipboardItem { Type = ClipboardContentType.File, Content = filePath };
        Assert.True(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_FileType_WithNonExistentFile_ReturnsFalse()
    {
        var filePath = Path.Combine(_basePath, "nonexistent.txt");
        var item = new ClipboardItem { Type = ClipboardContentType.File, Content = filePath };
        Assert.False(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_FolderType_WithExistingFolder_ReturnsTrue()
    {
        var folderPath = Path.Combine(_basePath, "testFolder");
        Directory.CreateDirectory(folderPath);

        var item = new ClipboardItem { Type = ClipboardContentType.Folder, Content = folderPath };
        Assert.True(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_FolderType_WithNonExistentFolder_ReturnsFalse()
    {
        var folderPath = Path.Combine(_basePath, "nonexistentFolder");
        var item = new ClipboardItem { Type = ClipboardContentType.Folder, Content = folderPath };
        Assert.False(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_AudioType_WithExistingFile_ReturnsTrue()
    {
        var filePath = Path.Combine(_basePath, "song.mp3");
        File.WriteAllText(filePath, "fake audio");

        var item = new ClipboardItem { Type = ClipboardContentType.Audio, Content = filePath };
        Assert.True(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_VideoType_WithExistingFile_ReturnsTrue()
    {
        var filePath = Path.Combine(_basePath, "video.mp4");
        File.WriteAllText(filePath, "fake video");

        var item = new ClipboardItem { Type = ClipboardContentType.Video, Content = filePath };
        Assert.True(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_MultipleFiles_ChecksFirstPathOnly()
    {
        var existingFile = Path.Combine(_basePath, "first.txt");
        File.WriteAllText(existingFile, "content");
        var nonExisting = Path.Combine(_basePath, "second.txt");

        var content = existingFile + Environment.NewLine + nonExisting;
        var item = new ClipboardItem { Type = ClipboardContentType.File, Content = content };

        Assert.True(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_MultipleFiles_FirstDoesNotExist_ReturnsFalse()
    {
        var nonExisting = Path.Combine(_basePath, "first_missing.txt");
        var existingFile = Path.Combine(_basePath, "second.txt");
        File.WriteAllText(existingFile, "content");

        var content = nonExisting + Environment.NewLine + existingFile;
        var item = new ClipboardItem { Type = ClipboardContentType.File, Content = content };

        Assert.False(item.IsFileAvailable());
    }

    [Fact]
    public void IsFileAvailable_FileType_OnlyNewlines_ReturnsFalse()
    {
        var item = new ClipboardItem
        {
            Type = ClipboardContentType.File,
            Content = Environment.NewLine + Environment.NewLine
        };
        Assert.False(item.IsFileAvailable());
    }

    #endregion

    #region Property Setters

    [Fact]
    public void Properties_CanBeSet()
    {
        var id = Guid.NewGuid();
        var now = DateTime.UtcNow;

        var item = new ClipboardItem
        {
            Id = id,
            Content = "test content",
            Type = ClipboardContentType.Link,
            CreatedAt = now,
            ModifiedAt = now,
            AppSource = "Chrome",
            IsPinned = true,
            Label = "My Label",
            CardColor = CardColor.Blue,
            Metadata = "{\"key\":\"value\"}",
            PasteCount = 5,
            ContentHash = "abc123"
        };

        Assert.Equal(id, item.Id);
        Assert.Equal("test content", item.Content);
        Assert.Equal(ClipboardContentType.Link, item.Type);
        Assert.Equal(now, item.CreatedAt);
        Assert.Equal(now, item.ModifiedAt);
        Assert.Equal("Chrome", item.AppSource);
        Assert.True(item.IsPinned);
        Assert.Equal("My Label", item.Label);
        Assert.Equal(CardColor.Blue, item.CardColor);
        Assert.Equal("{\"key\":\"value\"}", item.Metadata);
        Assert.Equal(5, item.PasteCount);
        Assert.Equal("abc123", item.ContentHash);
    }

    #endregion

    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_basePath))
                Directory.Delete(_basePath, recursive: true);
        }
        catch { }
    }
}
