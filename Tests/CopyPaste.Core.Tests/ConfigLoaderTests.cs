using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class ConfigLoaderTests : IDisposable
{
    private readonly string _basePath;

    public ConfigLoaderTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
        ConfigLoader.ClearCache();
    }

    #region Load Tests

    [Fact]
    public void Load_WithNoConfigFile_ReturnsDefaultConfig()
    {
        var config = ConfigLoader.Load();

        Assert.NotNull(config);
        Assert.Equal("auto", config.PreferredLanguage);
        Assert.True(config.RunOnStartup);
    }

    [Fact]
    public void Load_WithValidConfigFile_ReturnsDeserializedConfig()
    {
        var config = new MyMConfig { PreferredLanguage = "es-CL", RunOnStartup = false, PageSize = 50 };
        var json = JsonSerializer.Serialize(config, MyMConfigJsonContext.Default.MyMConfig);
        Directory.CreateDirectory(StorageConfig.ConfigPath);
        File.WriteAllText(ConfigLoader.ConfigFilePath, json);

        ConfigLoader.ClearCache();
        var loaded = ConfigLoader.Load();

        Assert.Equal("es-CL", loaded.PreferredLanguage);
        Assert.False(loaded.RunOnStartup);
        Assert.Equal(50, loaded.PageSize);
    }

    [Fact]
    public void Load_CachesResult_ReturnsSameInstance()
    {
        var first = ConfigLoader.Load();
        var second = ConfigLoader.Load();

        Assert.Same(first, second);
    }

    [Fact]
    public void Load_WithCorruptedJson_ReturnsDefaultConfig()
    {
        Directory.CreateDirectory(StorageConfig.ConfigPath);
        File.WriteAllText(ConfigLoader.ConfigFilePath, "{ invalid json!!!");

        ConfigLoader.ClearCache();
        var config = ConfigLoader.Load();

        Assert.NotNull(config);
        Assert.Equal("auto", config.PreferredLanguage);
    }

    [Fact]
    public void Config_Property_LoadsOnFirstAccess()
    {
        ConfigLoader.ClearCache();
        var config = ConfigLoader.Config;

        Assert.NotNull(config);
    }

    [Fact]
    public void Config_Property_ReturnsCachedInstance()
    {
        ConfigLoader.ClearCache();
        var first = ConfigLoader.Config;
        var second = ConfigLoader.Config;

        Assert.Same(first, second);
    }

    #endregion

    #region Save Tests

    [Fact]
    public void Save_WritesConfigFile_ReturnsTrue()
    {
        var config = new MyMConfig { PreferredLanguage = "en-US", PageSize = 99 };

        var result = ConfigLoader.Save(config);

        Assert.True(result);
        Assert.True(File.Exists(ConfigLoader.ConfigFilePath));
    }

    [Fact]
    public void Save_FileContainsSerializedConfig()
    {
        var config = new MyMConfig { PreferredLanguage = "es-CL", RetentionDays = 60 };

        ConfigLoader.Save(config);

        var json = File.ReadAllText(ConfigLoader.ConfigFilePath);
        Assert.Contains("es-CL", json, StringComparison.Ordinal);
    }

    [Fact]
    public void Save_UpdatesCache()
    {
        var config = new MyMConfig { PreferredLanguage = "es-CL" };

        ConfigLoader.Save(config);

        Assert.Same(config, ConfigLoader.Config);
    }

    [Fact]
    public void Save_CreatesDirectoryIfNotExists()
    {
        var newPath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(newPath);

        var config = new MyMConfig();
        var result = ConfigLoader.Save(config);

        Assert.True(result);
        Assert.True(Directory.Exists(StorageConfig.ConfigPath));

        try { Directory.Delete(newPath, true); } catch { }
    }

    #endregion

    #region ClearCache Tests

    [Fact]
    public void ClearCache_ForcesReloadOnNextAccess()
    {
        var first = ConfigLoader.Config;
        ConfigLoader.ClearCache();

        var config = new MyMConfig { PageSize = 77 };
        var json = JsonSerializer.Serialize(config, MyMConfigJsonContext.Default.MyMConfig);
        File.WriteAllText(ConfigLoader.ConfigFilePath, json);

        var reloaded = ConfigLoader.Config;

        Assert.Equal(77, reloaded.PageSize);
    }

    #endregion

    #region GetColorLabel Tests

    [Fact]
    public void GetColorLabel_String_WithConfiguredLabel_ReturnsLabel()
    {
        var config = new MyMConfig
        {
            ColorLabels = new Dictionary<string, string> { { "Red", "Urgent" } }
        };
        ConfigLoader.Save(config);
        ConfigLoader.ClearCache();

        var label = ConfigLoader.GetColorLabel("Red");

        Assert.Equal("Urgent", label);
    }

    [Fact]
    public void GetColorLabel_String_WithNoLabels_ReturnsNull()
    {
        var config = new MyMConfig { ColorLabels = null };
        ConfigLoader.Save(config);
        ConfigLoader.ClearCache();

        var label = ConfigLoader.GetColorLabel("Red");

        Assert.Null(label);
    }

    [Fact]
    public void GetColorLabel_String_WithEmptyLabel_ReturnsNull()
    {
        var config = new MyMConfig
        {
            ColorLabels = new Dictionary<string, string> { { "Red", "  " } }
        };
        ConfigLoader.Save(config);
        ConfigLoader.ClearCache();

        var label = ConfigLoader.GetColorLabel("Red");

        Assert.Null(label);
    }

    [Fact]
    public void GetColorLabel_String_WithMissingColor_ReturnsNull()
    {
        var config = new MyMConfig
        {
            ColorLabels = new Dictionary<string, string> { { "Red", "Urgent" } }
        };
        ConfigLoader.Save(config);
        ConfigLoader.ClearCache();

        var label = ConfigLoader.GetColorLabel("Blue");

        Assert.Null(label);
    }

    [Fact]
    public void GetColorLabel_CardColor_WithConfiguredLabel_ReturnsLabel()
    {
        var config = new MyMConfig
        {
            ColorLabels = new Dictionary<string, string> { { "Green", "Personal" } }
        };
        ConfigLoader.Save(config);
        ConfigLoader.ClearCache();

        var label = ConfigLoader.GetColorLabel(CardColor.Green);

        Assert.Equal("Personal", label);
    }

    [Fact]
    public void GetColorLabel_CardColorNone_ReturnsNull()
    {
        var label = ConfigLoader.GetColorLabel(CardColor.None);

        Assert.Null(label);
    }

    [Theory]
    [InlineData(CardColor.Red)]
    [InlineData(CardColor.Green)]
    [InlineData(CardColor.Purple)]
    [InlineData(CardColor.Yellow)]
    [InlineData(CardColor.Blue)]
    [InlineData(CardColor.Orange)]
    public void GetColorLabel_AllNonNoneColors_DoNotThrow(CardColor color)
    {
        var ex = Record.Exception(() => ConfigLoader.GetColorLabel(color));
        Assert.Null(ex);
    }

    #endregion

    #region ConfigFilePath and ConfigFileExists Tests

    [Fact]
    public void ConfigFilePath_ContainsConfigDirectory()
    {
        Assert.Contains(StorageConfig.ConfigPath, ConfigLoader.ConfigFilePath, StringComparison.Ordinal);
    }

    [Fact]
    public void ConfigFilePath_EndsWithMyMJson()
    {
        Assert.EndsWith("MyM.json", ConfigLoader.ConfigFilePath, StringComparison.Ordinal);
    }

    [Fact]
    public void ConfigFileExists_ReturnsFalse_WhenNoFile()
    {
        Assert.False(ConfigLoader.ConfigFileExists);
    }

    [Fact]
    public void ConfigFileExists_ReturnsTrue_WhenFileExists()
    {
        Directory.CreateDirectory(StorageConfig.ConfigPath);
        File.WriteAllText(ConfigLoader.ConfigFilePath, "{}");

        Assert.True(ConfigLoader.ConfigFileExists);
    }

    #endregion

    public void Dispose()
    {
        ConfigLoader.ClearCache();
        try
        {
            if (Directory.Exists(_basePath))
                Directory.Delete(_basePath, recursive: true);
        }
        catch { }
    }
}
