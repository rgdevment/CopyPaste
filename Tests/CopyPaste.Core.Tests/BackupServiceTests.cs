using System;
using System.IO;
using System.IO.Compression;
using System.Text;
using System.Text.Json;
using Microsoft.Data.Sqlite;
using Xunit;

namespace CopyPaste.Core.Tests;

public sealed class BackupServiceTests : IDisposable
{
    private readonly string _basePath;

    public BackupServiceTests()
    {
        _basePath = Path.Combine(Path.GetTempPath(), "CopyPasteTests", Guid.NewGuid().ToString());
        StorageConfig.SetBasePath(_basePath);
        StorageConfig.Initialize();
    }

    #region CreateBackup Tests

    [Fact]
    public void CreateBackup_WritesValidZip()
    {
        SeedDatabase();

        using var output = new MemoryStream();
        BackupService.CreateBackup(output, "1.0.0");

        output.Position = 0;
        using var archive = new ZipArchive(output, ZipArchiveMode.Read);
        Assert.NotNull(archive.GetEntry("manifest.json"));
        Assert.NotNull(archive.GetEntry("clipboard.db"));
    }

    [Fact]
    public void CreateBackup_ManifestContainsCorrectItemCount()
    {
        SeedDatabaseWithItems(5);

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "2.0.0");

