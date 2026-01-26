using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Core.Tests;

public class FileExtensionsTests
{
    [Theory]
    [InlineData(".mp3", ClipboardContentType.Audio)]
    [InlineData(".MP4", ClipboardContentType.Video)]
    [InlineData(".png", ClipboardContentType.Image)]
    [InlineData(".txt", ClipboardContentType.File)]
    public void GetContentType_ReturnsExpectedType(string extension, ClipboardContentType expected)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    public void GetContentType_NullOrEmpty_ReturnsFile(string? extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.File, result);
    }
}
