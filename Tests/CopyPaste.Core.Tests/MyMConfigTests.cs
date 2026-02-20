using System.Collections.Generic;
using System.Text.Json;
using Xunit;

namespace CopyPaste.Core.Tests;

public class MyMConfigTests
{
    #region Default Values

    [Fact]
    public void DefaultConfig_PreferredLanguage_IsAuto()
    {
        var config = new MyMConfig();
        Assert.Equal("auto", config.PreferredLanguage);
    }

    [Fact]
    public void DefaultConfig_RunOnStartup_IsTrue()
    {
        var config = new MyMConfig();
        Assert.True(config.RunOnStartup);
    }

    [Fact]
    public void DefaultConfig_ThemeId_IsDefault()
    {
        var config = new MyMConfig();
        Assert.Equal("copypaste.default", config.ThemeId);
    }

    [Fact]
    public void DefaultConfig_UseCtrlKey_IsFalse()
    {
        var config = new MyMConfig();
        Assert.False(config.UseCtrlKey);
    }

    [Fact]
    public void DefaultConfig_UseWinKey_IsTrue()
    {
        var config = new MyMConfig();
        Assert.True(config.UseWinKey);
    }

    [Fact]
    public void DefaultConfig_UseAltKey_IsTrue()
    {
        var config = new MyMConfig();
        Assert.True(config.UseAltKey);
    }

    [Fact]
    public void DefaultConfig_UseShiftKey_IsFalse()
    {
        var config = new MyMConfig();
        Assert.False(config.UseShiftKey);
    }

    [Fact]
    public void DefaultConfig_VirtualKey_Is0x56()
    {
        var config = new MyMConfig();
        Assert.Equal(0x56u, config.VirtualKey);
    }

    [Fact]
    public void DefaultConfig_KeyName_IsV()
    {
        var config = new MyMConfig();
        Assert.Equal("V", config.KeyName);
    }

    [Fact]
    public void DefaultConfig_PageSize_Is30()
    {
        var config = new MyMConfig();
        Assert.Equal(30, config.PageSize);
    }

    [Fact]
    public void DefaultConfig_MaxItemsBeforeCleanup_Is100()
    {
        var config = new MyMConfig();
        Assert.Equal(100, config.MaxItemsBeforeCleanup);
    }

    [Fact]
    public void DefaultConfig_ScrollLoadThreshold_Is400()
    {
        var config = new MyMConfig();
        Assert.Equal(400, config.ScrollLoadThreshold);
    }

    [Fact]
    public void DefaultConfig_ColorLabels_IsNull()
    {
        var config = new MyMConfig();
        Assert.Null(config.ColorLabels);
    }

    [Fact]
    public void DefaultConfig_RetentionDays_Is30()
    {
        var config = new MyMConfig();
        Assert.Equal(30, config.RetentionDays);
    }

    [Fact]
    public void DefaultConfig_LastBackupDateUtc_IsNull()
    {
        var config = new MyMConfig();
        Assert.Null(config.LastBackupDateUtc);
    }

    [Fact]
    public void DefaultConfig_DuplicateIgnoreWindowMs_Is450()
    {
        var config = new MyMConfig();
        Assert.Equal(450, config.DuplicateIgnoreWindowMs);
    }

    [Fact]
    public void DefaultConfig_DelayBeforeFocusMs_Is100()
    {
        var config = new MyMConfig();
        Assert.Equal(100, config.DelayBeforeFocusMs);
    }

    [Fact]
    public void DefaultConfig_DelayBeforePasteMs_Is180()
    {
        var config = new MyMConfig();
        Assert.Equal(180, config.DelayBeforePasteMs);
    }

    [Fact]
    public void DefaultConfig_MaxFocusVerifyAttempts_Is15()
    {
        var config = new MyMConfig();
        Assert.Equal(15, config.MaxFocusVerifyAttempts);
    }

    [Fact]
    public void DefaultConfig_ThumbnailWidth_Is170()
    {
        var config = new MyMConfig();
        Assert.Equal(170, config.ThumbnailWidth);
    }

    [Fact]
    public void DefaultConfig_ThumbnailQualityPng_Is80()
    {
        var config = new MyMConfig();
        Assert.Equal(80, config.ThumbnailQualityPng);
    }

    [Fact]
    public void DefaultConfig_ThumbnailQualityJpeg_Is80()
    {
        var config = new MyMConfig();
        Assert.Equal(80, config.ThumbnailQualityJpeg);
    }

    [Fact]
    public void DefaultConfig_ThumbnailGCThreshold_Is1000000()
    {
        var config = new MyMConfig();
        Assert.Equal(1_000_000, config.ThumbnailGCThreshold);
    }

    [Fact]
    public void DefaultConfig_ThumbnailUIDecodeHeight_Is95()
    {
        var config = new MyMConfig();
        Assert.Equal(95, config.ThumbnailUIDecodeHeight);
    }

    #endregion

    #region Property Setters