        Assert.Equal(5, manifest.ItemCount);
    }

    [Fact]
    public void CreateBackup_ManifestHasCorrectVersion()
    {
        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.5.0");

        Assert.Equal(BackupService.CurrentVersion, manifest.Version);
        Assert.Equal("1.5.0", manifest.AppVersion);
    }

    [Fact]
    public void CreateBackup_IncludesImages()
    {
        // Create a fake image file
        var imagePath = Path.Combine(StorageConfig.ImagesPath, "test-image.png");
        File.WriteAllBytes(imagePath, new byte[] { 0x89, 0x50, 0x4E, 0x47 });

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.Equal(1, manifest.ImageCount);

        output.Position = 0;
        using var archive = new ZipArchive(output, ZipArchiveMode.Read);
        Assert.NotNull(archive.GetEntry("images/test-image.png"));
    }

    [Fact]
    public void CreateBackup_IncludesThumbnails()
    {
        var thumbPath = Path.Combine(StorageConfig.ThumbnailsPath, "thumb_t.png");
        File.WriteAllBytes(thumbPath, new byte[] { 0x89, 0x50, 0x4E, 0x47 });

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.Equal(1, manifest.ThumbnailCount);
    }

    [Fact]
    public void CreateBackup_IncludesConfig()
    {
        var configPath = Path.Combine(StorageConfig.ConfigPath, "MyM.json");
        File.WriteAllText(configPath, """{"PreferredLanguage":"es-CL"}""");

        using var output = new MemoryStream();
        BackupService.CreateBackup(output, "1.0.0");

        output.Position = 0;
        using var archive = new ZipArchive(output, ZipArchiveMode.Read);
        Assert.NotNull(archive.GetEntry("config/MyM.json"));
    }

    [Fact]
    public void CreateBackup_ManifestHasMachineName()
    {
        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.Equal(Environment.MachineName, manifest.MachineName);
    }

    [Fact]
    public void CreateBackup_ManifestHasTimestamp()
    {
        var before = DateTime.UtcNow.AddSeconds(-1);

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        var after = DateTime.UtcNow.AddSeconds(1);
        Assert.InRange(manifest.CreatedAtUtc, before, after);
    }

    [Fact]
    public void CreateBackup_EmptyDatabase_Succeeds()
    {
        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.Equal(0, manifest.ItemCount);
        Assert.False(manifest.HasPinnedItems);
    }

    [Fact]
    public void CreateBackup_DetectsPinnedItems()
    {
        SeedDatabaseWithPinnedItem();

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.True(manifest.HasPinnedItems);
    }

    [Fact]
    public void CreateBackup_ThrowsOnNullStream()
    {
        Assert.Throws<ArgumentNullException>(() => BackupService.CreateBackup(null!, "1.0.0"));
    }

    #endregion

    #region ValidateBackup Tests

    [Fact]
    public void ValidateBackup_ValidZip_ReturnsManifest()
    {
        using var backupStream = CreateTestBackup();
        backupStream.Position = 0;

        var manifest = BackupService.ValidateBackup(backupStream);

        Assert.NotNull(manifest);
        Assert.Equal(BackupService.CurrentVersion, manifest.Version);
    }

    [Fact]
    public void ValidateBackup_InvalidZip_ReturnsNull()
    {
        using var invalidStream = new MemoryStream(Encoding.UTF8.GetBytes("not a zip file"));

        var manifest = BackupService.ValidateBackup(invalidStream);

        Assert.Null(manifest);
    }

    [Fact]
    public void ValidateBackup_ZipWithoutManifest_ReturnsNull()
    {
        using var zipStream = new MemoryStream();
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create, leaveOpen: true))
        {
            var entry = archive.CreateEntry("random.txt");
            using var writer = new StreamWriter(entry.Open());
            writer.Write("hello");
        }

        zipStream.Position = 0;
        var manifest = BackupService.ValidateBackup(zipStream);

        Assert.Null(manifest);
    }

    [Fact]
    public void ValidateBackup_ThrowsOnNullStream()
    {
        Assert.Throws<ArgumentNullException>(() => BackupService.ValidateBackup(null!));
    }

    #endregion

    #region GetBackupInfo Tests

    [Fact]
    public void GetBackupInfo_ReturnsManifestData()
    {
        using var backupStream = CreateTestBackup(itemCount: 10);
        backupStream.Position = 0;

        var info = BackupService.GetBackupInfo(backupStream);

        Assert.NotNull(info);
        Assert.Equal("1.0.0", info.AppVersion);
    }

    #endregion

    #region RestoreBackup Tests

    [Fact]
    public void RestoreBackup_RestoresDatabase()
    {
        // Seed and backup
        SeedDatabaseWithItems(3);
        using var backupStream = new MemoryStream();
        BackupService.CreateBackup(backupStream, "1.0.0");

        // Clear current data (release SQLite shared cache first)
        SqliteConnection.ClearAllPools();
        File.Delete(StorageConfig.DatabasePath);
        Assert.False(File.Exists(StorageConfig.DatabasePath));

        // Restore
        backupStream.Position = 0;
        var manifest = BackupService.RestoreBackup(backupStream);

        Assert.NotNull(manifest);
        Assert.True(File.Exists(StorageConfig.DatabasePath));
    }

    [Fact]
    public void RestoreBackup_RestoresImages()
    {
        // Create image and backup
        var imagePath = Path.Combine(StorageConfig.ImagesPath, "restored-image.png");
        File.WriteAllBytes(imagePath, new byte[] { 0x89, 0x50, 0x4E, 0x47 });

        using var backupStream = new MemoryStream();
        BackupService.CreateBackup(backupStream, "1.0.0");

        // Delete image
        File.Delete(imagePath);
        Assert.False(File.Exists(imagePath));

        // Restore
        backupStream.Position = 0;
        BackupService.RestoreBackup(backupStream);

        Assert.True(File.Exists(imagePath));
    }

    [Fact]
    public void RestoreBackup_RestoresConfig()
    {
        // Create config and backup
        var configFile = Path.Combine(StorageConfig.ConfigPath, "MyM.json");
        File.WriteAllText(configFile, """{"RetentionDays":60}""");

        using var backupStream = new MemoryStream();
        BackupService.CreateBackup(backupStream, "1.0.0");

        // Delete config
        File.Delete(configFile);
        Assert.False(File.Exists(configFile));

        // Restore
        backupStream.Position = 0;
        BackupService.RestoreBackup(backupStream);

        Assert.True(File.Exists(configFile));
        Assert.Contains("60", File.ReadAllText(configFile), StringComparison.Ordinal);
    }

    [Fact]
    public void RestoreBackup_CleansOrphanFilesInImageDirectory()
    {
        // Backup with one image
        var imagePath = Path.Combine(StorageConfig.ImagesPath, "good.png");
        File.WriteAllBytes(imagePath, new byte[] { 0x01 });

        using var backupStream = new MemoryStream();
        BackupService.CreateBackup(backupStream, "1.0.0");

        // Add an orphan image that shouldn't exist after restore
        var orphanPath = Path.Combine(StorageConfig.ImagesPath, "orphan.png");
        File.WriteAllBytes(orphanPath, new byte[] { 0x02 });

        // Restore
        backupStream.Position = 0;
        BackupService.RestoreBackup(backupStream);

        Assert.True(File.Exists(imagePath));
        Assert.False(File.Exists(orphanPath));
    }

    [Fact]
    public void RestoreBackup_InvalidStream_ReturnsNull()
    {
        using var invalidStream = new MemoryStream(Encoding.UTF8.GetBytes("not a zip"));

        var result = BackupService.RestoreBackup(invalidStream);

        Assert.Null(result);
    }

    [Fact]
    public void RestoreBackup_ThrowsOnNullStream()
    {
        Assert.Throws<ArgumentNullException>(() => BackupService.RestoreBackup(null!));
    }

    #endregion

    #region Roundtrip Tests

    [Fact]
    public void Roundtrip_BackupAndRestore_PreservesAllData()
    {
        // Setup: DB + images + thumbs + config
        SeedDatabaseWithItems(5);

        var img = Path.Combine(StorageConfig.ImagesPath, "roundtrip.png");
        File.WriteAllBytes(img, new byte[] { 0x89, 0x50 });

        var thumb = Path.Combine(StorageConfig.ThumbnailsPath, "roundtrip_t.png");
        File.WriteAllBytes(thumb, new byte[] { 0xAA, 0xBB });

        var config = Path.Combine(StorageConfig.ConfigPath, "MyM.json");
        File.WriteAllText(config, """{"PreferredLanguage":"ja-JP"}""");

        // Backup
        using var backupStream = new MemoryStream();
        var backupManifest = BackupService.CreateBackup(backupStream, "3.0.0");

        // Simulate data loss (release SQLite shared cache first)
        SqliteConnection.ClearAllPools();
        File.Delete(StorageConfig.DatabasePath);
        File.Delete(img);
        File.Delete(thumb);
        File.Delete(config);

        // Restore
        backupStream.Position = 0;
        var restoredManifest = BackupService.RestoreBackup(backupStream);

        // Verify
        Assert.NotNull(restoredManifest);
        Assert.Equal(backupManifest.ItemCount, restoredManifest.ItemCount);
        Assert.True(File.Exists(StorageConfig.DatabasePath));
        Assert.True(File.Exists(img));
        Assert.True(File.Exists(thumb));
        Assert.True(File.Exists(config));
        Assert.Contains("ja-JP", File.ReadAllText(config), StringComparison.Ordinal);
    }

    #endregion

    #region BackupManifest Serialization Tests

    [Fact]
    public void BackupManifest_Serialization_Roundtrip()
    {
        var manifest = new BackupManifest
        {
            Version = 1,
            AppVersion = "1.2.3",
            CreatedAtUtc = new DateTime(2026, 2, 12, 10, 0, 0, DateTimeKind.Utc),
            ItemCount = 42,
            ImageCount = 10,
            ThumbnailCount = 10,
            HasPinnedItems = true,
            MachineName = "TEST-PC"
        };

        var json = JsonSerializer.Serialize(manifest, BackupManifestJsonContext.Default.BackupManifest);
        var deserialized = JsonSerializer.Deserialize(json, BackupManifestJsonContext.Default.BackupManifest);

        Assert.NotNull(deserialized);
        Assert.Equal(manifest.Version, deserialized.Version);
        Assert.Equal(manifest.AppVersion, deserialized.AppVersion);
        Assert.Equal(manifest.ItemCount, deserialized.ItemCount);
        Assert.Equal(manifest.ImageCount, deserialized.ImageCount);
        Assert.Equal(manifest.ThumbnailCount, deserialized.ThumbnailCount);
        Assert.True(deserialized.HasPinnedItems);
        Assert.Equal("TEST-PC", deserialized.MachineName);
    }

    #endregion

    #region Helpers

    private static void SeedDatabase()
    {
        using var repo = new SqliteRepository(StorageConfig.DatabasePath);
        repo.Save(new ClipboardItem
        {
            Content = "test content",
            Type = ClipboardContentType.Text
        });
    }

    private static void SeedDatabaseWithItems(int count)
    {
        using var repo = new SqliteRepository(StorageConfig.DatabasePath);
        for (int i = 0; i < count; i++)
        {
            repo.Save(new ClipboardItem
            {
                Content = $"item {i}",
                Type = ClipboardContentType.Text
            });
        }
    }

    private static void SeedDatabaseWithPinnedItem()
    {
        using var repo = new SqliteRepository(StorageConfig.DatabasePath);
        repo.Save(new ClipboardItem
        {
            Content = "pinned item",
            Type = ClipboardContentType.Text,
            IsPinned = true
        });
    }

    private static MemoryStream CreateTestBackup(int itemCount = 0)
    {
        if (itemCount > 0)
            SeedDatabaseWithItems(itemCount);
        else
            SeedDatabase();

        var stream = new MemoryStream();
        BackupService.CreateBackup(stream, "1.0.0");
        return stream;
    }

    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_basePath))
            {
                Directory.Delete(_basePath, recursive: true);
            }
        }
        catch
        {
            // Best-effort cleanup for temp test data.
        }
    }

    #endregion
}
