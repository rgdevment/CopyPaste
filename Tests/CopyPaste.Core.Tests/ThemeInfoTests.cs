using CopyPaste.Core.Themes;
using Xunit;

namespace CopyPaste.Core.Tests;

public class ThemeInfoTests
{
    [Fact]
    public void Constructor_SetsAllProperties()
    {
        var info = new ThemeInfo("test.id", "Test Theme", "1.0.0", "Test Author", false);

        Assert.Equal("test.id", info.Id);
        Assert.Equal("Test Theme", info.Name);
        Assert.Equal("1.0.0", info.Version);
        Assert.Equal("Test Author", info.Author);
        Assert.False(info.IsCommunity);
    }

    [Fact]
    public void Constructor_CommunityTheme_SetsIsCommunityTrue()
    {
        var info = new ThemeInfo("community.theme", "Community", "2.0.0", "Author", true);

        Assert.True(info.IsCommunity);
    }

    [Fact]
    public void Equality_SameValues_AreEqual()
    {
        var info1 = new ThemeInfo("test.id", "Test", "1.0.0", "Author", false);
        var info2 = new ThemeInfo("test.id", "Test", "1.0.0", "Author", false);

        Assert.Equal(info1, info2);
    }

    [Fact]
    public void Equality_DifferentId_AreNotEqual()
    {
        var info1 = new ThemeInfo("test.id1", "Test", "1.0.0", "Author", false);
        var info2 = new ThemeInfo("test.id2", "Test", "1.0.0", "Author", false);

        Assert.NotEqual(info1, info2);
    }

    [Fact]
    public void Equality_DifferentName_AreNotEqual()
    {
        var info1 = new ThemeInfo("test.id", "Test1", "1.0.0", "Author", false);
        var info2 = new ThemeInfo("test.id", "Test2", "1.0.0", "Author", false);

        Assert.NotEqual(info1, info2);
    }

    [Fact]
    public void Equality_DifferentIsCommunity_AreNotEqual()
    {
        var info1 = new ThemeInfo("test.id", "Test", "1.0.0", "Author", false);
        var info2 = new ThemeInfo("test.id", "Test", "1.0.0", "Author", true);

        Assert.NotEqual(info1, info2);
    }

    [Fact]
    public void GetHashCode_SameValues_SameHash()
    {
        var info1 = new ThemeInfo("test.id", "Test", "1.0.0", "Author", false);
        var info2 = new ThemeInfo("test.id", "Test", "1.0.0", "Author", false);

        Assert.Equal(info1.GetHashCode(), info2.GetHashCode());
    }

    [Fact]
    public void ToString_ContainsId()
    {
        var info = new ThemeInfo("copypaste.default", "Default", "1.0.0", "CopyPaste", false);
        var str = info.ToString();

        Assert.Contains("copypaste.default", str, System.StringComparison.Ordinal);
    }

    [Fact]
    public void With_ReturnsModifiedCopy()
    {
        var original = new ThemeInfo("test.id", "Test", "1.0.0", "Author", false);
        var modified = original with { Name = "Modified" };

        Assert.Equal("Modified", modified.Name);
        Assert.Equal("Test", original.Name);
        Assert.Equal("test.id", modified.Id);
    }
}
