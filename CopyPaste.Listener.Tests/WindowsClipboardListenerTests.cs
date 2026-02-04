using System;
using System.Collections.ObjectModel;
using System.IO;
using CopyPaste.Listener;
using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Listener.Tests;

public sealed class WindowsClipboardListenerTests : IDisposable
{
    private readonly string _tempDir;

    public WindowsClipboardListenerTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "CopyPasteListenerTests", Guid.NewGuid().ToString());
        Directory.CreateDirectory(_tempDir);
    }

    #region DetectTextType Tests

    [Theory]
    [InlineData("https://example.com", ClipboardContentType.Link)]
    [InlineData("http://example.com/page", ClipboardContentType.Link)]
    [InlineData("some text", ClipboardContentType.Text)]
    [InlineData("   ", ClipboardContentType.Text)]
    public void DetectTextType_ReturnsExpected(string input, ClipboardContentType expected)
    {
        var result = WindowsClipboardListener.DetectTextType(input);

        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData("https://github.com/user/repo", ClipboardContentType.Link)]
    [InlineData("http://localhost:3000", ClipboardContentType.Link)]
    [InlineData("https://www.google.com", ClipboardContentType.Link)]
    [InlineData("HTTP://EXAMPLE.COM", ClipboardContentType.Link)]
    [InlineData("HTTPS://EXAMPLE.COM", ClipboardContentType.Link)]
    public void DetectTextType_VariousUrls_ReturnsLink(string url, ClipboardContentType expected)
    {
        var result = WindowsClipboardListener.DetectTextType(url);

        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData("Just plain text")]
    [InlineData("Multiple\nLines\nOf\nText")]
    [InlineData("Text with numbers 12345")]
    [InlineData("")]
    [InlineData("  ")]
    public void DetectTextType_NonUrls_ReturnsText(string input)
    {
        var result = WindowsClipboardListener.DetectTextType(input);

        Assert.Equal(ClipboardContentType.Text, result);
    }

    [Theory]
    [InlineData("ftp://example.com")]
    [InlineData("file:///C:/path")]
    [InlineData("ht tp://broken.com")]
    public void DetectTextType_NonHttpProtocols_ReturnsText(string input)
    {
        var result = WindowsClipboardListener.DetectTextType(input);

        Assert.Equal(ClipboardContentType.Text, result);
    }

    [Fact]
    public void DetectTextType_NullInput_ReturnsText()
    {
        var result = WindowsClipboardListener.DetectTextType(null!);

        Assert.Equal(ClipboardContentType.Text, result);
    }

    #endregion

    #region DetectFileCollectionType Tests - Single Files

    [Fact]
    public void DetectFileCollectionType_SingleAudioFile_ReturnsAudio()
    {
        var files = new Collection<string> { Path.Combine(_tempDir, "track.mp3") };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.Audio, result);
    }

    [Theory]
    [InlineData("track.mp3", ClipboardContentType.Audio)]
    [InlineData("song.wav", ClipboardContentType.Audio)]
    [InlineData("audio.flac", ClipboardContentType.Audio)]
    [InlineData("music.aac", ClipboardContentType.Audio)]
    [InlineData("sound.ogg", ClipboardContentType.Audio)]
    [InlineData("tune.wma", ClipboardContentType.Audio)]
    [InlineData("audio.m4a", ClipboardContentType.Audio)]
    public void DetectFileCollectionType_SingleAudioVariousFormats_ReturnsAudio(string filename, ClipboardContentType expected)
    {
        var files = new Collection<string> { Path.Combine(_tempDir, filename) };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData("video.mp4", ClipboardContentType.Video)]
    [InlineData("movie.avi", ClipboardContentType.Video)]
    [InlineData("clip.mkv", ClipboardContentType.Video)]
    [InlineData("film.mov", ClipboardContentType.Video)]
    [InlineData("video.wmv", ClipboardContentType.Video)]
    [InlineData("clip.flv", ClipboardContentType.Video)]
    [InlineData("video.webm", ClipboardContentType.Video)]
    public void DetectFileCollectionType_SingleVideoVariousFormats_ReturnsVideo(string filename, ClipboardContentType expected)
    {
        var files = new Collection<string> { Path.Combine(_tempDir, filename) };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData("image.png", ClipboardContentType.Image)]
    [InlineData("photo.jpg", ClipboardContentType.Image)]
    [InlineData("picture.jpeg", ClipboardContentType.Image)]
    [InlineData("graphic.gif", ClipboardContentType.Image)]
    [InlineData("bitmap.bmp", ClipboardContentType.Image)]
    [InlineData("image.webp", ClipboardContentType.Image)]
    [InlineData("icon.svg", ClipboardContentType.Image)]
    [InlineData("favicon.ico", ClipboardContentType.Image)]
    public void DetectFileCollectionType_SingleImageVariousFormats_ReturnsImage(string filename, ClipboardContentType expected)
    {
        var files = new Collection<string> { Path.Combine(_tempDir, filename) };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData("document.txt")]
    [InlineData("data.json")]
    [InlineData("config.xml")]
    [InlineData("script.js")]
    [InlineData("style.css")]
    [InlineData("readme.md")]
    [InlineData("code.cs")]
    public void DetectFileCollectionType_SingleOtherFile_ReturnsFile(string filename)
    {
        var files = new Collection<string> { Path.Combine(_tempDir, filename) };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.File, result);
    }

    #endregion

    #region DetectFileCollectionType Tests - Multiple Files

    [Fact]
    public void DetectFileCollectionType_MultipleFiles_ReturnsFile()
    {
        var files = new Collection<string>
        {
            Path.Combine(_tempDir, "first.txt"),
            Path.Combine(_tempDir, "second.txt")
        };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Fact]
    public void DetectFileCollectionType_MultipleImages_ReturnsFile()
    {
        var files = new Collection<string>
        {
            Path.Combine(_tempDir, "image1.png"),
            Path.Combine(_tempDir, "image2.png")
        };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Fact]
    public void DetectFileCollectionType_MixedTypes_ReturnsFile()
    {
        var files = new Collection<string>
        {
            Path.Combine(_tempDir, "image.png"),
            Path.Combine(_tempDir, "video.mp4"),
            Path.Combine(_tempDir, "audio.mp3")
        };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.File, result);
    }

    #endregion

    #region DetectFileCollectionType Tests - Folders

    [Fact]
    public void DetectFileCollectionType_Folder_ReturnsFolder()
    {
        var folder = Directory.CreateDirectory(Path.Combine(_tempDir, "myFolder")).FullName;
        var files = new Collection<string> { folder };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.Folder, result);
    }

    [Fact]
    public void DetectFileCollectionType_MultipleFolders_ReturnsFile()
    {
        var folder1 = Directory.CreateDirectory(Path.Combine(_tempDir, "folder1")).FullName;
        var folder2 = Directory.CreateDirectory(Path.Combine(_tempDir, "folder2")).FullName;
        var files = new Collection<string> { folder1, folder2 };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Fact]
    public void DetectFileCollectionType_FolderAndFile_ReturnsFile()
    {
        var folder = Directory.CreateDirectory(Path.Combine(_tempDir, "folder")).FullName;
        var file = Path.Combine(_tempDir, "file.txt");
        var files = new Collection<string> { folder, file };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.File, result);
    }

    #endregion

    #region DetectFileCollectionType Tests - Edge Cases

    [Fact]
    public void DetectFileCollectionType_EmptyCollection_ReturnsFile()
    {
        var files = new Collection<string>();

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Fact]
    public void DetectFileCollectionType_NullCollection_ReturnsFile()
    {
        var result = WindowsClipboardListener.DetectFileCollectionType(null!);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Fact]
    public void DetectFileCollectionType_FileWithoutExtension_ReturnsFile()
    {
        var files = new Collection<string> { Path.Combine(_tempDir, "noextension") };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Fact]
    public void DetectFileCollectionType_CaseInsensitive_Works()
    {
        var files = new Collection<string> { Path.Combine(_tempDir, "IMAGE.PNG") };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.Image, result);
    }

    [Fact]
    public void DetectFileCollectionType_NonExistentFile_ReturnsBasedOnExtension()
    {
        var files = new Collection<string> { Path.Combine(_tempDir, "nonexistent.mp3") };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.Audio, result);
    }

    #endregion

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types", Justification = "Best-effort cleanup of temp test data should not fail tests")]
    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_tempDir))
            {
                Directory.Delete(_tempDir, recursive: true);
            }
        }
        catch
        {
            // Best-effort cleanup.
        }
    }
}
