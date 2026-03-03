using CopyPaste.Core;
using CopyPaste.UI.Themes;
using System;
using System.IO;
using System.Reflection;
using System.Threading;
using Xunit;

namespace CopyPaste.UI.Tests;

// Single semaphore ensures only one settings test holds the redirected _appDataPath at a time.
internal static class SettingsTestSync
{
    internal static readonly SemaphoreSlim Gate = new(1, 1);
    internal static readonly FieldInfo AppDataPathField =
        typeof(StorageConfig).GetField("_appDataPath", BindingFlags.NonPublic | BindingFlags.Static)!;
}

[Collection("SettingsSerialTests")]
public sealed class CompactSettingsLoadSaveTests : IDisposable
{
    private readonly string _tempDir;
    private readonly string _originalPath;

    public CompactSettingsLoadSaveTests()
    {
        SettingsTestSync.Gate.Wait();
        _tempDir = Path.Combine(Path.GetTempPath(), "CopyPasteTests_Compact", Guid.NewGuid().ToString());
        Directory.CreateDirectory(Path.Combine(_tempDir, "config"));
        _originalPath = (SettingsTestSync.AppDataPathField.GetValue(null) as string)!;
        SettingsTestSync.AppDataPathField.SetValue(null, _tempDir);
    }

    public void Dispose()
    {
        SettingsTestSync.AppDataPathField.SetValue(null, _originalPath);
        try { Directory.Delete(_tempDir, recursive: true); } catch { }
        SettingsTestSync.Gate.Release();
    }

    [Fact]
    public void Load_WhenNoFileExists_ReturnsDefaultValues()
    {
        var settings = CompactSettings.Load();

        Assert.NotNull(settings);
        Assert.Equal(368, settings.PopupWidth);
    }

    [Fact]
    public void Load_WhenValidFileExists_ReturnsSavedValues()
    {
        CompactSettings.Save(new CompactSettings { PopupWidth = 500, PopupHeight = 600 });

        var loaded = CompactSettings.Load();

        Assert.Equal(500, loaded.PopupWidth);
        Assert.Equal(600, loaded.PopupHeight);
    }

    [Fact]
    public void Load_WhenFileHasInvalidJson_ReturnsDefaultValues()
    {
        var configFile = Path.Combine(_tempDir, "config", "CompactTheme.json");
        Directory.CreateDirectory(Path.GetDirectoryName(configFile)!);
        File.WriteAllText(configFile, "{ this is: not valid json !!");

        var settings = CompactSettings.Load();

        Assert.NotNull(settings);
        Assert.Equal(368, settings.PopupWidth);
    }

    [Fact]
    public void Save_ReturnsTrueOnSuccess()
    {
        var result = CompactSettings.Save(new CompactSettings());

        Assert.True(result);
    }

    [Fact]
    public void Save_PersistsBooleanProperties()
    {
        CompactSettings.Save(new CompactSettings
        {
            PinWindow = true,
            ScrollToTopOnPaste = false,
            HideOnDeactivate = false,
            ResetSearchOnShow = false
        });

        var loaded = CompactSettings.Load();

        Assert.True(loaded.PinWindow);
        Assert.False(loaded.ScrollToTopOnPaste);
    }

    [Fact]
    public void RoundTrip_PreservesAllWrittenValues()
    {
        CompactSettings.Save(new CompactSettings { PopupWidth = 380, PopupHeight = 520, CardMinLines = 1, CardMaxLines = 10 });

        var loaded = CompactSettings.Load();

        Assert.Equal(380, loaded.PopupWidth);
        Assert.Equal(1, loaded.CardMinLines);
        Assert.Equal(10, loaded.CardMaxLines);
    }

    [Fact]
    public void Save_CreatesConfigDir_WhenMissing()
    {
        Directory.Delete(Path.Combine(_tempDir, "config"), recursive: true);

        var result = CompactSettings.Save(new CompactSettings());

        Assert.True(result);
    }

    [Fact]
    public void Load_WhenFileContainsJsonNull_ReturnsDefaults()
    {
        // JsonSerializer.Deserialize returns null for "null" JSON → Load falls back to defaults
        var configFile = Path.Combine(_tempDir, "config", "CompactTheme.json");
        File.WriteAllText(configFile, "null");

        var settings = CompactSettings.Load();

        Assert.NotNull(settings);
        Assert.Equal(368, settings.PopupWidth);
    }

