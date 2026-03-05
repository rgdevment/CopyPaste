using System;
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

    [Fact]
    public void NormalizeText_WhitespaceOnly_ReturnsWhitespace()
    {
        var normalized = SearchHelper.NormalizeText("   ");

        Assert.Equal("   ", normalized);
    }

    [Theory]
    [InlineData("Ελληνικά", "Ελληνικα")]       // Greek with tonos
    [InlineData("日本語", "日本語")]              // Japanese (no diacritics)
    [InlineData("中文字符", "中文字符")]           // Chinese (no diacritics)
    [InlineData("한국어", "한국어")]               // Korean (no diacritics)
    public void NormalizeText_NonLatinScripts_HandledCorrectly(string input, string expected)
    {
        var normalized = SearchHelper.NormalizeText(input);

        Assert.Equal(expected, normalized);
    }

    [Theory]
    [InlineData("Café 123", "Cafe 123")]
    [InlineData("Año 2024", "Ano 2024")]
    [InlineData("Ñoño #42!", "Nono #42!")]
    public void NormalizeText_MixedDiacriticsAndNumbers_NormalizesCorrectly(string input, string expected)
    {
        var normalized = SearchHelper.NormalizeText(input);

        Assert.Equal(expected, normalized);
    }

    [Fact]
    public void NormalizeText_SpecialSymbols_PreservesUnchanged()
    {
        var normalized = SearchHelper.NormalizeText("@#$%^&*()_+");

        Assert.Equal("@#$%^&*()_+", normalized);
    }

    [Fact]
    public void NormalizeText_VeryLongString_HandlesWithoutError()
    {
        var longString = new string('a', 10_000) + "café" + new string('b', 10_000);

        var normalized = SearchHelper.NormalizeText(longString);

        Assert.NotNull(normalized);
        Assert.Contains("cafe", normalized, StringComparison.Ordinal);
        Assert.Equal(20_004, normalized.Length);
    }

    [Theory]
    [InlineData("résumé", "resume")]
    [InlineData("naïve", "naive")]
    [InlineData("Ångström", "Angstrom")]
    [InlineData("über", "uber")]
    public void NormalizeText_CommonEnglishLoanwords_NormalizesCorrectly(string input, string expected)
    {
        var normalized = SearchHelper.NormalizeText(input);

        Assert.Equal(expected, normalized);
    }

    [Fact]
    public void NormalizeText_NewlinesAndTabs_PreservesWhitespace()
    {
        var normalized = SearchHelper.NormalizeText("línea\nuno\ttab");

        Assert.Equal("linea\nuno\ttab", normalized);
    }

    #endregion
}
