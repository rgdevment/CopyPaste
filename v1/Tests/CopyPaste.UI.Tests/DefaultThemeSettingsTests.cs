using CopyPaste.UI.Themes;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class DefaultThemeSettingsTests
{
    #region Default Values

    [Fact]
    public void DefaultSettings_WindowWidth_Is400()
    {
        var settings = new DefaultThemeSettings();
        Assert.Equal(400, settings.WindowWidth);
    }

    [Fact]
    public void DefaultSettings_WindowMarginTop_Is8()
    {
        var settings = new DefaultThemeSettings();
        Assert.Equal(8, settings.WindowMarginTop);
    }

    [Fact]
    public void DefaultSettings_WindowMarginBottom_Is16()
    {
        var settings = new DefaultThemeSettings();
        Assert.Equal(16, settings.WindowMarginBottom);
    }

    [Fact]
    public void DefaultSettings_CardMinLines_Is3()
    {
        var settings = new DefaultThemeSettings();
        Assert.Equal(3, settings.CardMinLines);
    }

    [Fact]
    public void DefaultSettings_CardMaxLines_Is9()
    {
        var settings = new DefaultThemeSettings();
        Assert.Equal(9, settings.CardMaxLines);
    }

    [Fact]
    public void DefaultSettings_PinWindow_IsFalse()
    {
        var settings = new DefaultThemeSettings();
        Assert.False(settings.PinWindow);
    }

    [Fact]
    public void DefaultSettings_ScrollToTopOnPaste_IsTrue()
    {
        var settings = new DefaultThemeSettings();
        Assert.True(settings.ScrollToTopOnPaste);
    }

    [Fact]
    public void DefaultSettings_ResetScrollOnShow_IsTrue()
    {
        var settings = new DefaultThemeSettings();
        Assert.True(settings.ResetScrollOnShow);
    }

    [Fact]
    public void DefaultSettings_ResetFilterModeOnShow_IsTrue()
    {
        var settings = new DefaultThemeSettings();
        Assert.True(settings.ResetFilterModeOnShow);
    }

    [Fact]
    public void DefaultSettings_ResetContentFilterOnShow_IsTrue()
    {
        var settings = new DefaultThemeSettings();
        Assert.True(settings.ResetContentFilterOnShow);
    }

    [Fact]
    public void DefaultSettings_ResetCategoryFilterOnShow_IsTrue()
    {
        var settings = new DefaultThemeSettings();
        Assert.True(settings.ResetCategoryFilterOnShow);
    }

    [Fact]
    public void DefaultSettings_ResetTypeFilterOnShow_IsTrue()
    {
        var settings = new DefaultThemeSettings();
        Assert.True(settings.ResetTypeFilterOnShow);
    }

    #endregion

    #region Property Setters

    [Fact]
    public void AllProperties_CanBeModified()
    {
        var settings = new DefaultThemeSettings
        {
            WindowWidth = 500,
            WindowMarginTop = 10,
            WindowMarginBottom = 20,
            CardMinLines = 2,
            CardMaxLines = 12,
            PinWindow = true,
            ScrollToTopOnPaste = false,
            ResetScrollOnShow = false,
            ResetFilterModeOnShow = false,
            ResetContentFilterOnShow = false,
            ResetCategoryFilterOnShow = false,
            ResetTypeFilterOnShow = false
        };

        Assert.Equal(500, settings.WindowWidth);
        Assert.Equal(10, settings.WindowMarginTop);
        Assert.Equal(20, settings.WindowMarginBottom);
        Assert.Equal(2, settings.CardMinLines);
        Assert.Equal(12, settings.CardMaxLines);
        Assert.True(settings.PinWindow);
        Assert.False(settings.ScrollToTopOnPaste);
        Assert.False(settings.ResetScrollOnShow);
        Assert.False(settings.ResetFilterModeOnShow);
        Assert.False(settings.ResetContentFilterOnShow);
        Assert.False(settings.ResetCategoryFilterOnShow);
        Assert.False(settings.ResetTypeFilterOnShow);
    }

    [Fact]
    public void Properties_IndividualSetters_Work()
    {
        var settings = new DefaultThemeSettings();

        settings.WindowWidth = 800;
        Assert.Equal(800, settings.WindowWidth);

        settings.WindowMarginTop = 15;
        Assert.Equal(15, settings.WindowMarginTop);

        settings.WindowMarginBottom = 25;
        Assert.Equal(25, settings.WindowMarginBottom);

        settings.CardMinLines = 1;
        Assert.Equal(1, settings.CardMinLines);

        settings.CardMaxLines = 20;
        Assert.Equal(20, settings.CardMaxLines);

        settings.PinWindow = true;
        Assert.True(settings.PinWindow);

        settings.ScrollToTopOnPaste = false;
        Assert.False(settings.ScrollToTopOnPaste);

        settings.ResetScrollOnShow = false;
        Assert.False(settings.ResetScrollOnShow);

        settings.ResetFilterModeOnShow = false;
        Assert.False(settings.ResetFilterModeOnShow);

        settings.ResetContentFilterOnShow = false;
        Assert.False(settings.ResetContentFilterOnShow);

        settings.ResetCategoryFilterOnShow = false;
        Assert.False(settings.ResetCategoryFilterOnShow);

        settings.ResetTypeFilterOnShow = false;
        Assert.False(settings.ResetTypeFilterOnShow);
    }

    #endregion
}