    [Fact]
    public void AllProperties_CanBeModified()
    {
        var config = new MyMConfig
        {
            PreferredLanguage = "fr-FR",
            RunOnStartup = false,
            ThemeId = "custom.theme",
            UseCtrlKey = true,
            UseWinKey = false,
            UseAltKey = false,
            UseShiftKey = true,
            VirtualKey = 0x43,
            KeyName = "C",
            PageSize = 50,
            MaxItemsBeforeCleanup = 200,
            ScrollLoadThreshold = 600,
            ColorLabels = new Dictionary<string, string> { { "Red", "Urgent" } },
            RetentionDays = 90,
            LastBackupDateUtc = new System.DateTime(2025, 1, 1, 0, 0, 0, System.DateTimeKind.Utc),
            DuplicateIgnoreWindowMs = 600,
            DelayBeforeFocusMs = 200,
            DelayBeforePasteMs = 300,
            MaxFocusVerifyAttempts = 20,
            ThumbnailWidth = 200,
            ThumbnailQualityPng = 90,
            ThumbnailQualityJpeg = 70,
            ThumbnailGCThreshold = 2_000_000,
            ThumbnailUIDecodeHeight = 120
        };

        Assert.Equal("fr-FR", config.PreferredLanguage);
        Assert.False(config.RunOnStartup);
        Assert.Equal("custom.theme", config.ThemeId);
        Assert.True(config.UseCtrlKey);
        Assert.False(config.UseWinKey);
        Assert.False(config.UseAltKey);
        Assert.True(config.UseShiftKey);
        Assert.Equal(0x43u, config.VirtualKey);
        Assert.Equal("C", config.KeyName);
        Assert.Equal(50, config.PageSize);
        Assert.Equal(200, config.MaxItemsBeforeCleanup);
        Assert.Equal(600, config.ScrollLoadThreshold);
        Assert.NotNull(config.ColorLabels);
        Assert.Equal("Urgent", config.ColorLabels["Red"]);
        Assert.Equal(90, config.RetentionDays);
        Assert.NotNull(config.LastBackupDateUtc);
        Assert.Equal(600, config.DuplicateIgnoreWindowMs);
        Assert.Equal(200, config.DelayBeforeFocusMs);
        Assert.Equal(300, config.DelayBeforePasteMs);
        Assert.Equal(20, config.MaxFocusVerifyAttempts);
        Assert.Equal(200, config.ThumbnailWidth);
        Assert.Equal(90, config.ThumbnailQualityPng);
        Assert.Equal(70, config.ThumbnailQualityJpeg);
        Assert.Equal(2_000_000, config.ThumbnailGCThreshold);
        Assert.Equal(120, config.ThumbnailUIDecodeHeight);
    }

    #endregion

    #region JSON Serialization

    [Fact]
    public void JsonSerialization_RoundTrip_PreservesValues()
    {
        var original = new MyMConfig
        {
            PreferredLanguage = "es-CL",
            RunOnStartup = false,
            PageSize = 50,
            RetentionDays = 60,
            ThumbnailWidth = 200
        };

        var json = JsonSerializer.Serialize(original, MyMConfigJsonContext.Default.MyMConfig);
        var deserialized = JsonSerializer.Deserialize(json, MyMConfigJsonContext.Default.MyMConfig);

        Assert.NotNull(deserialized);
        Assert.Equal("es-CL", deserialized.PreferredLanguage);
        Assert.False(deserialized.RunOnStartup);
        Assert.Equal(50, deserialized.PageSize);
        Assert.Equal(60, deserialized.RetentionDays);
        Assert.Equal(200, deserialized.ThumbnailWidth);
    }

    [Fact]
    public void JsonSerialization_DefaultConfig_RoundTrips()
    {
        var original = new MyMConfig();

        var json = JsonSerializer.Serialize(original, MyMConfigJsonContext.Default.MyMConfig);
        var deserialized = JsonSerializer.Deserialize(json, MyMConfigJsonContext.Default.MyMConfig);

        Assert.NotNull(deserialized);
        Assert.Equal(original.PreferredLanguage, deserialized.PreferredLanguage);
        Assert.Equal(original.PageSize, deserialized.PageSize);
        Assert.Equal(original.RetentionDays, deserialized.RetentionDays);
    }

    [Fact]
    public void JsonSerialization_WithColorLabels_RoundTrips()
    {
        var original = new MyMConfig
        {
            ColorLabels = new Dictionary<string, string>
            {
                { "Red", "Urgent" },
                { "Green", "Personal" },
                { "Blue", "Work" }
            }
        };

        var json = JsonSerializer.Serialize(original, MyMConfigJsonContext.Default.MyMConfig);
        var deserialized = JsonSerializer.Deserialize(json, MyMConfigJsonContext.Default.MyMConfig);

        Assert.NotNull(deserialized?.ColorLabels);
        Assert.Equal(3, deserialized.ColorLabels.Count);
        Assert.Equal("Urgent", deserialized.ColorLabels["Red"]);
        Assert.Equal("Personal", deserialized.ColorLabels["Green"]);
        Assert.Equal("Work", deserialized.ColorLabels["Blue"]);
    }

    [Fact]
    public void JsonDeserialization_MissingProperties_UsesDefaults()
    {
        var json = "{}";
        var config = JsonSerializer.Deserialize(json, MyMConfigJsonContext.Default.MyMConfig);

        Assert.NotNull(config);
        Assert.Equal("auto", config.PreferredLanguage);
        Assert.True(config.RunOnStartup);
        Assert.Equal(30, config.PageSize);
    }

    [Fact]
    public void JsonDeserialization_PartialProperties_MergesWithDefaults()
    {
        var json = """{"PageSize": 100, "RetentionDays": 90}""";
        var config = JsonSerializer.Deserialize(json, MyMConfigJsonContext.Default.MyMConfig);

        Assert.NotNull(config);
        Assert.Equal(100, config.PageSize);
        Assert.Equal(90, config.RetentionDays);
        Assert.Equal("auto", config.PreferredLanguage);
        Assert.True(config.RunOnStartup);
    }

    #endregion
}
