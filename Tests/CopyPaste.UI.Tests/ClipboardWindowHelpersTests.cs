using CopyPaste.UI.Helpers;
using Windows.UI;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class ClipboardWindowHelpersTests
{
    #region ParseColor Tests

    [Theory]
    [InlineData("#E74C3C", 231, 76, 60)]
    [InlineData("#2ECC71", 46, 204, 113)]
    [InlineData("#9B59B6", 155, 89, 182)]
    [InlineData("#F1C40F", 241, 196, 15)]
    [InlineData("#3498DB", 52, 152, 219)]
    [InlineData("#E67E22", 230, 126, 34)]
    public void ParseColor_CardColorHexValues_ReturnsCorrectRgb(string hex, byte r, byte g, byte b)
    {
        var color = ClipboardWindowHelpers.ParseColor(hex);

        Assert.Equal(r, color.R);
        Assert.Equal(g, color.G);
        Assert.Equal(b, color.B);
    }

    [Theory]
    [InlineData("#E74C3C")]
    [InlineData("#2ECC71")]
    [InlineData("#000000")]
    public void ParseColor_AlwaysSetsFullAlpha(string hex)
    {
        var color = ClipboardWindowHelpers.ParseColor(hex);

        Assert.Equal(255, color.A);
    }

    [Theory]
    [InlineData("E74C3C", 231, 76, 60)]
    [InlineData("000000", 0, 0, 0)]
    [InlineData("FFFFFF", 255, 255, 255)]
    public void ParseColor_WithoutHashPrefix_ParsesCorrectly(string hex, byte r, byte g, byte b)
    {
        var color = ClipboardWindowHelpers.ParseColor(hex);

        Assert.Equal(r, color.R);
        Assert.Equal(g, color.G);
        Assert.Equal(b, color.B);
    }

    [Fact]
    public void ParseColor_Black_ReturnsZeroRgb()
    {
        var color = ClipboardWindowHelpers.ParseColor("#000000");

        Assert.Equal(0, color.R);
        Assert.Equal(0, color.G);
        Assert.Equal(0, color.B);
        Assert.Equal(255, color.A);
    }

    [Fact]
    public void ParseColor_White_ReturnsMaxRgb()
    {
        var color = ClipboardWindowHelpers.ParseColor("#FFFFFF");

        Assert.Equal(255, color.R);
        Assert.Equal(255, color.G);
        Assert.Equal(255, color.B);
        Assert.Equal(255, color.A);
    }

    [Fact]
    public void ParseColor_HexIsCaseInsensitive()
    {
        var lower = ClipboardWindowHelpers.ParseColor("#e74c3c");
        var upper = ClipboardWindowHelpers.ParseColor("#E74C3C");

        Assert.Equal(lower.R, upper.R);
        Assert.Equal(lower.G, upper.G);
        Assert.Equal(lower.B, upper.B);
    }

    [Theory]
    [InlineData("#FF0000", 255, 0, 0)]
    [InlineData("#00FF00", 0, 255, 0)]
    [InlineData("#0000FF", 0, 0, 255)]
    public void ParseColor_PrimaryColors_ReturnsExpected(string hex, byte r, byte g, byte b)
    {
        var color = ClipboardWindowHelpers.ParseColor(hex);

        Assert.Equal(r, color.R);
        Assert.Equal(g, color.G);
        Assert.Equal(b, color.B);
    }

    #endregion
}
