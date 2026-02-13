using System.IO.Compression;
using System.Text.Json;
using Microsoft.Data.Sqlite;

namespace CopyPaste.Core;

public static class BackupService
{
    private const string _manifestFileName = "manifest.json";
    private const string _databaseFileName = "clipboard.db";
    private const string _imagesFolderName = "images";
    private const string _thumbsFolderName = "thumbs";
    private const string _configFolderName = "config";

    public const int CurrentVersion = 1;

    public static BackupManifest CreateBackup(Stream output, string appVersion)
    {
        ArgumentNullException.ThrowIfNull(output);

        CheckpointDatabase();

        var manifest = new BackupManifest
        {
            Version = CurrentVersion,
            AppVersion = appVersion,
            CreatedAtUtc = DateTime.UtcNow,
            MachineName = Environment.MachineName
        };

        using var archive = new ZipArchive(output, ZipArchiveMode.Create, leaveOpen: true);

        var dbPath = StorageConfig.DatabasePath;
        if (File.Exists(dbPath))
        {
            AddFileToArchive(archive, dbPath, _databaseFileName);
            manifest.ItemCount = CountDatabaseItems(dbPath);
            manifest.HasPinnedItems = HasPinnedItems(dbPath);
        }

        manifest.ImageCount = AddDirectoryToArchive(archive, StorageConfig.ImagesPath, _imagesFolderName);
        manifest.ThumbnailCount = AddDirectoryToArchive(archive, StorageConfig.ThumbnailsPath, _thumbsFolderName);
        AddDirectoryToArchive(archive, StorageConfig.ConfigPath, _configFolderName);

        var manifestEntry = archive.CreateEntry(_manifestFileName, CompressionLevel.Optimal);
        using (var manifestStream = manifestEntry.Open())
        {
            JsonSerializer.Serialize(manifestStream, manifest, BackupManifestJsonContext.Default.BackupManifest);
        }

        AppLogger.Info($"Backup created: {manifest.ItemCount} items, {manifest.ImageCount} images, {manifest.ThumbnailCount} thumbs");
        return manifest;
    }

    public static BackupManifest? RestoreBackup(Stream input)
    {
        ArgumentNullException.ThrowIfNull(input);

        var manifest = ReadManifestFromStream(input);
        if (manifest == null)
        {
            AppLogger.Error("Restore failed: invalid backup file (missing or corrupt manifest)");
            return null;
        }

        if (manifest.Version > CurrentVersion)
        {
            AppLogger.Error($"Restore failed: backup version {manifest.Version} is newer than supported version {CurrentVersion}");
            return null;
        }

        var snapshotPath = CreatePreRestoreSnapshot();

        try
        {
            SqliteConnection.ClearAllPools();

            input.Position = 0;
            using var archive = new ZipArchive(input, ZipArchiveMode.Read, leaveOpen: true);

            RestoreFile(archive, _databaseFileName, StorageConfig.DatabasePath);
            RestoreDirectory(archive, _imagesFolderName, StorageConfig.ImagesPath);
            RestoreDirectory(archive, _thumbsFolderName, StorageConfig.ThumbnailsPath);
            RestoreDirectory(archive, _configFolderName, StorageConfig.ConfigPath);

            AppLogger.Info($"Restore completed: {manifest.ItemCount} items from backup dated {manifest.CreatedAtUtc:O}");
            CleanupSnapshot(snapshotPath);

            return manifest;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Restore failed, attempting rollback from snapshot");
            RollbackFromSnapshot(snapshotPath);
            return null;
        }
    }

    public static BackupManifest? ValidateBackup(Stream input)
    {
        ArgumentNullException.ThrowIfNull(input);
        return ReadManifestFromStream(input);
    }

    public static BackupManifest? GetBackupInfo(Stream input)
    {
        ArgumentNullException.ThrowIfNull(input);
        return ReadManifestFromStream(input);
    }

