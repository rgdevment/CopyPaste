using Microsoft.Data.Sqlite;
using System.Diagnostics;

namespace CopyPaste.Core;

/// <summary>
/// Native AOT-compatible SQLite repository with FTS5 full-text search.
/// Minimal RAM usage - data stays on disk, only accessed when needed.
/// </summary>
public sealed class SqliteRepository : IClipboardRepository, IDisposable
{
    private readonly string _dbPath;
    private readonly string _connectionString;
    private bool _disposed;

    public SqliteRepository(string dbPath)
    {
        _dbPath = dbPath;
        _connectionString = $"Data Source={dbPath};Cache=Shared";
        InitializeDatabaseSafe();
    }

    private void InitializeDatabaseSafe()
    {
        try
        {
            InitializeDatabase();
        }
        catch (SqliteException ex) when (ex.SqliteErrorCode == 26) // SQLITE_NOTADB
        {
            Debug.WriteLine($"Database file is not valid SQLite: {ex.Message}");
            HandleCorruptDatabase();
            InitializeDatabase();
        }
        catch (SqliteException ex)
        {
            Debug.WriteLine($"SQLite error during initialization: {ex.Message}");
            HandleCorruptDatabase();
            InitializeDatabase();
        }
    }

    private void HandleCorruptDatabase()
    {
        if (!File.Exists(_dbPath)) return;

        try
        {
            // Backup the corrupt/incompatible file
            var backupPath = $"{_dbPath}.backup.{DateTime.Now:yyyyMMddHHmmss}";
            File.Move(_dbPath, backupPath);
            Debug.WriteLine($"Moved incompatible database to: {backupPath}");

            // Also remove WAL/SHM files if they exist
            var walPath = _dbPath + "-wal";
            var shmPath = _dbPath + "-shm";
            if (File.Exists(walPath)) File.Delete(walPath);
            if (File.Exists(shmPath)) File.Delete(shmPath);
        }
        catch (IOException ex)
        {
            Debug.WriteLine($"Failed to backup corrupt database: {ex.Message}");
            // Try to delete instead
            try
            {
                File.Delete(_dbPath);
            }
            catch { /* Last resort failed */ }
        }
    }

    private void InitializeDatabase()
    {
        using var connection = CreateConnection();

        // Main table
        ExecuteNonQuery(connection, """
            CREATE TABLE IF NOT EXISTS ClipboardItems (
                Id TEXT PRIMARY KEY,
                Content TEXT NOT NULL,
                Type INTEGER NOT NULL,
                CreatedAt TEXT NOT NULL,
                ModifiedAt TEXT NOT NULL,
                AppSource TEXT,
                IsPinned INTEGER NOT NULL DEFAULT 0,
                Metadata TEXT
            )
            """);

        // Indexes for common queries
        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_CreatedAt ON ClipboardItems(CreatedAt DESC)");
        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_ModifiedAt ON ClipboardItems(ModifiedAt DESC)");
        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_Type ON ClipboardItems(Type)");
        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_IsPinned ON ClipboardItems(IsPinned)");

        // FTS5 virtual table for full-text search
        ExecuteNonQuery(connection, """
            CREATE VIRTUAL TABLE IF NOT EXISTS ClipboardItems_fts USING fts5(
                Content,
                AppSource,
                content='ClipboardItems',
                content_rowid='rowid'
            )
            """);

        // Triggers to keep FTS in sync
        ExecuteNonQuery(connection, """
            CREATE TRIGGER IF NOT EXISTS ClipboardItems_ai AFTER INSERT ON ClipboardItems BEGIN
                INSERT INTO ClipboardItems_fts(rowid, Content, AppSource) 
                VALUES (NEW.rowid, NEW.Content, NEW.AppSource);
            END
            """);

        ExecuteNonQuery(connection, """
            CREATE TRIGGER IF NOT EXISTS ClipboardItems_ad AFTER DELETE ON ClipboardItems BEGIN
                INSERT INTO ClipboardItems_fts(ClipboardItems_fts, rowid, Content, AppSource) 
                VALUES ('delete', OLD.rowid, OLD.Content, OLD.AppSource);
            END
            """);

        ExecuteNonQuery(connection, """
            CREATE TRIGGER IF NOT EXISTS ClipboardItems_au AFTER UPDATE ON ClipboardItems BEGIN
                INSERT INTO ClipboardItems_fts(ClipboardItems_fts, rowid, Content, AppSource) 
                VALUES ('delete', OLD.rowid, OLD.Content, OLD.AppSource);
                INSERT INTO ClipboardItems_fts(rowid, Content, AppSource) 
                VALUES (NEW.rowid, NEW.Content, NEW.AppSource);
            END
            """);
    }

