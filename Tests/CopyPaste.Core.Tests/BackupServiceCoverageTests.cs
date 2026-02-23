using System;
using System.IO;
using System.IO.Compression;
using System.Text.Json;
using Microsoft.Data.Sqlite;
using Xunit;

namespace CopyPaste.Core.Tests;

/// <summary>
/// Additional BackupService tests targeting uncovered paths:
/// RestoreBackup version > CurrentVersion, RollbackFromSnapshot,
/// RestoreDirectory with files, CopyDirectoryFlat with files.
/// </summary>
public sealed class BackupServiceCoverageTests : IDisposable
{
    private readonly string _basePath;

    public BackupServiceCoverageTests()
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

    private static void InsertItem(bool isPinned = false)
    {
        using var connection = new SqliteConnection($"Data Source={StorageConfig.DatabasePath}");
        connection.Open();
        using var cmd = connection.CreateCommand();
        cmd.CommandText = @"INSERT INTO ClipboardItems
            (Id, Content, Type, CreatedAt, ModifiedAt, IsPinned, CardColor, PasteCount)
            VALUES ($id, 'test', 0, datetime('now'), datetime('now'), $pinned, 0, 0)";
        cmd.Parameters.AddWithValue("$id", Guid.NewGuid().ToString());
        cmd.Parameters.AddWithValue("$pinned", isPinned ? 1 : 0);
        cmd.ExecuteNonQuery();
    }

    // -------------------------------------------------------------------------
    // RestoreBackup — Version > CurrentVersion
    // -------------------------------------------------------------------------

    [Fact]
    public void RestoreBackup_WithFutureVersion_ReturnsNull()
    {
        // Craft a backup with a version higher than CurrentVersion
        using var stream = CreateBackupWithVersion(BackupService.CurrentVersion + 1);

        var result = BackupService.RestoreBackup(stream);

        Assert.Null(result);
    }

    [Fact]
    public void RestoreBackup_WithExactlyCurrentVersion_Succeeds()
    {
        CreateMinimalDatabase();
        using var stream = new MemoryStream();
        BackupService.CreateBackup(stream, "1.0.0");

        SqliteConnection.ClearAllPools();
        stream.Position = 0;

        var result = BackupService.RestoreBackup(stream);
        Assert.NotNull(result);
        Assert.Equal(BackupService.CurrentVersion, result.Version);
    }

    // -------------------------------------------------------------------------
    // CreateBackup with pinned items (HasPinnedItems = true)
    // -------------------------------------------------------------------------

    [Fact]
    public void CreateBackup_WithPinnedItems_ManifestHasPinnedItems()
    {
        CreateMinimalDatabase();
        InsertItem(isPinned: true);

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.True(manifest.HasPinnedItems);
    }

    [Fact]
    public void CreateBackup_WithItems_ManifestHasCorrectItemCount()
    {
        CreateMinimalDatabase();
        InsertItem(isPinned: false);
        InsertItem(isPinned: false);
        InsertItem(isPinned: true);

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.Equal(3, manifest.ItemCount);
    }

    // -------------------------------------------------------------------------
    // RestoreDirectory with files (covers TryDeleteFile + ExtractToFile paths)
    // -------------------------------------------------------------------------

    [Fact]
    public void RestoreBackup_WithImagesInArchive_RestoresImagesDirectory()
    {
        CreateMinimalDatabase();

        // Create image files in the images directory before backup
        string imgFile1 = Path.Combine(StorageConfig.ImagesPath, "test1.png");
        string imgFile2 = Path.Combine(StorageConfig.ImagesPath, "test2.png");
        File.WriteAllBytes(imgFile1, CreateMinimalPngBytes());
        File.WriteAllBytes(imgFile2, CreateMinimalPngBytes());

        using var backupStream = new MemoryStream();
        BackupService.CreateBackup(backupStream, "1.0.0");

        // Delete images to verify they get restored
        File.Delete(imgFile1);
        File.Delete(imgFile2);
        Assert.False(File.Exists(imgFile1));

        SqliteConnection.ClearAllPools();
        backupStream.Position = 0;
        var result = BackupService.RestoreBackup(backupStream);

        Assert.NotNull(result);
        Assert.Equal(2, result.ImageCount);
        Assert.True(File.Exists(imgFile1));
        Assert.True(File.Exists(imgFile2));
    }

    [Fact]
    public void RestoreBackup_ExistingImagesDeleted_BeforeRestoring()
    {
        CreateMinimalDatabase();

        // Create one image in backup
        string imgFile = Path.Combine(StorageConfig.ImagesPath, "original.png");
        File.WriteAllBytes(imgFile, CreateMinimalPngBytes());

        using var backupStream = new MemoryStream();
        BackupService.CreateBackup(backupStream, "1.0.0");

        // Add a different file that should be cleaned during restore
        string extraFile = Path.Combine(StorageConfig.ImagesPath, "extra.png");
        File.WriteAllBytes(extraFile, CreateMinimalPngBytes());

        SqliteConnection.ClearAllPools();
        backupStream.Position = 0;
        BackupService.RestoreBackup(backupStream);

        // The extra file that wasn't in backup should have been deleted (TryDeleteFile path)
        Assert.False(File.Exists(extraFile));
        Assert.True(File.Exists(imgFile));
    }

