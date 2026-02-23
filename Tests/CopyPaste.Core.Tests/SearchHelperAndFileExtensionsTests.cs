using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class SearchHelperCombinedTests
{
    #region Null and Empty

    [Fact]
    public void NormalizeText_Null_ReturnsNull()
    {
        var result = SearchHelper.NormalizeText(null!);

        Assert.Null(result);
    }

    [Fact]
    public void NormalizeText_Empty_ReturnsEmpty()
    {
        var result = SearchHelper.NormalizeText(string.Empty);

        Assert.Equal(string.Empty, result);
    }

    #endregion

    #region ASCII and Plain Text

    [Fact]
    public void NormalizeText_PlainAscii_ReturnsUnchanged()
    {
        var result = SearchHelper.NormalizeText("Hello World 123");

        Assert.Equal("Hello World 123", result);
    }

    [Theory]
    [InlineData("1234567890")]
    [InlineData("@#$%^&*()_+-=[]{}|;':\",./<>?")]
    public void NormalizeText_NumbersAndSpecialChars_ReturnsUnchanged(string input)
    {
        var result = SearchHelper.NormalizeText(input);

        Assert.Equal(input, result);
    }

    #endregion

    #region Accented Characters

    [Theory]
    [InlineData("café", "cafe")]
    [InlineData("résumé", "resume")]
    [InlineData("naïve", "naive")]
    [InlineData("über", "uber")]
    [InlineData("Ångström", "Angstrom")]
    public void NormalizeText_AccentedChars_RemovesDiacritics(string input, string expected)
    {
        var result = SearchHelper.NormalizeText(input);

        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData("ñ", "n")]
    [InlineData("Ñoño", "Nono")]
    [InlineData("ü", "u")]
    [InlineData("Zürich", "Zurich")]
    [InlineData("ö", "o")]
    [InlineData("Köln", "Koln")]
    public void NormalizeText_SpecialLatinChars_RemovesDiacritics(string input, string expected)
    {
        var result = SearchHelper.NormalizeText(input);

        Assert.Equal(expected, result);
    }

    #endregion

    #region Mixed Content

    [Theory]
    [InlineData("Café 123!", "Cafe 123!")]
    [InlineData("résumé v2.0", "resume v2.0")]
    [InlineData("Año 2024 #1", "Ano 2024 #1")]
    public void NormalizeText_MixedAccentedAndNonAccented_NormalizesCorrectly(string input, string expected)
    {
        var result = SearchHelper.NormalizeText(input);

        Assert.Equal(expected, result);
    }

    #endregion

    #region CJK and Non-Latin Scripts

    [Theory]
    [InlineData("日本語")]
    [InlineData("中文字符")]
    [InlineData("한국어")]
    public void NormalizeText_CjkCharacters_ReturnsUnchanged(string input)
    {
        var result = SearchHelper.NormalizeText(input);

        Assert.Equal(input, result);
    }

    #endregion
}

public sealed class FileExtensionsCombinedTests
{
    #region Null and Empty

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    public void GetContentType_NullOrEmpty_ReturnsFile(string? extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.File, result);
    }

    #endregion

    #region Audio Extensions

    [Theory]
    [InlineData(".mp3")]
    [InlineData(".wav")]
    [InlineData(".flac")]
    [InlineData(".aac")]
    [InlineData(".ogg")]
    [InlineData(".wma")]
    [InlineData(".m4a")]
    public void GetContentType_AudioExtensions_ReturnsAudio(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.Audio, result);
    }

    #endregion

    #region Video Extensions

    [Theory]
    [InlineData(".mp4")]
    [InlineData(".avi")]
    [InlineData(".mkv")]
    [InlineData(".mov")]
    [InlineData(".wmv")]
    [InlineData(".flv")]
    [InlineData(".webm")]
    public void GetContentType_VideoExtensions_ReturnsVideo(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.Video, result);
    }

    #endregion

    #region Image Extensions

    [Theory]
    [InlineData(".png")]
    [InlineData(".jpg")]
    [InlineData(".jpeg")]
    [InlineData(".gif")]
    [InlineData(".bmp")]
    [InlineData(".webp")]
    [InlineData(".svg")]
    [InlineData(".ico")]
    public void GetContentType_ImageExtensions_ReturnsImage(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.Image, result);
    }

    #endregion

    #region Case Insensitivity

    [Theory]
    [InlineData(".Mp3")]
    [InlineData(".MP3")]
    [InlineData(".PNG")]
    [InlineData(".Png")]
    [InlineData(".mp4")]
    [InlineData(".MP4")]
    [InlineData(".Mp4")]
    public void GetContentType_CaseInsensitive_ReturnsCorrectType(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.NotEqual(ClipboardContentType.File, result);
    }

    #endregion

    #region Unknown Extensions

    [Theory]
    [InlineData(".txt")]
    [InlineData(".pdf")]
    [InlineData(".doc")]
    public void GetContentType_UnknownExtension_ReturnsFile(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.File, result);
    }

    #endregion

    #region No Dot

    [Theory]
    [InlineData("mp3")]
    [InlineData("png")]
    [InlineData("mp4")]
    public void GetContentType_NoDot_ReturnsFile(string extension)
    {
        var result = FileExtensions.GetContentType(extension);

        Assert.Equal(ClipboardContentType.File, result);
    }

    #endregion
}