    private SqliteConnection CreateConnection()
    {
        var connection = new SqliteConnection(_connectionString);
        connection.Open();

        // Performance optimizations
        using var cmd = connection.CreateCommand();
        cmd.CommandText = "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA cache_size=10000;";
        cmd.ExecuteNonQuery();

        return connection;
    }

    private static void ExecuteNonQuery(SqliteConnection connection, string sql)
    {
        using var cmd = connection.CreateCommand();
#pragma warning disable CA2100
        cmd.CommandText = sql;
#pragma warning restore CA2100
        cmd.ExecuteNonQuery();
    }

    public void Save(ClipboardItem item)
    {
        ArgumentNullException.ThrowIfNull(item);

        if (item.Id == Guid.Empty)
            item.Id = Guid.NewGuid();

        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        cmd.CommandText = """
            INSERT INTO ClipboardItems (Id, Content, Type, CreatedAt, ModifiedAt, AppSource, IsPinned, Metadata)
            VALUES (@Id, @Content, @Type, @CreatedAt, @ModifiedAt, @AppSource, @IsPinned, @Metadata)
            """;

        AddParameters(cmd, item);
        cmd.ExecuteNonQuery();
    }

    public void Update(ClipboardItem item)
    {
        ArgumentNullException.ThrowIfNull(item);

        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        cmd.CommandText = """
            UPDATE ClipboardItems 
            SET Content = @Content, Type = @Type, CreatedAt = @CreatedAt, ModifiedAt = @ModifiedAt, 
                AppSource = @AppSource, IsPinned = @IsPinned, Metadata = @Metadata
            WHERE Id = @Id
            """;

        AddParameters(cmd, item);
        cmd.ExecuteNonQuery();
    }

    public ClipboardItem? GetById(Guid id)
    {
        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        cmd.CommandText = "SELECT * FROM ClipboardItems WHERE Id = @Id";
        cmd.Parameters.AddWithValue("@Id", id.ToString());

        using var reader = cmd.ExecuteReader();
        return reader.Read() ? MapToItem(reader) : null;
    }

    public ClipboardItem? GetLatest()
    {
        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        cmd.CommandText = """
            SELECT * FROM ClipboardItems 
            WHERE Type != @UnknownType 
            ORDER BY CreatedAt DESC 
            LIMIT 1
            """;
        cmd.Parameters.AddWithValue("@UnknownType", (int)ClipboardContentType.Unknown);

        using var reader = cmd.ExecuteReader();
        return reader.Read() ? MapToItem(reader) : null;
    }

    public IEnumerable<ClipboardItem> GetAll()
    {
        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        cmd.CommandText = """
            SELECT * FROM ClipboardItems 
            WHERE Type != @UnknownType 
            ORDER BY ModifiedAt DESC
            """;
        cmd.Parameters.AddWithValue("@UnknownType", (int)ClipboardContentType.Unknown);

        using var reader = cmd.ExecuteReader();
        var items = new List<ClipboardItem>();
        while (reader.Read())
        {
            items.Add(MapToItem(reader));
        }
        return items;
    }

    public void Delete(Guid id)
    {
        var item = GetById(id);
        if (item == null) return;

        // Clean up physical files
        CleanupItemFiles(item);

        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();
        cmd.CommandText = "DELETE FROM ClipboardItems WHERE Id = @Id";
        cmd.Parameters.AddWithValue("@Id", id.ToString());
        cmd.ExecuteNonQuery();
    }

    public int ClearOldItems(int days, bool excludePinned = true)
    {
        var limitDate = DateTime.UtcNow.AddDays(-days);

        using var connection = CreateConnection();

        // First get items to delete (for file cleanup)
        var itemsToDelete = new List<ClipboardItem>();
        using (var selectCmd = connection.CreateCommand())
        {
            selectCmd.CommandText = "SELECT * FROM ClipboardItems WHERE CreatedAt < @LimitDate" + (excludePinned ? " AND IsPinned = 0" : "");
            selectCmd.Parameters.AddWithValue("@LimitDate", limitDate.ToString("O"));

            using var reader = selectCmd.ExecuteReader();
            while (reader.Read())
            {
                itemsToDelete.Add(MapToItem(reader));
            }
        }

        // Cleanup files
        foreach (var item in itemsToDelete)
        {
            CleanupItemFiles(item);
        }

        // Delete from database
        using var deleteCmd = connection.CreateCommand();
        deleteCmd.CommandText = "DELETE FROM ClipboardItems WHERE CreatedAt < @LimitDate" + (excludePinned ? " AND IsPinned = 0" : "");
        deleteCmd.Parameters.AddWithValue("@LimitDate", limitDate.ToString("O"));

        var deletedCount = deleteCmd.ExecuteNonQuery();

        if (deletedCount > 50)
        {
            // Optimize database after large deletions
            ExecuteNonQuery(connection, "VACUUM");
            GC.Collect(0, GCCollectionMode.Optimized, false);
        }

        return deletedCount;
    }

