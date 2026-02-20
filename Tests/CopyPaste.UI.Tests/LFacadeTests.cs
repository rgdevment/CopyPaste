using CopyPaste.UI.Localization;
using System.Diagnostics.CodeAnalysis;
using Xunit;

namespace CopyPaste.UI.Tests;

[Collection("L Facade Tests")]
[SuppressMessage("Reliability", "CA2000:Dispose objects before losing scope",
    Justification = "L.Initialize takes ownership; L.Dispose() called in Dispose()")]
public sealed class LFacadeTests : IDisposable
{
    public void Dispose()
    {
        L.Dispose();
    }

    #region Initialize Tests

    [Fact]
    public void Initialize_WithNull_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() => L.Initialize(null!));
    }

    [Fact]
    public void Initialize_WithValidService_Succeeds()
    {
        var service = new LocalizationService("en-US");

        var ex = Record.Exception(() => L.Initialize(service));

        Assert.Null(ex);
    }

    #endregion

    #region Get Before Initialization Tests

    [Fact]
    public void Get_BeforeInitialize_ReturnsBracketedKey()
    {
        var value = L.Get("search_placeholder");

        Assert.Equal("[search_placeholder]", value);
    }

    [Fact]
    public void CurrentLanguage_BeforeInitialize_ReturnsEnUS()
    {
        Assert.Equal("en-US", L.CurrentLanguage);
    }

    #endregion

    #region Get After Initialization Tests

    [Fact]
    public void Get_AfterInitialize_ReturnsValue()
    {
        var service = new LocalizationService("en-US");
        L.Initialize(service);

        var value = L.Get("window.title");

        Assert.NotNull(value);
        Assert.NotEmpty(value);
        Assert.DoesNotContain("[", value, StringComparison.Ordinal);
    }

    [Fact]
    public void Get_UnknownKey_ReturnsBracketedKey()
    {
        var service = new LocalizationService("en-US");
        L.Initialize(service);

        var value = L.Get("unknown.key.that.does.not.exist");

        Assert.Equal("[unknown.key.that.does.not.exist]", value);
    }

    [Fact]
    public void Get_UnknownKey_WithDefault_ReturnsDefault()
    {
        var service = new LocalizationService("en-US");
        L.Initialize(service);

        var value = L.Get("unknown.key.that.does.not.exist", "fallback value");

        Assert.Equal("fallback value", value);
    }

    [Fact]
    public void CurrentLanguage_AfterInitialize_ReturnsLanguage()
    {
        var service = new LocalizationService("en-US");
        L.Initialize(service);

        Assert.Equal("en-US", L.CurrentLanguage);
    }

    [Fact]
    public void Get_WhenServiceExternallyDisposed_ReturnsBracketedKey()
    {
        var service = new LocalizationService("en-US");
        L.Initialize(service);
        service.Dispose();

        var value = L.Get("window.title");

        Assert.Equal("[window.title]", value);
    }

    [Fact]
    public void Get_WhenServiceExternallyDisposed_WithDefault_ReturnsDefault()
    {
        var service = new LocalizationService("en-US");
        L.Initialize(service);
        service.Dispose();

        var value = L.Get("window.title", "fallback");

        Assert.Equal("fallback", value);
    }

    #endregion

    #region Dispose Tests

    [Fact]
    public void Dispose_DoesNotThrow()
    {
        var service = new LocalizationService("en-US");
        L.Initialize(service);

        var ex = Record.Exception(() => L.Dispose());

        Assert.Null(ex);
    }

    #endregion
}
