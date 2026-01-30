using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class SearchHelperTests
{
    #region NormalizeText Tests

    [Fact]
    public void NormalizeText_RemovesDiacritics()
    {
        var normalized = SearchHelper.NormalizeText("canción");

        Assert.Equal("cancion", normalized);
    }

    [Theory]
    [InlineData("Café", "Cafe")]
    [InlineData("Niño", "Nino")]
    [InlineData("Ñoño", "Nono")]
    [InlineData("São Paulo", "Sao Paulo")]
    [InlineData("Zürich", "Zurich")]
    public void NormalizeText_RemovesVariousDiacritics(string input, string expected)
    {
        var normalized = SearchHelper.NormalizeText(input);

        Assert.Equal(expected, normalized);
    }

    [Fact]
    public void NormalizeText_NullInput_ReturnsNull()
    {
        var normalized = SearchHelper.NormalizeText(null!);

        Assert.Null(normalized);
    }

    [Fact]
    public void NormalizeText_EmptyString_ReturnsEmpty()
    {
        var normalized = SearchHelper.NormalizeText(string.Empty);

        Assert.Equal(string.Empty, normalized);
    }

    [Fact]
    public void NormalizeText_PlainAscii_ReturnsUnchanged()
    {
        var normalized = SearchHelper.NormalizeText("Hello World 123");

        Assert.Equal("Hello World 123", normalized);
    }

    #endregion

    #region MatchesQuery - Type Keyword Tests

    [Fact]
    public void MatchesQuery_KeywordFiltersByType()
    {
        var imageItem = new ClipboardItem { Type = ClipboardContentType.Image };
        var textItem = new ClipboardItem { Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(imageItem, "imagen"));
        Assert.False(SearchHelper.MatchesQuery(textItem, "imagen"));
    }

    [Theory]
    [InlineData("IMAGE", ClipboardContentType.Image)]
    [InlineData("IMAGEN", ClipboardContentType.Image)]
    [InlineData("IMG", ClipboardContentType.Image)]
    [InlineData("image", ClipboardContentType.Image)]
    [InlineData("  imagen  ", ClipboardContentType.Image)]
    public void MatchesQuery_ImageKeywords_MatchesImageType(string keyword, ClipboardContentType type)
    {
        var item = new ClipboardItem { Type = type, Content = "test" };

        Assert.True(SearchHelper.MatchesQuery(item, keyword));
    }

    [Theory]
    [InlineData("VIDEO", ClipboardContentType.Video)]
    [InlineData("video", ClipboardContentType.Video)]
    public void MatchesQuery_VideoKeywords_MatchesVideoType(string keyword, ClipboardContentType type)
    {
        var item = new ClipboardItem { Type = type, Content = "test" };

        Assert.True(SearchHelper.MatchesQuery(item, keyword));
    }

    [Theory]
    [InlineData("AUDIO", ClipboardContentType.Audio)]
    [InlineData("MUSICA", ClipboardContentType.Audio)]
    [InlineData("CANCION", ClipboardContentType.Audio)]
    [InlineData("audio", ClipboardContentType.Audio)]
    public void MatchesQuery_AudioKeywords_MatchesAudioType(string keyword, ClipboardContentType type)
    {
        var item = new ClipboardItem { Type = type, Content = "test" };

        Assert.True(SearchHelper.MatchesQuery(item, keyword));
    }

    [Theory]
    [InlineData("FILE", ClipboardContentType.File)]
    [InlineData("ARCHIVO", ClipboardContentType.File)]
    [InlineData("file", ClipboardContentType.File)]
    public void MatchesQuery_FileKeywords_MatchesFileType(string keyword, ClipboardContentType type)
    {
        var item = new ClipboardItem { Type = type, Content = "test" };

        Assert.True(SearchHelper.MatchesQuery(item, keyword));
    }

    [Theory]
    [InlineData("FOLDER", ClipboardContentType.Folder)]
    [InlineData("CARPETA", ClipboardContentType.Folder)]
    [InlineData("DIRECTORIO", ClipboardContentType.Folder)]
    [InlineData("folder", ClipboardContentType.Folder)]
    public void MatchesQuery_FolderKeywords_MatchesFolderType(string keyword, ClipboardContentType type)
    {
        var item = new ClipboardItem { Type = type, Content = "test" };

        Assert.True(SearchHelper.MatchesQuery(item, keyword));
    }

    [Theory]
    [InlineData("LINK", ClipboardContentType.Link)]
    [InlineData("ENLACE", ClipboardContentType.Link)]
    [InlineData("URL", ClipboardContentType.Link)]
    [InlineData("link", ClipboardContentType.Link)]
    public void MatchesQuery_LinkKeywords_MatchesLinkType(string keyword, ClipboardContentType type)
    {
        var item = new ClipboardItem { Type = type, Content = "test" };

        Assert.True(SearchHelper.MatchesQuery(item, keyword));
    }

    [Theory]
    [InlineData("TEXT", ClipboardContentType.Text)]
    [InlineData("TEXTO", ClipboardContentType.Text)]
    [InlineData("text", ClipboardContentType.Text)]
    public void MatchesQuery_TextKeywords_MatchesTextType(string keyword, ClipboardContentType type)
    {
        var item = new ClipboardItem { Type = type, Content = "test" };

        Assert.True(SearchHelper.MatchesQuery(item, keyword));
    }

    [Fact]
    public void MatchesQuery_TypeKeyword_DoesNotMatchDifferentType()
    {
        var textItem = new ClipboardItem { Type = ClipboardContentType.Text, Content = "text" };

        Assert.False(SearchHelper.MatchesQuery(textItem, "imagen"));
        Assert.False(SearchHelper.MatchesQuery(textItem, "video"));
        Assert.False(SearchHelper.MatchesQuery(textItem, "audio"));
    }

    #endregion

    #region MatchesQuery - Content Search Tests

    [Fact]
    public void MatchesQuery_NormalizesContentBeforeMatching()
    {
        var item = new ClipboardItem
        {
            Content = "Café con leche",
            Type = ClipboardContentType.Text
        };

        Assert.True(SearchHelper.MatchesQuery(item, "cafe"));
    }

    [Fact]
    public void MatchesQuery_EmptyQuery_ReturnsTrue()
    {
        var item = new ClipboardItem { Content = "anything" };

        Assert.True(SearchHelper.MatchesQuery(item, "   "));
    }

    [Fact]
    public void MatchesQuery_NullQuery_ReturnsTrue()
    {
        var item = new ClipboardItem { Content = "anything" };

        Assert.True(SearchHelper.MatchesQuery(item, string.Empty));
    }

    [Fact]
    public void MatchesQuery_CaseInsensitive_Matches()
    {
        var item = new ClipboardItem { Content = "Hello World", Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(item, "hello"));
        Assert.True(SearchHelper.MatchesQuery(item, "HELLO"));
        Assert.True(SearchHelper.MatchesQuery(item, "world"));
        Assert.True(SearchHelper.MatchesQuery(item, "WoRLd"));
    }

    [Fact]
    public void MatchesQuery_PartialMatch_Matches()
    {
        var item = new ClipboardItem { Content = "The quick brown fox", Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(item, "quick"));
        Assert.True(SearchHelper.MatchesQuery(item, "brown"));
        Assert.True(SearchHelper.MatchesQuery(item, "fox"));
        Assert.True(SearchHelper.MatchesQuery(item, "qui"));
    }

    [Fact]
    public void MatchesQuery_NotFound_ReturnsFalse()
    {
        var item = new ClipboardItem { Content = "Hello World", Type = ClipboardContentType.Text };

        Assert.False(SearchHelper.MatchesQuery(item, "goodbye"));
        Assert.False(SearchHelper.MatchesQuery(item, "xyz"));
    }

    [Fact]
    public void MatchesQuery_DefaultContent_DoesNotMatchNonEmptyQuery()
    {
        var item = new ClipboardItem { Type = ClipboardContentType.Text };

        Assert.False(SearchHelper.MatchesQuery(item, "test"));
    }

    [Fact]
    public void MatchesQuery_EmptyContent_MatchesEmptyQuery()
    {
        var item = new ClipboardItem { Content = string.Empty, Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(item, ""));
        Assert.True(SearchHelper.MatchesQuery(item, "   "));
    }

    [Fact]
    public void MatchesQuery_EmptyContent_DoesNotMatchNonEmptyQuery()
    {
        var item = new ClipboardItem { Content = string.Empty, Type = ClipboardContentType.Text };

        Assert.False(SearchHelper.MatchesQuery(item, "test"));
    }

    [Theory]
    [InlineData("José García", "jose garcia")]
    [InlineData("Montréal Québec", "montreal quebec")]
    [InlineData("Ñoño López", "nono lopez")]
    public void MatchesQuery_AccentedContent_MatchesNormalizedQuery(string content, string query)
    {
        var item = new ClipboardItem { Content = content, Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(item, query));
    }

    [Fact]
    public void MatchesQuery_SpecialCharacters_Matches()
    {
        var item = new ClipboardItem { Content = "test@email.com", Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(item, "test@"));
        Assert.True(SearchHelper.MatchesQuery(item, "@email"));
        Assert.True(SearchHelper.MatchesQuery(item, ".com"));
    }

    [Fact]
    public void MatchesQuery_Numbers_Matches()
    {
        var item = new ClipboardItem { Content = "Invoice #12345", Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(item, "12345"));
        Assert.True(SearchHelper.MatchesQuery(item, "123"));
        Assert.True(SearchHelper.MatchesQuery(item, "#123"));
    }

    [Fact]
    public void MatchesQuery_WithWhitespace_TrimsAndMatches()
    {
        var item = new ClipboardItem { Content = "Hello World", Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(item, "  hello  "));
        Assert.True(SearchHelper.MatchesQuery(item, "\tworld\t"));
    }

    #endregion
}
