using System;
using System.Text.Json;
using Xunit;

namespace CopyPaste.Core.Tests;

public class BackupManifestTests
{
    #region Default Values

    [Fact]
    public void DefaultManifest_Version_Is1()
    {
        var manifest = new BackupManifest();
        Assert.Equal(1, manifest.Version);
    }

    [Fact]
    public void DefaultManifest_AppVersion_IsEmpty()
    {
        var manifest = new BackupManifest();
        Assert.Equal(string.Empty, manifest.AppVersion);
    }

    [Fact]
    public void DefaultManifest_CreatedAtUtc_IsDefault()
    {
        var manifest = new BackupManifest();
        Assert.Equal(default, manifest.CreatedAtUtc);
    }

    [Fact]
    public void DefaultManifest_ItemCount_IsZero()
    {
        var manifest = new BackupManifest();
        Assert.Equal(0, manifest.ItemCount);
    }

    [Fact]
    public void DefaultManifest_ImageCount_IsZero()
    {
        var manifest = new BackupManifest();
        Assert.Equal(0, manifest.ImageCount);
    }

    [Fact]
    public void DefaultManifest_ThumbnailCount_IsZero()
    {
        var manifest = new BackupManifest();
        Assert.Equal(0, manifest.ThumbnailCount);
    }

    [Fact]
    public void DefaultManifest_HasPinnedItems_IsFalse()
    {
        var manifest = new BackupManifest();
        Assert.False(manifest.HasPinnedItems);
    }

    [Fact]
    public void DefaultManifest_MachineName_IsEmpty()
    {
        var manifest = new BackupManifest();
        Assert.Equal(string.Empty, manifest.MachineName);
    }

    #endregion

    #region Property Setters

    [Fact]
    public void AllProperties_CanBeSet()
    {
        var now = DateTime.UtcNow;
        var manifest = new BackupManifest
        {
            Version = 2,
            AppVersion = "1.5.0",
            CreatedAtUtc = now,
            ItemCount = 100,
            ImageCount = 25,
            ThumbnailCount = 25,
            HasPinnedItems = true,
            MachineName = "MY-PC"
        };

        Assert.Equal(2, manifest.Version);
        Assert.Equal("1.5.0", manifest.AppVersion);
        Assert.Equal(now, manifest.CreatedAtUtc);
        Assert.Equal(100, manifest.ItemCount);
        Assert.Equal(25, manifest.ImageCount);
        Assert.Equal(25, manifest.ThumbnailCount);
        Assert.True(manifest.HasPinnedItems);
        Assert.Equal("MY-PC", manifest.MachineName);
    }

    #endregion

    #region JSON Serialization

    [Fact]
    public void JsonSerialization_RoundTrip_PreservesValues()
    {
        var now = DateTime.UtcNow;
        var original = new BackupManifest
        {
            Version = 1,
            AppVersion = "2.0.0",
            CreatedAtUtc = now,
            ItemCount = 50,
            ImageCount = 10,
            ThumbnailCount = 10,
            HasPinnedItems = true,
            MachineName = "TEST-PC"
        };

        var json = JsonSerializer.Serialize(original, BackupManifestJsonContext.Default.BackupManifest);
        var deserialized = JsonSerializer.Deserialize(json, BackupManifestJsonContext.Default.BackupManifest);

        Assert.NotNull(deserialized);
        Assert.Equal(original.Version, deserialized.Version);
        Assert.Equal(original.AppVersion, deserialized.AppVersion);
        Assert.Equal(original.ItemCount, deserialized.ItemCount);
        Assert.Equal(original.ImageCount, deserialized.ImageCount);
        Assert.Equal(original.ThumbnailCount, deserialized.ThumbnailCount);
        Assert.Equal(original.HasPinnedItems, deserialized.HasPinnedItems);
        Assert.Equal(original.MachineName, deserialized.MachineName);
    }

    [Fact]
    public void JsonDeserialization_EmptyJson_UsesDefaults()
    {
        var json = "{}";
        var manifest = JsonSerializer.Deserialize(json, BackupManifestJsonContext.Default.BackupManifest);

        Assert.NotNull(manifest);
        Assert.Equal(1, manifest.Version);
        Assert.Equal(0, manifest.ItemCount);
    }

    #endregion
}
