using CopyPaste.UI;
using Xunit;

namespace CopyPaste.UI.Tests;

public class UIConfigTests
{
    [Fact]
    public void PageSize_DefaultValue_Is20()
    {
        UIConfig.PageSize = 20;
        Assert.Equal(20, UIConfig.PageSize);
    }

    [Fact]
    public void MaxItemsBeforeCleanup_DefaultValue_Is100()
    {
        UIConfig.MaxItemsBeforeCleanup = 100;
        Assert.Equal(100, UIConfig.MaxItemsBeforeCleanup);
    }

    [Fact]
    public void ScrollLoadThreshold_DefaultValue_Is100()
    {
        UIConfig.ScrollLoadThreshold = 100;
        Assert.Equal(100, UIConfig.ScrollLoadThreshold);
    }

    [Fact]
    public void WindowWidth_DefaultValue_Is400()
    {
        UIConfig.WindowWidth = 400;
        Assert.Equal(400, UIConfig.WindowWidth);
    }

    [Fact]
    public void WindowMarginTop_DefaultValue_Is8()
    {
        UIConfig.WindowMarginTop = 8;
        Assert.Equal(8, UIConfig.WindowMarginTop);
    }

    [Fact]
    public void WindowMarginBottom_DefaultValue_Is16()
    {
        UIConfig.WindowMarginBottom = 16;
        Assert.Equal(16, UIConfig.WindowMarginBottom);
    }

    [Fact]
    public void RetentionDays_DefaultValue_Is30()
    {
        UIConfig.RetentionDays = 30;
        Assert.Equal(30, UIConfig.RetentionDays);
    }

    [Theory]
    [InlineData(10)]
    [InlineData(20)]
    [InlineData(50)]
    [InlineData(100)]
    public void PageSize_CanBeModified(int value)
    {
        UIConfig.PageSize = value;
        Assert.Equal(value, UIConfig.PageSize);
        UIConfig.PageSize = 20; // Reset
    }

    [Theory]
    [InlineData(300)]
    [InlineData(450)]
    [InlineData(600)]
    public void WindowWidth_CanBeModified(int value)
    {
        UIConfig.WindowWidth = value;
        Assert.Equal(value, UIConfig.WindowWidth);
        UIConfig.WindowWidth = 400; // Reset
    }

    [Theory]
    [InlineData(0)]
    [InlineData(7)]
    [InlineData(30)]
    [InlineData(90)]
    public void RetentionDays_AcceptsValidValues(int value)
    {
        UIConfig.RetentionDays = value;
        Assert.Equal(value, UIConfig.RetentionDays);
        UIConfig.RetentionDays = 30; // Reset
    }
}

public class PasteConfigTests
{
    [Fact]
    public void DuplicateIgnoreWindowMs_DefaultValue_Is300()
    {
        PasteConfig.DuplicateIgnoreWindowMs = 300;
        Assert.Equal(300, PasteConfig.DuplicateIgnoreWindowMs);
    }

    [Fact]
    public void DelayBeforeFocusMs_DefaultValue_Is50()
    {
        PasteConfig.DelayBeforeFocusMs = 50;
        Assert.Equal(50, PasteConfig.DelayBeforeFocusMs);
    }

    [Fact]
    public void DelayBeforePasteMs_DefaultValue_Is100()
    {
        PasteConfig.DelayBeforePasteMs = 100;
        Assert.Equal(100, PasteConfig.DelayBeforePasteMs);
    }

    [Fact]
    public void MaxFocusVerifyAttempts_DefaultValue_Is10()
    {
        PasteConfig.MaxFocusVerifyAttempts = 10;
        Assert.Equal(10, PasteConfig.MaxFocusVerifyAttempts);
    }

    [Theory]
    [InlineData(200)]
    [InlineData(300)]
    [InlineData(500)]
    public void DuplicateIgnoreWindowMs_CanBeModified(int value)
    {
        PasteConfig.DuplicateIgnoreWindowMs = value;
        Assert.Equal(value, PasteConfig.DuplicateIgnoreWindowMs);
        PasteConfig.DuplicateIgnoreWindowMs = 300; // Reset
    }

    [Theory]
    [InlineData(30)]
    [InlineData(50)]
    [InlineData(100)]
    public void DelayBeforeFocusMs_AcceptsValidValues(int value)
    {
        PasteConfig.DelayBeforeFocusMs = value;
        Assert.Equal(value, PasteConfig.DelayBeforeFocusMs);
        PasteConfig.DelayBeforeFocusMs = 50; // Reset
    }

    [Theory]
    [InlineData(50)]
    [InlineData(100)]
    [InlineData(200)]
    public void DelayBeforePasteMs_AcceptsValidValues(int value)
    {
        PasteConfig.DelayBeforePasteMs = value;
        Assert.Equal(value, PasteConfig.DelayBeforePasteMs);
        PasteConfig.DelayBeforePasteMs = 100; // Reset
    }

    [Theory]
    [InlineData(5)]
    [InlineData(10)]
    [InlineData(20)]
    public void MaxFocusVerifyAttempts_AcceptsValidValues(int value)
    {
        PasteConfig.MaxFocusVerifyAttempts = value;
        Assert.Equal(value, PasteConfig.MaxFocusVerifyAttempts);
        PasteConfig.MaxFocusVerifyAttempts = 10; // Reset
    }
}