    private static BackupManifest? ReadManifestFromStream(Stream input)
    {
        try
        {
            input.Position = 0;
            using var archive = new ZipArchive(input, ZipArchiveMode.Read, leaveOpen: true);
            var manifestEntry = archive.GetEntry(_manifestFileName);
            if (manifestEntry == null) return null;

            using var stream = manifestEntry.Open();
            return JsonSerializer.Deserialize(stream, BackupManifestJsonContext.Default.BackupManifest);
        }
        catch (Exception ex)
        {
            AppLogger.Error($"Failed to read backup manifest: {ex.Message}");
            return null;
        }
    }

    private static int AddDirectoryToArchive(ZipArchive archive, string sourcePath, string entryPrefix)
    {
        if (!Directory.Exists(sourcePath)) return 0;

        var files = Directory.GetFiles(sourcePath);
        foreach (var file in files)
        {
            var entryName = $"{entryPrefix}/{Path.GetFileName(file)}";
            AddFileToArchive(archive, file, entryName);
        }

        return files.Length;
    }

    private static void AddFileToArchive(ZipArchive archive, string filePath, string entryName)
    {
        var entry = archive.CreateEntry(entryName, CompressionLevel.Optimal);
        using var entryStream = entry.Open();
        using var fileStream = new FileStream(filePath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
        fileStream.CopyTo(entryStream);
    }

    private static void RestoreFile(ZipArchive archive, string entryName, string destinationPath)
    {
        var entry = archive.GetEntry(entryName);
        if (entry == null) return;

        var directory = Path.GetDirectoryName(destinationPath);
        if (!string.IsNullOrEmpty(directory))
            Directory.CreateDirectory(directory);

        if (entryName == _databaseFileName)
        {
            TryDeleteFile(destinationPath + "-wal");
            TryDeleteFile(destinationPath + "-shm");
        }

        using var source = entry.Open();
        using var destination = new FileStream(destinationPath, FileMode.Create, FileAccess.Write, FileShare.ReadWrite);
        source.CopyTo(destination);
    }

    private static void RestoreDirectory(ZipArchive archive, string entryPrefix, string destinationPath)
    {
        if (Directory.Exists(destinationPath))
        {
            foreach (var file in Directory.GetFiles(destinationPath))
            {
                TryDeleteFile(file);
            }
        }
        else
        {
            Directory.CreateDirectory(destinationPath);
        }

        var prefix = entryPrefix + "/";
        foreach (var entry in archive.Entries)
        {
            if (!entry.FullName.StartsWith(prefix, StringComparison.OrdinalIgnoreCase))
                continue;

            var fileName = Path.GetFileName(entry.FullName);
            if (string.IsNullOrEmpty(fileName) || fileName.Contains("..", StringComparison.Ordinal))
                continue;

            var destinationFile = Path.Combine(destinationPath, fileName);
            entry.ExtractToFile(destinationFile, overwrite: true);
        }
    }

    private static void CheckpointDatabase()
    {
        var dbPath = StorageConfig.DatabasePath;
        if (!File.Exists(dbPath)) return;

        try
        {
            using var connection = new SqliteConnection($"Data Source={dbPath};Cache=Shared");
            connection.Open();
            using var cmd = connection.CreateCommand();
            cmd.CommandText = "PRAGMA wal_checkpoint(TRUNCATE);";
            cmd.ExecuteNonQuery();
        }
        catch (Exception ex)
        {
            AppLogger.Warn($"WAL checkpoint before backup failed (non-critical): {ex.Message}");
        }
    }

    private static int CountDatabaseItems(string dbPath)
    {
        try
        {
            using var connection = new SqliteConnection($"Data Source={dbPath};Mode=ReadOnly");
            connection.Open();
            using var cmd = connection.CreateCommand();
            cmd.CommandText = "SELECT COUNT(*) FROM ClipboardItems";
            return Convert.ToInt32(cmd.ExecuteScalar(), System.Globalization.CultureInfo.InvariantCulture);
        }
        catch
        {
            return 0;
        }
    }

    private static bool HasPinnedItems(string dbPath)
    {
        try
        {
            using var connection = new SqliteConnection($"Data Source={dbPath};Mode=ReadOnly");
            connection.Open();
            using var cmd = connection.CreateCommand();
            cmd.CommandText = "SELECT COUNT(*) FROM ClipboardItems WHERE IsPinned = 1";
            return Convert.ToInt32(cmd.ExecuteScalar(), System.Globalization.CultureInfo.InvariantCulture) > 0;
        }
        catch
        {
            return false;
        }
    }

    private static string CreatePreRestoreSnapshot()
    {
        var snapshotPath = Path.Combine(
            Path.GetDirectoryName(StorageConfig.DatabasePath) ?? string.Empty,
            $".pre-restore-{DateTime.UtcNow:yyyyMMddHHmmss}");

        try
        {
            Directory.CreateDirectory(snapshotPath);

            if (File.Exists(StorageConfig.DatabasePath))
                File.Copy(StorageConfig.DatabasePath, Path.Combine(snapshotPath, _databaseFileName), overwrite: true);

            CopyDirectoryFlat(StorageConfig.ImagesPath, Path.Combine(snapshotPath, _imagesFolderName));
            CopyDirectoryFlat(StorageConfig.ThumbnailsPath, Path.Combine(snapshotPath, _thumbsFolderName));
            CopyDirectoryFlat(StorageConfig.ConfigPath, Path.Combine(snapshotPath, _configFolderName));

            AppLogger.Info($"Pre-restore snapshot created at: {snapshotPath}");
        }
        catch (Exception ex)
        {
            AppLogger.Warn($"Failed to create pre-restore snapshot (restore will proceed): {ex.Message}");
        }

        return snapshotPath;
    }

    private static void RollbackFromSnapshot(string snapshotPath)
    {
        if (!Directory.Exists(snapshotPath)) return;

        try
        {
            var snapshotDb = Path.Combine(snapshotPath, _databaseFileName);
            if (File.Exists(snapshotDb))
                File.Copy(snapshotDb, StorageConfig.DatabasePath, overwrite: true);

            RestoreDirectoryFromSnapshot(Path.Combine(snapshotPath, _imagesFolderName), StorageConfig.ImagesPath);
            RestoreDirectoryFromSnapshot(Path.Combine(snapshotPath, _thumbsFolderName), StorageConfig.ThumbnailsPath);
            RestoreDirectoryFromSnapshot(Path.Combine(snapshotPath, _configFolderName), StorageConfig.ConfigPath);

            AppLogger.Info("Rollback from pre-restore snapshot completed");
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Rollback from snapshot failed - manual recovery may be needed");
        }
    }

    private static void RestoreDirectoryFromSnapshot(string snapshotDir, string targetDir)
    {
        if (!Directory.Exists(snapshotDir)) return;

        Directory.CreateDirectory(targetDir);
        foreach (var file in Directory.GetFiles(snapshotDir))
        {
            File.Copy(file, Path.Combine(targetDir, Path.GetFileName(file)), overwrite: true);
        }
    }

    private static void CleanupSnapshot(string snapshotPath)
    {
        try
        {
            if (Directory.Exists(snapshotPath))
                Directory.Delete(snapshotPath, recursive: true);
        }
        catch (Exception ex)
        {
            AppLogger.Warn($"Failed to clean up snapshot at {snapshotPath}: {ex.Message}");
        }
    }

    private static void CopyDirectoryFlat(string source, string destination)
    {
        if (!Directory.Exists(source)) return;

        Directory.CreateDirectory(destination);
        foreach (var file in Directory.GetFiles(source))
        {
            File.Copy(file, Path.Combine(destination, Path.GetFileName(file)), overwrite: true);
        }
    }

    private static void TryDeleteFile(string path)
    {
        try
        {
            if (File.Exists(path)) File.Delete(path);
        }
        catch (IOException) { }
        catch (UnauthorizedAccessException) { }
    }
}
