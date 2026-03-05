using Microsoft.Data.Sqlite;

namespace CopyPaste.Core;

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
            AppLogger.Warn($"Database file is not valid SQLite, recreating: {ex.Message}");
            HandleCorruptDatabase();
            InitializeDatabase();
        }
        catch (SqliteException ex)
        {
            AppLogger.Exception(ex, "SQLite error during initialization, recreating database");
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
            AppLogger.Info($"Moved incompatible database to: {backupPath}");

            // Also remove WAL/SHM files if they exist
            var walPath = _dbPath + "-wal";
            var shmPath = _dbPath + "-shm";
            if (File.Exists(walPath)) File.Delete(walPath);
            if (File.Exists(shmPath)) File.Delete(shmPath);
        }
        catch (IOException ex)
        {
            AppLogger.Exception(ex, "Failed to backup corrupt database");
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

        ExecuteNonQuery(connection, """
            CREATE TABLE IF NOT EXISTS ClipboardItems (
                Id TEXT PRIMARY KEY,
                Content TEXT NOT NULL,
                Type INTEGER NOT NULL,
                CreatedAt TEXT NOT NULL,
                ModifiedAt TEXT NOT NULL,
                AppSource TEXT,
                IsPinned INTEGER NOT NULL DEFAULT 0,
                Metadata TEXT,
                Label TEXT,
                CardColor INTEGER NOT NULL DEFAULT 0,
                PasteCount INTEGER NOT NULL DEFAULT 0,
                ContentHash TEXT
            )
            """);

        MigrateAddColumnIfMissing(connection, "Label", "TEXT");
        MigrateAddColumnIfMissing(connection, "CardColor", "INTEGER NOT NULL DEFAULT 0");
        MigrateAddColumnIfMissing(connection, "PasteCount", "INTEGER NOT NULL DEFAULT 0");
        MigrateAddColumnIfMissing(connection, "ContentHash", "TEXT");

        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_CreatedAt ON ClipboardItems(CreatedAt DESC)");
        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_ModifiedAt ON ClipboardItems(ModifiedAt DESC)");
        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_Type ON ClipboardItems(Type)");
        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_IsPinned ON ClipboardItems(IsPinned)");
        ExecuteNonQuery(connection, "CREATE INDEX IF NOT EXISTS IX_ClipboardItems_ContentHash ON ClipboardItems(ContentHash)");

        ExecuteNonQuery(connection, """
            CREATE VIRTUAL TABLE IF NOT EXISTS ClipboardItems_fts USING fts5(
                Content,
                AppSource,
                Label,
                content='ClipboardItems',
                content_rowid='rowid'
            )
            """);

        RebuildFtsIfNeeded(connection);

        ExecuteNonQuery(connection, """
            CREATE TRIGGER IF NOT EXISTS ClipboardItems_ai AFTER INSERT ON ClipboardItems BEGIN
                INSERT INTO ClipboardItems_fts(rowid, Content, AppSource, Label)
                VALUES (NEW.rowid, NEW.Content, NEW.AppSource, NEW.Label);
            END
            """);

        ExecuteNonQuery(connection, """
            CREATE TRIGGER IF NOT EXISTS ClipboardItems_ad AFTER DELETE ON ClipboardItems BEGIN
                INSERT INTO ClipboardItems_fts(ClipboardItems_fts, rowid, Content, AppSource, Label)
                VALUES ('delete', OLD.rowid, OLD.Content, OLD.AppSource, OLD.Label);
            END
            """);

        ExecuteNonQuery(connection, """
            CREATE TRIGGER IF NOT EXISTS ClipboardItems_au AFTER UPDATE ON ClipboardItems BEGIN
                INSERT INTO ClipboardItems_fts(ClipboardItems_fts, rowid, Content, AppSource, Label)
                VALUES ('delete', OLD.rowid, OLD.Content, OLD.AppSource, OLD.Label);
                INSERT INTO ClipboardItems_fts(rowid, Content, AppSource, Label)
                VALUES (NEW.rowid, NEW.Content, NEW.AppSource, NEW.Label);
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
            INSERT INTO ClipboardItems (Id, Content, Type, CreatedAt, ModifiedAt, AppSource, IsPinned, Metadata, Label, CardColor, PasteCount, ContentHash)
            VALUES (@Id, @Content, @Type, @CreatedAt, @ModifiedAt, @AppSource, @IsPinned, @Metadata, @Label, @CardColor, @PasteCount, @ContentHash)
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
                AppSource = @AppSource, IsPinned = @IsPinned, Metadata = @Metadata,
                Label = @Label, CardColor = @CardColor, PasteCount = @PasteCount, ContentHash = @ContentHash
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
            ORDER BY ModifiedAt DESC
            LIMIT 1
            """;
        cmd.Parameters.AddWithValue("@UnknownType", (int)ClipboardContentType.Unknown);

        using var reader = cmd.ExecuteReader();
        return reader.Read() ? MapToItem(reader) : null;
    }

    public ClipboardItem? FindByContentHash(string contentHash)
    {
        if (string.IsNullOrEmpty(contentHash)) return null;

        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        cmd.CommandText = """
            SELECT * FROM ClipboardItems
            WHERE ContentHash = @ContentHash
            LIMIT 1
            """;
        cmd.Parameters.AddWithValue("@ContentHash", contentHash);

        using var reader = cmd.ExecuteReader();
        return reader.Read() ? MapToItem(reader) : null;
    }

    public ClipboardItem? FindByContentAndType(string content, ClipboardContentType type)
    {
        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        cmd.CommandText = """
            SELECT * FROM ClipboardItems
            WHERE Content = @Content AND Type = @Type
            LIMIT 1
            """;
        cmd.Parameters.AddWithValue("@Content", content);
        cmd.Parameters.AddWithValue("@Type", (int)type);

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

        foreach (var item in itemsToDelete)
        {
            CleanupItemFiles(item);
        }

        using var deleteCmd = connection.CreateCommand();
        deleteCmd.CommandText = "DELETE FROM ClipboardItems WHERE CreatedAt < @LimitDate" + (excludePinned ? " AND IsPinned = 0" : "");
        deleteCmd.Parameters.AddWithValue("@LimitDate", limitDate.ToString("O"));

        var deletedCount = deleteCmd.ExecuteNonQuery();

        if (deletedCount > 50)
        {
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

            var trimmedQuery = query.Trim();
            var ftsQuery = trimmedQuery.Replace("\"", "\"\"", StringComparison.Ordinal) + "*";
            var likePattern = "%" + trimmedQuery.Replace("%", "[%]", StringComparison.Ordinal)
                                                .Replace("_", "[_]", StringComparison.Ordinal) + "%";

            cmd.CommandText = """
                WITH fts_results AS (
                    SELECT c.*, bm25(ClipboardItems_fts) AS rank, 1 AS source
                    FROM ClipboardItems c
                    INNER JOIN ClipboardItems_fts fts ON c.rowid = fts.rowid
                    WHERE ClipboardItems_fts MATCH @FtsQuery AND c.Type != @UnknownType
                ),
                like_results AS (
                    SELECT c.*, 0.0 AS rank, 2 AS source
                    FROM ClipboardItems c
                    WHERE c.Type != @UnknownType
                      AND (c.Content LIKE @LikePattern COLLATE NOCASE
                           OR c.Label LIKE @LikePattern COLLATE NOCASE
                           OR c.AppSource LIKE @LikePattern COLLATE NOCASE)
                      AND c.Id NOT IN (SELECT Id FROM fts_results)
                )
                SELECT * FROM (
                    SELECT * FROM fts_results
                    UNION ALL
                    SELECT * FROM like_results
                )
                ORDER BY IsPinned DESC, source ASC, rank ASC, ModifiedAt DESC
                LIMIT @Limit OFFSET @Skip
                """;
            cmd.Parameters.AddWithValue("@FtsQuery", ftsQuery);
            cmd.Parameters.AddWithValue("@LikePattern", likePattern);
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

#pragma warning disable CA2100 // Dynamic SQL is safe - only uses parameterized column/parameter names
    public IEnumerable<ClipboardItem> SearchAdvanced(
        string? query,
        IReadOnlyCollection<ClipboardContentType>? types,
        IReadOnlyCollection<CardColor>? colors,
        bool? isPinned,
        int limit,
        int skip)
    {
        using var connection = CreateConnection();
        using var cmd = connection.CreateCommand();

        var hasTextQuery = !string.IsNullOrWhiteSpace(query);
        var hasTypeFilter = types is { Count: > 0 };
        var hasColorFilter = colors is { Count: > 0 };
        var hasAnyFilter = hasTextQuery || hasTypeFilter || hasColorFilter;

        // Build dynamic WHERE clause
        var conditions = new List<string> { "c.Type != @UnknownType" };

        // When searching/filtering, search ALL items (PRO mode)
        // Only apply isPinned filter in normal mode (no filters)
        if (!hasAnyFilter && isPinned.HasValue)
        {
            conditions.Add("c.IsPinned = @IsPinned");
            cmd.Parameters.AddWithValue("@IsPinned", isPinned.Value ? 1 : 0);
        }

        if (hasTypeFilter && types is not null)
        {
            var typeParams = new List<string>();
            for (int i = 0; i < types.Count; i++)
            {
                var paramName = $"@Type{i}";
                typeParams.Add(paramName);
                cmd.Parameters.AddWithValue(paramName, (int)types.ElementAt(i));
            }
            conditions.Add($"c.Type IN ({string.Join(",", typeParams)})");
        }

        if (hasColorFilter && colors is not null)
        {
            var colorParams = new List<string>();
            for (int i = 0; i < colors.Count; i++)
            {
                var paramName = $"@Color{i}";
                colorParams.Add(paramName);
                cmd.Parameters.AddWithValue(paramName, (int)colors.ElementAt(i));
            }
            conditions.Add($"c.CardColor IN ({string.Join(",", colorParams)})");
        }

        var filterClause = string.Join(" AND ", conditions);

        if (hasTextQuery)
        {
            var trimmedQuery = query!.Trim();
            var ftsQuery = trimmedQuery.Replace("\"", "\"\"", StringComparison.Ordinal) + "*";
            var likePattern = "%" + trimmedQuery.Replace("%", "[%]", StringComparison.Ordinal)
                                                .Replace("_", "[_]", StringComparison.Ordinal) + "%";

            cmd.CommandText = $"""
                WITH fts_results AS (
                    SELECT c.*, bm25(ClipboardItems_fts) AS rank, 1 AS source
                    FROM ClipboardItems c
                    INNER JOIN ClipboardItems_fts fts ON c.rowid = fts.rowid
                    WHERE ClipboardItems_fts MATCH @FtsQuery AND {filterClause}
                ),
                like_results AS (
                    SELECT c.*, 0.0 AS rank, 2 AS source
                    FROM ClipboardItems c
                    WHERE {filterClause}
                      AND (c.Content LIKE @LikePattern COLLATE NOCASE
                           OR c.Label LIKE @LikePattern COLLATE NOCASE
                           OR c.AppSource LIKE @LikePattern COLLATE NOCASE)
                      AND c.Id NOT IN (SELECT Id FROM fts_results)
                )
                SELECT * FROM (
                    SELECT * FROM fts_results
                    UNION ALL
                    SELECT * FROM like_results
                )
                ORDER BY IsPinned DESC, source ASC, rank ASC, ModifiedAt DESC
                LIMIT @Limit OFFSET @Skip
                """;
            cmd.Parameters.AddWithValue("@FtsQuery", ftsQuery);
            cmd.Parameters.AddWithValue("@LikePattern", likePattern);
        }
        else
        {
            cmd.CommandText = $"""
                SELECT c.* FROM ClipboardItems c
                WHERE {filterClause}
                ORDER BY c.IsPinned DESC, c.ModifiedAt DESC
                LIMIT @Limit OFFSET @Skip
                """;
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
#pragma warning restore CA2100

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
        cmd.Parameters.AddWithValue("@Label", item.Label ?? (object)DBNull.Value);
        cmd.Parameters.AddWithValue("@CardColor", (int)item.CardColor);
        cmd.Parameters.AddWithValue("@PasteCount", item.PasteCount);
        cmd.Parameters.AddWithValue("@ContentHash", item.ContentHash ?? (object)DBNull.Value);
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
        Metadata = reader.IsDBNull(reader.GetOrdinal("Metadata")) ? null : reader.GetString(reader.GetOrdinal("Metadata")),
        Label = GetNullableString(reader, "Label"),
        CardColor = GetCardColor(reader, "CardColor"),
        PasteCount = GetIntOrDefault(reader, "PasteCount"),
        ContentHash = GetNullableString(reader, "ContentHash")
    };

    private static void CleanupItemFiles(ClipboardItem item)
    {
        if (item.Type == ClipboardContentType.Image && !string.IsNullOrEmpty(item.Content) && File.Exists(item.Content))
        {
            try { File.Delete(item.Content); }
            catch { /* Ignore */ }
        }

        var thumbBaseName = $"{item.Id}_t";
        var thumbDir = StorageConfig.ThumbnailsPath;
        var possibleExtensions = new[] { ".png", ".jpg", ".jpeg", ".webp" };

        foreach (var ext in possibleExtensions)
        {
            var thumbPath = Path.Combine(thumbDir, thumbBaseName + ext);
            if (File.Exists(thumbPath))
            {
                try { File.Delete(thumbPath); }
                catch { /* Ignore */ }
            }
        }
    }

    private static int GetIntOrDefault(SqliteDataReader reader, string columnName)
    {
        try
        {
            var ordinal = reader.GetOrdinal(columnName);
            return reader.IsDBNull(ordinal) ? 0 : reader.GetInt32(ordinal);
        }
        catch (ArgumentOutOfRangeException)
        {
            return 0;
        }
    }

    private static string? GetNullableString(SqliteDataReader reader, string columnName)
    {
        try
        {
            var ordinal = reader.GetOrdinal(columnName);
            return reader.IsDBNull(ordinal) ? null : reader.GetString(ordinal);
        }
        catch (ArgumentOutOfRangeException)
        {
            return null;
        }
    }

    private static CardColor GetCardColor(SqliteDataReader reader, string columnName)
    {
        try
        {
            var ordinal = reader.GetOrdinal(columnName);
            if (reader.IsDBNull(ordinal))
                return CardColor.None;

            var value = reader.GetInt32(ordinal);
            return Enum.IsDefined(typeof(CardColor), value) ? (CardColor)value : CardColor.None;
        }
        catch (ArgumentOutOfRangeException)
        {
            return CardColor.None;
        }
    }

    private static void MigrateAddColumnIfMissing(SqliteConnection connection, string columnName, string columnType)
    {
        using var cmd = connection.CreateCommand();
        cmd.CommandText = $"PRAGMA table_info(ClipboardItems)";

        var columnExists = false;
        using (var reader = cmd.ExecuteReader())
        {
            while (reader.Read())
            {
                var name = reader.GetString(reader.GetOrdinal("name"));
                if (string.Equals(name, columnName, StringComparison.OrdinalIgnoreCase))
                {
                    columnExists = true;
                    break;
                }
            }
        }

        if (!columnExists)
        {
            AppLogger.Info($"Migrating database: adding column {columnName}");
            ExecuteNonQuery(connection, $"ALTER TABLE ClipboardItems ADD COLUMN {columnName} {columnType}");
        }
    }

    private static void RebuildFtsIfNeeded(SqliteConnection connection)
    {
        try
        {
            using var testCmd = connection.CreateCommand();
            testCmd.CommandText = "SELECT Label FROM ClipboardItems_fts LIMIT 0";
            testCmd.ExecuteNonQuery();
        }
        catch (SqliteException)
        {
            AppLogger.Info("Rebuilding FTS index to include Label column");

            ExecuteNonQuery(connection, "DROP TRIGGER IF EXISTS ClipboardItems_ai");
            ExecuteNonQuery(connection, "DROP TRIGGER IF EXISTS ClipboardItems_ad");
            ExecuteNonQuery(connection, "DROP TRIGGER IF EXISTS ClipboardItems_au");
            ExecuteNonQuery(connection, "DROP TABLE IF EXISTS ClipboardItems_fts");
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        try
        {
            using var connection = CreateConnection();
            ExecuteNonQuery(connection, "PRAGMA wal_checkpoint(TRUNCATE)");
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "WAL checkpoint failed during dispose");
        }
    }
}
