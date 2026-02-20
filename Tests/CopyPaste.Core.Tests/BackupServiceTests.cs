using System;
using System.IO;
using System.IO.Compression;
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

    public void Dispose()
    {
        SqliteConnection.ClearAllPools();
        try
        {
            if (Directory.Exists(_basePath))
                Directory.Delete(_basePath, recursive: true);
        }
        catch { }
    }

    private static void CreateMinimalDatabase()
    {
        using var connection = new SqliteConnection($"Data Source={StorageConfig.DatabasePath}");
        connection.Open();
        using var cmd = connection.CreateCommand();
        cmd.CommandText = @"CREATE TABLE IF NOT EXISTS ClipboardItems (
            Id TEXT PRIMARY KEY, Content TEXT NOT NULL, Type INTEGER NOT NULL,
            CreatedAt TEXT NOT NULL, ModifiedAt TEXT NOT NULL, AppSource TEXT,
            IsPinned INTEGER NOT NULL DEFAULT 0, Metadata TEXT, Label TEXT,
            CardColor INTEGER NOT NULL DEFAULT 0, PasteCount INTEGER NOT NULL DEFAULT 0,
            ContentHash TEXT)";
        cmd.ExecuteNonQuery();
    }

    #region CurrentVersion

    [Fact]
    public void CurrentVersion_Is1()
    {
        Assert.Equal(1, BackupService.CurrentVersion);
    }

    #endregion

    #region CreateBackup

    [Fact]
    public void CreateBackup_WithNullStream_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() => BackupService.CreateBackup(null!, "1.0.0"));
    }

    [Fact]
    public void CreateBackup_EmptyDatabase_ReturnsManifest()
    {
        CreateMinimalDatabase();

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.NotNull(manifest);
        Assert.Equal(0, manifest.ItemCount);
    }

    [Fact]
    public void CreateBackup_WritesValidZipToStream()
    {
        CreateMinimalDatabase();

        using var output = new MemoryStream();
        BackupService.CreateBackup(output, "1.0.0");

        output.Position = 0;
        using var archive = new ZipArchive(output, ZipArchiveMode.Read);
        Assert.NotNull(archive.GetEntry("manifest.json"));
    }

    [Fact]
    public void CreateBackup_ManifestHasCorrectVersion()
    {
        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.Equal(BackupService.CurrentVersion, manifest.Version);
    }

    [Fact]
    public void CreateBackup_ManifestHasAppVersion()
    {
        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "2.5.0");

        Assert.Equal("2.5.0", manifest.AppVersion);
    }

    [Fact]
    public void CreateBackup_ManifestHasMachineName()
    {
        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.Equal(Environment.MachineName, manifest.MachineName);
    }

    [Fact]
    public void CreateBackup_ManifestHasCreatedAtUtc()
    {
        var before = DateTime.UtcNow.AddSeconds(-1);

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        var after = DateTime.UtcNow.AddSeconds(1);
        Assert.InRange(manifest.CreatedAtUtc, before, after);
    }

    #endregion

    #region ValidateBackup and GetBackupInfo

    [Fact]
    public void ValidateBackup_WithNullStream_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() => BackupService.ValidateBackup(null!));
    }

    [Fact]
    public void ValidateBackup_WithValidBackup_ReturnsManifest()
    {
        using var stream = new MemoryStream();
        BackupService.CreateBackup(stream, "1.0.0");
        stream.Position = 0;

        var manifest = BackupService.ValidateBackup(stream);

        Assert.NotNull(manifest);
        Assert.Equal(BackupService.CurrentVersion, manifest.Version);
    }

    [Fact]
    public void ValidateBackup_WithInvalidStream_ReturnsNull()
    {
        using var stream = new MemoryStream();

        var manifest = BackupService.ValidateBackup(stream);

        Assert.Null(manifest);
    }

    [Fact]
    public void GetBackupInfo_WithNullStream_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() => BackupService.GetBackupInfo(null!));
    }

    [Fact]
    public void GetBackupInfo_WithValidBackup_ReturnsManifest()
    {
        using var stream = new MemoryStream();
        BackupService.CreateBackup(stream, "1.0.0");
        stream.Position = 0;

        var info = BackupService.GetBackupInfo(stream);

        Assert.NotNull(info);
        Assert.Equal(BackupService.CurrentVersion, info.Version);
    }

    [Fact]
    public void GetBackupInfo_SameAsValidateBackup()
    {
        using var stream1 = new MemoryStream();
        BackupService.CreateBackup(stream1, "3.0.0");

        using var stream2 = new MemoryStream(stream1.ToArray());

        stream1.Position = 0;
        var validateResult = BackupService.ValidateBackup(stream1);

        stream2.Position = 0;
        var infoResult = BackupService.GetBackupInfo(stream2);

        Assert.NotNull(validateResult);
        Assert.NotNull(infoResult);
        Assert.Equal(validateResult.Version, infoResult.Version);
        Assert.Equal(validateResult.AppVersion, infoResult.AppVersion);
        Assert.Equal(validateResult.MachineName, infoResult.MachineName);
        Assert.Equal(validateResult.ItemCount, infoResult.ItemCount);
    }

    #endregion

    #region RestoreBackup

    [Fact]
    public void RestoreBackup_WithNullStream_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() => BackupService.RestoreBackup(null!));
    }

    [Fact]
    public void RestoreBackup_WithValidBackup_ReturnsManifest()
    {
        CreateMinimalDatabase();

        using var stream = new MemoryStream();
        BackupService.CreateBackup(stream, "1.0.0");

        SqliteConnection.ClearAllPools();
        stream.Position = 0;
        var manifest = BackupService.RestoreBackup(stream);

        Assert.NotNull(manifest);
    }

    [Fact]
    public void RestoreBackup_WithInvalidStream_ReturnsNull()
    {
        using var stream = new MemoryStream();

        var manifest = BackupService.RestoreBackup(stream);

        Assert.Null(manifest);
    }

    #endregion

    #region Round-trip

    [Fact]
    public void CreateAndValidate_RoundTrip_PreservesManifestData()
    {
        CreateMinimalDatabase();

        using var stream = new MemoryStream();
        var created = BackupService.CreateBackup(stream, "5.0.0");
        stream.Position = 0;

        var validated = BackupService.ValidateBackup(stream);

        Assert.NotNull(validated);
        Assert.Equal(created.Version, validated.Version);
        Assert.Equal(created.AppVersion, validated.AppVersion);
        Assert.Equal(created.MachineName, validated.MachineName);
        Assert.Equal(created.ItemCount, validated.ItemCount);
        Assert.Equal(created.ImageCount, validated.ImageCount);
        Assert.Equal(created.ThumbnailCount, validated.ThumbnailCount);
        Assert.Equal(created.HasPinnedItems, validated.HasPinnedItems);
    }

    [Fact]
    public void CreateAndRestore_RoundTrip_Works()
    {
        CreateMinimalDatabase();

        using var stream = new MemoryStream();
        var created = BackupService.CreateBackup(stream, "1.0.0");

        SqliteConnection.ClearAllPools();
        if (File.Exists(StorageConfig.DatabasePath))
            File.Delete(StorageConfig.DatabasePath);

        stream.Position = 0;
        var restored = BackupService.RestoreBackup(stream);

        Assert.NotNull(restored);
        Assert.Equal(created.Version, restored.Version);
        Assert.Equal(created.AppVersion, restored.AppVersion);
        Assert.True(File.Exists(StorageConfig.DatabasePath));
    }

    #endregion
}
