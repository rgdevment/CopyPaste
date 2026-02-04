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
}