    [Fact]
    public void Save_ReturnsFalse_WhenFileLocked()
    {
        // Create a locked/read-only file → WriteAllText throws; Save returns false
        var configFile = Path.Combine(_tempDir, "config", "CompactTheme.json");
        File.WriteAllText(configFile, "{}");
        // Hold an exclusive read lock on the file
        using var lockHandle = new FileStream(configFile, FileMode.Open, FileAccess.Read, FileShare.None);

        var result = CompactSettings.Save(new CompactSettings());

        Assert.False(result);
    }
}

[Collection("SettingsSerialTests")]
public sealed class DefaultThemeSettingsLoadSaveTests : IDisposable
{
    private readonly string _tempDir;
    private readonly string _originalPath;

    public DefaultThemeSettingsLoadSaveTests()
    {
        SettingsTestSync.Gate.Wait();
        _tempDir = Path.Combine(Path.GetTempPath(), "CopyPasteTests_Default", Guid.NewGuid().ToString());
        Directory.CreateDirectory(Path.Combine(_tempDir, "config"));
        _originalPath = (SettingsTestSync.AppDataPathField.GetValue(null) as string)!;
        SettingsTestSync.AppDataPathField.SetValue(null, _tempDir);
    }

    public void Dispose()
    {
        SettingsTestSync.AppDataPathField.SetValue(null, _originalPath);
        try { Directory.Delete(_tempDir, recursive: true); } catch { }
        SettingsTestSync.Gate.Release();
    }

    [Fact]
    public void Load_WhenNoFileExists_ReturnsDefaultValues()
    {
        var settings = DefaultThemeSettings.Load();

        Assert.NotNull(settings);
        Assert.Equal(400, settings.WindowWidth);
    }

    [Fact]
    public void Load_WhenValidFileExists_ReturnsSavedValues()
    {
        DefaultThemeSettings.Save(new DefaultThemeSettings { WindowWidth = 600, CardMaxLines = 12 });

        var loaded = DefaultThemeSettings.Load();

        Assert.Equal(600, loaded.WindowWidth);
        Assert.Equal(12, loaded.CardMaxLines);
    }

    [Fact]
    public void Load_WhenFileHasInvalidJson_ReturnsDefaultValues()
    {
        var configFile = Path.Combine(_tempDir, "config", "DefaultTheme.json");
        Directory.CreateDirectory(Path.GetDirectoryName(configFile)!);
        File.WriteAllText(configFile, "INVALID {{{ JSON");

        var settings = DefaultThemeSettings.Load();

        Assert.NotNull(settings);
        Assert.Equal(400, settings.WindowWidth);
    }

    [Fact]
    public void Save_ReturnsTrueOnSuccess()
    {
        var result = DefaultThemeSettings.Save(new DefaultThemeSettings());

        Assert.True(result);
    }

    [Fact]
    public void Save_PersistsMarginValues()
    {
        DefaultThemeSettings.Save(new DefaultThemeSettings { WindowMarginTop = 4, WindowMarginBottom = 20 });

        var loaded = DefaultThemeSettings.Load();

        Assert.Equal(4, loaded.WindowMarginTop);
        Assert.Equal(20, loaded.WindowMarginBottom);
    }

    [Fact]
    public void RoundTrip_PreservesAllWrittenValues()
    {
        DefaultThemeSettings.Save(new DefaultThemeSettings { WindowWidth = 500, CardMinLines = 2, CardMaxLines = 10, PinWindow = true });

        var loaded = DefaultThemeSettings.Load();

        Assert.Equal(500, loaded.WindowWidth);
        Assert.Equal(2, loaded.CardMinLines);
        Assert.True(loaded.PinWindow);
    }

    [Fact]
    public void Save_CreatesConfigDir_WhenMissing()
    {
        Directory.Delete(Path.Combine(_tempDir, "config"), recursive: true);

        var result = DefaultThemeSettings.Save(new DefaultThemeSettings());

        Assert.True(result);
    }

    [Fact]
    public void Load_WhenFileContainsJsonNull_ReturnsDefaults()
    {
        var configFile = Path.Combine(_tempDir, "config", "DefaultTheme.json");
        File.WriteAllText(configFile, "null");

        var settings = DefaultThemeSettings.Load();

        Assert.NotNull(settings);
        Assert.Equal(400, settings.WindowWidth);
    }

    [Fact]
    public void Save_ReturnsFalse_WhenFileLocked()
    {
        var configFile = Path.Combine(_tempDir, "config", "DefaultTheme.json");
        File.WriteAllText(configFile, "{}");
        using var lockHandle = new FileStream(configFile, FileMode.Open, FileAccess.Read, FileShare.None);

        var result = DefaultThemeSettings.Save(new DefaultThemeSettings());

        Assert.False(result);
    }
}
