using CopyPaste.Core;
using Xunit;

namespace CopyPaste.Core.Tests;

public class SearchHelperTests
{
    [Fact]
    public void NormalizeText_RemovesDiacritics()
    {
        var normalized = SearchHelper.NormalizeText("canción");

        Assert.Equal("cancion", normalized);
    }

    [Fact]
    public void MatchesQuery_KeywordFiltersByType()
    {
        var imageItem = new ClipboardItem { Type = ClipboardContentType.Image };
        var textItem = new ClipboardItem { Type = ClipboardContentType.Text };

        Assert.True(SearchHelper.MatchesQuery(imageItem, "imagen"));
        Assert.False(SearchHelper.MatchesQuery(textItem, "imagen"));
    }

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
}
