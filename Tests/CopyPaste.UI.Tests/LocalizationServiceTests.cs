using CopyPaste.UI.Localization;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class LocalizationServiceTests : IDisposable
{
    private LocalizationService? _service;

    #region Constructor Tests

    [Fact]
    public void Constructor_WithAutoPreference_ResolvesLanguage()
    {
        _service = new LocalizationService("auto");

        Assert.NotNull(_service.CurrentLanguage);
        Assert.NotEmpty(_service.CurrentLanguage);
    }

    [Fact]
    public void Constructor_WithNull_ResolvesLanguage()
    {
        _service = new LocalizationService(null);

        Assert.NotNull(_service.CurrentLanguage);
    }

    [Fact]
    public void Constructor_WithEnUS_SetsEnglish()
    {
        _service = new LocalizationService("en-US");

        Assert.Equal("en-US", _service.CurrentLanguage);
    }

    [Fact]
    public void Constructor_WithEsCL_SetsSpanish()
    {
        _service = new LocalizationService("es-CL");

        Assert.Equal("es-CL", _service.CurrentLanguage);
    }

    [Fact]
    public void Constructor_WithUnsupportedLanguage_FallsBack()
    {
        _service = new LocalizationService("zh-CN");

        Assert.NotNull(_service.CurrentLanguage);
    }

    #endregion

    #region Get Tests

    [Fact]
    public void Get_ExistingKey_ReturnsValue()
    {
        _service = new LocalizationService("en-US");

        var value = _service.Get("clipboard.contextMenu.paste");

        Assert.NotNull(value);
        Assert.NotEmpty(value);
        Assert.DoesNotContain("[", value, StringComparison.Ordinal);
    }

    [Fact]
    public void Get_NonExistingKey_ReturnsBracketedKey()
    {
        _service = new LocalizationService("en-US");

        var value = _service.Get("non.existing.key");

        Assert.Equal("[non.existing.key]", value);
    }

    [Fact]
    public void Get_NonExistingKey_WithDefault_ReturnsDefault()
    {
        _service = new LocalizationService("en-US");

        var value = _service.Get("non.existing.key", "fallback");

        Assert.Equal("fallback", value);
    }

    [Fact]
    public void Get_NullKey_ReturnsEmpty()
    {
        _service = new LocalizationService("en-US");

        var value = _service.Get(null!);

        Assert.Equal(string.Empty, value);
    }

    [Fact]
    public void Get_EmptyKey_ReturnsEmpty()
    {
        _service = new LocalizationService("en-US");

        var value = _service.Get(string.Empty);

        Assert.Equal(string.Empty, value);
    }

    [Fact]
    public void Get_NullKey_WithDefault_ReturnsDefault()
    {
        _service = new LocalizationService("en-US");

        var value = _service.Get(null!, "my default");

        Assert.Equal("my default", value);
    }

    [Fact]
    public void Get_SpanishLanguage_ReturnsSpanishValues()
    {
        _service = new LocalizationService("es-CL");

        var value = _service.Get("clipboard.contextMenu.paste");

        Assert.NotNull(value);
        Assert.NotEmpty(value);
    }

    #endregion

    #region Dispose Tests

    [Fact]
    public void Dispose_AfterDispose_GetThrows()
    {
        _service = new LocalizationService("en-US");
        _service.Dispose();

        Assert.Throws<ObjectDisposedException>(() => _service.Get("any.key"));
    }

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        _service = new LocalizationService("en-US");
        _service.Dispose();
        _service.Dispose();
    }

    #endregion

    #region Language Resolution Tests

    [Fact]
    public void CurrentLanguage_IsAlwaysSet()
    {
        _service = new LocalizationService();

        Assert.NotNull(_service.CurrentLanguage);
        Assert.NotEmpty(_service.CurrentLanguage);
    }

    [Fact]
    public void Constructor_LoadsKeysSuccessfully()
    {
        _service = new LocalizationService("en-US");

        var clipboardKey = _service.Get("clipboard.contextMenu.delete");
        Assert.NotEqual("[clipboard.contextMenu.delete]", clipboardKey);
    }

    #endregion

    public void Dispose()
    {
        _service?.Dispose();
    }
}