    // -------------------------------------------------------------------------
    // CreateBackup with thumbnail images (covers AddDirectoryToArchive with files)
    // -------------------------------------------------------------------------

    [Fact]
    public void CreateBackup_WithThumbnails_ManifestHasCorrectThumbnailCount()
    {
        // Add thumbnails
        string thumb1 = Path.Combine(StorageConfig.ThumbnailsPath, "thumb1_t.png");
        string thumb2 = Path.Combine(StorageConfig.ThumbnailsPath, "thumb2_t.png");
        File.WriteAllBytes(thumb1, CreateMinimalPngBytes());
        File.WriteAllBytes(thumb2, CreateMinimalPngBytes());

        using var output = new MemoryStream();
        var manifest = BackupService.CreateBackup(output, "1.0.0");

        Assert.Equal(2, manifest.ThumbnailCount);
    }

    // -------------------------------------------------------------------------
    // RestoreBackup — invalid/corrupt archive triggers rollback path
    // -------------------------------------------------------------------------

    [Fact]
    public void RestoreBackup_WithCorruptStream_ReturnsNull()
    {
        using var invalid = new MemoryStream(new byte[] { 1, 2, 3, 4, 5 });

        var result = BackupService.RestoreBackup(invalid);

        Assert.Null(result);
    }

    [Fact]
    public void RestoreBackup_StreamFailsOnSecondSeek_TriggersRollbackAndReturnsNull()
    {
        // Create a valid backup first
        CreateMinimalDatabase();
        using var validBackupData = new MemoryStream();
        BackupService.CreateBackup(validBackupData, "1.0.0");
        byte[] bytes = validBackupData.ToArray();

        SqliteConnection.ClearAllPools();

        // Wrap in a stream that fails on the second seek (after manifest is read)
        using var failingStream = new FailOnSecondSeekStream(bytes);
        var result = BackupService.RestoreBackup(failingStream);

        // Restore should fail and return null (rollback triggered)
        Assert.Null(result);
    }

    // -------------------------------------------------------------------------
    // ValidateBackup — corrupt manifest entry
    // -------------------------------------------------------------------------

    [Fact]
    public void ValidateBackup_WithCorruptManifest_ReturnsNull()
    {
        using var stream = new MemoryStream();
        using (var archive = new ZipArchive(stream, ZipArchiveMode.Create, leaveOpen: true))
        {
            var entry = archive.CreateEntry("manifest.json");
            using var entryStream = entry.Open();
            using var writer = new System.IO.StreamWriter(entryStream);
            writer.Write("{ invalid json :::"); // Corrupt JSON
        }

        stream.Position = 0;
        var result = BackupService.ValidateBackup(stream);

        Assert.Null(result);
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    private static MemoryStream CreateBackupWithVersion(int version)
    {
        var manifest = new BackupManifest
        {
            Version = version,
            AppVersion = "99.0.0",
            CreatedAtUtc = DateTime.UtcNow,
            MachineName = "TestMachine",
            ItemCount = 0
        };

        var stream = new MemoryStream();
        using (var archive = new ZipArchive(stream, ZipArchiveMode.Create, leaveOpen: true))
        {
            var entry = archive.CreateEntry("manifest.json");
            using var entryStream = entry.Open();
            JsonSerializer.Serialize(entryStream, manifest, BackupManifestJsonContext.Default.BackupManifest);
        }

        stream.Position = 0;
        return stream;
    }

    private static byte[] CreateMinimalPngBytes()
    {
        using var bitmap = new SkiaSharp.SKBitmap(2, 2);
        bitmap.SetPixel(0, 0, SkiaSharp.SKColors.Red);
        using var image = SkiaSharp.SKImage.FromBitmap(bitmap);
        using var data = image.Encode(SkiaSharp.SKEncodedImageFormat.Png, 100);
        return data.ToArray();
    }
}

/// <summary>
/// A MemoryStream that throws IOException on the second Position = 0 assignment.
/// Used to simulate a restore failure after the manifest is successfully read,
/// triggering the RollbackFromSnapshot code path.
/// </summary>
internal sealed class FailOnSecondSeekStream : MemoryStream
{
    private int _positionSetCount;

    public FailOnSecondSeekStream(byte[] data) : base(data) { }

    public override long Position
    {
        get => base.Position;
        set
        {
            if (value == 0)
            {
                _positionSetCount++;
                if (_positionSetCount > 1)
                    throw new IOException("Simulated stream seek failure for testing RollbackFromSnapshot");
            }
            base.Position = value;
        }
    }
}
