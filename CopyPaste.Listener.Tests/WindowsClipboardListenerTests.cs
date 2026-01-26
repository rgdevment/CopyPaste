using System.Collections.ObjectModel;
using CopyPaste.Listener;
using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Listener.Tests;

public class WindowsClipboardListenerTests : IDisposable
{
    private readonly string _tempDir;

    public WindowsClipboardListenerTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "CopyPasteListenerTests", Guid.NewGuid().ToString());
        Directory.CreateDirectory(_tempDir);
    }

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

    [Fact]
    public void DetectFileCollectionType_SingleAudioFile_ReturnsAudio()
    {
        var files = new Collection<string> { Path.Combine(_tempDir, "track.mp3") };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.Audio, result);
    }

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
    public void DetectFileCollectionType_Folder_ReturnsFolder()
    {
        var folder = Directory.CreateDirectory(Path.Combine(_tempDir, "myFolder")).FullName;
        var files = new Collection<string> { folder };

        var result = WindowsClipboardListener.DetectFileCollectionType(files);

        Assert.Equal(ClipboardContentType.Folder, result);
    }

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