    public IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0)
    {
        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        if (string.IsNullOrWhiteSpace(query))
        {
            cmd.CommandText = """
                SELECT * FROM ClipboardItems 
                WHERE Type != @UnknownType
                ORDER BY IsPinned DESC, CreatedAt DESC
                LIMIT @Limit OFFSET @Skip
                """;
        }
        else
        {
            // Use FTS5 for full-text search with prefix matching
            var searchTerm = query.Trim().Replace("\"", "\"\"", StringComparison.Ordinal) + "*";
            cmd.CommandText = """
                SELECT c.* FROM ClipboardItems c
                INNER JOIN ClipboardItems_fts fts ON c.rowid = fts.rowid
                WHERE ClipboardItems_fts MATCH @Query AND c.Type != @UnknownType
                ORDER BY c.IsPinned DESC, c.CreatedAt DESC
                LIMIT @Limit OFFSET @Skip
                """;
            cmd.Parameters.AddWithValue("@Query", searchTerm);
        }

        cmd.Parameters.AddWithValue("@UnknownType", (int)ClipboardContentType.Unknown);
        cmd.Parameters.AddWithValue("@Limit", limit);
        cmd.Parameters.AddWithValue("@Skip", skip);

        using var reader = cmd.ExecuteReader();
        var items = new List<ClipboardItem>();
        while (reader.Read())
        {
            items.Add(MapToItem(reader));
        }
        return items;
    }

    private static void AddParameters(SqliteCommand cmd, ClipboardItem item)
    {
        cmd.Parameters.AddWithValue("@Id", item.Id.ToString());
        cmd.Parameters.AddWithValue("@Content", item.Content);
        cmd.Parameters.AddWithValue("@Type", (int)item.Type);
        cmd.Parameters.AddWithValue("@CreatedAt", item.CreatedAt.ToString("O"));
        cmd.Parameters.AddWithValue("@ModifiedAt", item.ModifiedAt.ToString("O"));
        cmd.Parameters.AddWithValue("@AppSource", item.AppSource ?? (object)DBNull.Value);
        cmd.Parameters.AddWithValue("@IsPinned", item.IsPinned ? 1 : 0);
        cmd.Parameters.AddWithValue("@Metadata", item.Metadata ?? (object)DBNull.Value);
    }

    private static ClipboardItem MapToItem(SqliteDataReader reader) => new()
    {
        Id = Guid.Parse(reader.GetString(reader.GetOrdinal("Id"))),
        Content = reader.GetString(reader.GetOrdinal("Content")),
        Type = (ClipboardContentType)reader.GetInt32(reader.GetOrdinal("Type")),
        CreatedAt = DateTime.Parse(reader.GetString(reader.GetOrdinal("CreatedAt")), System.Globalization.CultureInfo.InvariantCulture),
        ModifiedAt = DateTime.Parse(reader.GetString(reader.GetOrdinal("ModifiedAt")), System.Globalization.CultureInfo.InvariantCulture),
        AppSource = reader.IsDBNull(reader.GetOrdinal("AppSource")) ? null : reader.GetString(reader.GetOrdinal("AppSource")),
        IsPinned = reader.GetInt32(reader.GetOrdinal("IsPinned")) == 1,
        Metadata = reader.IsDBNull(reader.GetOrdinal("Metadata")) ? null : reader.GetString(reader.GetOrdinal("Metadata"))
    };

    private static void CleanupItemFiles(ClipboardItem item)
    {
        // Clean up image backup
        if (item.Type == ClipboardContentType.Image && !string.IsNullOrEmpty(item.Content) && File.Exists(item.Content))
        {
            try { File.Delete(item.Content); }
            catch { /* Ignore */ }
        }

        // Clean up thumbnail
        var thumbPath = Path.Combine(StorageConfig.ThumbnailsPath, $"{item.Id}_t.png");
        if (File.Exists(thumbPath))
        {
            try { File.Delete(thumbPath); }
            catch { /* Ignore */ }
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        // SQLite connections are closed automatically when disposed
        // WAL checkpoint for clean shutdown
        try
        {
            using var connection = CreateConnection();
            ExecuteNonQuery(connection, "PRAGMA wal_checkpoint(TRUNCATE)");
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"WAL checkpoint failed: {ex.Message}");
        }
    }
}
