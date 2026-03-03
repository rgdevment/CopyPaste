using CopyPaste.UI.Themes;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class CompactSettingsTests
{
    #region Default Values

    [Fact]
    public void DefaultSettings_PopupWidth_Is368()
    {
        var settings = new CompactSettings();
        Assert.Equal(368, settings.PopupWidth);
    }

    [Fact]
    public void DefaultSettings_PopupHeight_Is480()
    {
        var settings = new CompactSettings();
        Assert.Equal(480, settings.PopupHeight);
    }

    [Fact]
    public void DefaultSettings_CardMinLines_Is2()
    {
        var settings = new CompactSettings();
        Assert.Equal(2, settings.CardMinLines);
    }

    [Fact]
    public void DefaultSettings_CardMaxLines_Is5()
    {
        var settings = new CompactSettings();
        Assert.Equal(5, settings.CardMaxLines);
    }

    [Fact]
    public void DefaultSettings_PinWindow_IsFalse()
    {
        var settings = new CompactSettings();
        Assert.False(settings.PinWindow);
    }

    [Fact]
    public void DefaultSettings_ScrollToTopOnPaste_IsTrue()
    {
        var settings = new CompactSettings();
        Assert.True(settings.ScrollToTopOnPaste);
    }

    [Fact]
    public void DefaultSettings_HideOnDeactivate_IsTrue()
    {
        var settings = new CompactSettings();
        Assert.True(settings.HideOnDeactivate);
    }

    [Fact]
    public void DefaultSettings_ResetScrollOnShow_IsTrue()
    {
        var settings = new CompactSettings();
        Assert.True(settings.ResetScrollOnShow);
    }

    [Fact]
    public void DefaultSettings_ResetSearchOnShow_IsTrue()
    {
        var settings = new CompactSettings();
        Assert.True(settings.ResetSearchOnShow);
    }

    [Fact]
    public void DefaultSettings_ResetFilterModeOnShow_IsTrue()
    {
        var settings = new CompactSettings();
        Assert.True(settings.ResetFilterModeOnShow);
    }

    [Fact]
    public void DefaultSettings_ResetCategoryFilterOnShow_IsTrue()
    {
        var settings = new CompactSettings();
        Assert.True(settings.ResetCategoryFilterOnShow);
    }

    [Fact]
    public void DefaultSettings_ResetTypeFilterOnShow_IsTrue()
    {
        var settings = new CompactSettings();
        Assert.True(settings.ResetTypeFilterOnShow);
    }

    #endregion

    #region Property Setters

    [Fact]
    public void AllProperties_CanBeModified()
    {
        var settings = new CompactSettings
        {
            PopupWidth = 500,
            PopupHeight = 600,
            CardMinLines = 1,
            CardMaxLines = 10,
            PinWindow = true,
            ScrollToTopOnPaste = false,
            HideOnDeactivate = false,
            ResetScrollOnShow = false,
            ResetSearchOnShow = false,
            ResetFilterModeOnShow = false,
            ResetCategoryFilterOnShow = false,
            ResetTypeFilterOnShow = false
        };

        Assert.Equal(500, settings.PopupWidth);
        Assert.Equal(600, settings.PopupHeight);
        Assert.Equal(1, settings.CardMinLines);
        Assert.Equal(10, settings.CardMaxLines);
        Assert.True(settings.PinWindow);
        Assert.False(settings.ScrollToTopOnPaste);
        Assert.False(settings.HideOnDeactivate);
        Assert.False(settings.ResetScrollOnShow);
        Assert.False(settings.ResetSearchOnShow);
        Assert.False(settings.ResetFilterModeOnShow);
        Assert.False(settings.ResetCategoryFilterOnShow);
        Assert.False(settings.ResetTypeFilterOnShow);
    }

    [Fact]
    public void Properties_IndividualSetters_Work()
    {
        var settings = new CompactSettings();

        settings.PopupWidth = 500;
        Assert.Equal(500, settings.PopupWidth);

        settings.PopupHeight = 600;
        Assert.Equal(600, settings.PopupHeight);

        settings.CardMinLines = 1;
        Assert.Equal(1, settings.CardMinLines);

        settings.CardMaxLines = 8;
        Assert.Equal(8, settings.CardMaxLines);

        settings.PinWindow = true;
        Assert.True(settings.PinWindow);

        settings.HideOnDeactivate = false;
        Assert.False(settings.HideOnDeactivate);

        settings.ResetSearchOnShow = false;
        Assert.False(settings.ResetSearchOnShow);
    }

    #endregion
}
