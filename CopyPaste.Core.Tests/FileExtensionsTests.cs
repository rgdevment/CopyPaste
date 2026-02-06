using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class FileExtensionsTests
{
    #region Audio Extensions

    [Theory]
    [InlineData(".mp3", ClipboardContentType.Audio)]
    [InlineData(".MP3", ClipboardContentType.Audio)]
    [InlineData(".Mp3", ClipboardContentType.Audio)]
    [InlineData(".wav", ClipboardContentType.Audio)]
    [InlineData(".WAV", ClipboardContentType.Audio)]
    [InlineData(".flac", ClipboardContentType.Audio)]
    [InlineData(".FLAC", ClipboardContentType.Audio)]
    [InlineData(".aac", ClipboardContentType.Audio)]
    [InlineData(".AAC", ClipboardContentType.Audio)]
    [InlineData(".ogg", ClipboardContentType.Audio)]
    [InlineData(".OGG", ClipboardContentType.Audio)]
    [InlineData(".wma", ClipboardContentType.Audio)]
    [InlineData(".WMA", ClipboardContentType.Audio)]
    [InlineData(".m4a", ClipboardContentType.Audio)]
    [InlineData(".M4A", ClipboardContentType.Audio)]
    public void GetContentType_AudioExtensions_ReturnsAudio(string extension, ClipboardContentType expected)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(expected, result);
    }

    #endregion

    #region Video Extensions

    [Theory]
    [InlineData(".mp4", ClipboardContentType.Video)]
    [InlineData(".MP4", ClipboardContentType.Video)]
    [InlineData(".Mp4", ClipboardContentType.Video)]
    [InlineData(".avi", ClipboardContentType.Video)]
    [InlineData(".AVI", ClipboardContentType.Video)]
    [InlineData(".mkv", ClipboardContentType.Video)]
    [InlineData(".MKV", ClipboardContentType.Video)]
    [InlineData(".mov", ClipboardContentType.Video)]
    [InlineData(".MOV", ClipboardContentType.Video)]
    [InlineData(".wmv", ClipboardContentType.Video)]
    [InlineData(".WMV", ClipboardContentType.Video)]
    [InlineData(".flv", ClipboardContentType.Video)]
    [InlineData(".FLV", ClipboardContentType.Video)]
    [InlineData(".webm", ClipboardContentType.Video)]
    [InlineData(".WEBM", ClipboardContentType.Video)]
    public void GetContentType_VideoExtensions_ReturnsVideo(string extension, ClipboardContentType expected)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(expected, result);
    }

    #endregion

    #region Image Extensions

    [Theory]
    [InlineData(".png", ClipboardContentType.Image)]
    [InlineData(".PNG", ClipboardContentType.Image)]
    [InlineData(".Png", ClipboardContentType.Image)]
    [InlineData(".jpg", ClipboardContentType.Image)]
    [InlineData(".JPG", ClipboardContentType.Image)]
    [InlineData(".jpeg", ClipboardContentType.Image)]
    [InlineData(".JPEG", ClipboardContentType.Image)]
    [InlineData(".gif", ClipboardContentType.Image)]
    [InlineData(".GIF", ClipboardContentType.Image)]
    [InlineData(".bmp", ClipboardContentType.Image)]
    [InlineData(".BMP", ClipboardContentType.Image)]
    [InlineData(".webp", ClipboardContentType.Image)]
    [InlineData(".WEBP", ClipboardContentType.Image)]
    [InlineData(".svg", ClipboardContentType.Image)]
    [InlineData(".SVG", ClipboardContentType.Image)]
    [InlineData(".ico", ClipboardContentType.Image)]
    [InlineData(".ICO", ClipboardContentType.Image)]
    public void GetContentType_ImageExtensions_ReturnsImage(string extension, ClipboardContentType expected)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(expected, result);
    }

    #endregion

    #region File Extensions (Default)

    [Theory]
    [InlineData(".txt", ClipboardContentType.File)]
    [InlineData(".TXT", ClipboardContentType.File)]
    [InlineData(".doc", ClipboardContentType.File)]
    [InlineData(".docx", ClipboardContentType.File)]
    [InlineData(".pdf", ClipboardContentType.File)]
    [InlineData(".PDF", ClipboardContentType.File)]
    [InlineData(".zip", ClipboardContentType.File)]
    [InlineData(".rar", ClipboardContentType.File)]
    [InlineData(".7z", ClipboardContentType.File)]
    [InlineData(".exe", ClipboardContentType.File)]
    [InlineData(".dll", ClipboardContentType.File)]
    [InlineData(".cs", ClipboardContentType.File)]
    [InlineData(".json", ClipboardContentType.File)]
    [InlineData(".xml", ClipboardContentType.File)]
    [InlineData(".html", ClipboardContentType.File)]
    [InlineData(".css", ClipboardContentType.File)]
    [InlineData(".js", ClipboardContentType.File)]
    [InlineData(".unknown", ClipboardContentType.File)]
    [InlineData(".xyz123", ClipboardContentType.File)]
    public void GetContentType_UnknownExtensions_ReturnsFile(string extension, ClipboardContentType expected)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(expected, result);
    }

    #endregion

    #region Edge Cases

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    public void GetContentType_NullOrEmpty_ReturnsFile(string? extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Theory]
    [InlineData("mp3")]
    [InlineData("PNG")]
    [InlineData("mp4")]
    public void GetContentType_WithoutDot_ReturnsFile(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Theory]
    [InlineData("   .mp3   ")]
    [InlineData("\t.png\t")]
    public void GetContentType_WithWhitespace_HandlesCorrectly(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        // Whitespace is not trimmed, so it won't match
        Assert.Equal(ClipboardContentType.File, result);
    }

    [Theory]
    [InlineData("..mp3")]
    [InlineData("...png")]
    public void GetContentType_MultipleDots_ReturnsFile(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.File, result);
    }

    [Fact]
    public void GetContentType_CaseInsensitive_AllVariations()
    {
        Assert.Equal(ClipboardContentType.Audio, FileExtensions.GetContentType(".mp3"));
        Assert.Equal(ClipboardContentType.Audio, FileExtensions.GetContentType(".MP3"));
        Assert.Equal(ClipboardContentType.Audio, FileExtensions.GetContentType(".Mp3"));
        Assert.Equal(ClipboardContentType.Audio, FileExtensions.GetContentType(".mP3"));
    }

    #endregion
}
