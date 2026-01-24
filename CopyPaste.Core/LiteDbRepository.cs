using LiteDB;

namespace CopyPaste.Core;

public class LiteDbRepository(string dbPath) : IClipboardRepository
{
    private readonly string _connectionString = $"Filename={dbPath};Connection=shared";
    private const string _collectionName = "items";

    public void Save(ClipboardItem item)
    {
        using var db = new LiteDatabase(_connectionString);
        var col = db.GetCollection<ClipboardItem>(_collectionName);

        // Ensure indices for performance
        col.EnsureIndex(x => x.CreatedAt);

        col.Insert(item);
    }

    public void Update(ClipboardItem item)
    {
        using var db = new LiteDatabase(_connectionString);
        var col = db.GetCollection<ClipboardItem>(_collectionName);

        col.Update(item);
    }

    public ClipboardItem? GetLatest()
    {
        using var db = new LiteDatabase(_connectionString);
        var col = db.GetCollection<ClipboardItem>(_collectionName);

        // Retrieve most recent entry for deduplication check
        return col.Query()
                  .OrderByDescending(x => x.CreatedAt)
                  .FirstOrDefault();
    }

    public IEnumerable<ClipboardItem> GetAll()
    {
        using var db = new LiteDatabase(_connectionString);
        return db.GetCollection<ClipboardItem>(_collectionName)
                 .FindAll()
                 .OrderByDescending(x => x.CreatedAt)
                 .ToList();
    }

    public void Delete(Guid id)
    {
        using var db = new LiteDatabase(_connectionString);
        var col = db.GetCollection<ClipboardItem>(_collectionName);

        var item = col.FindById(id);

        // Clean up physical image file before removing record
        if (item?.Type == ClipboardContentType.Image && File.Exists(item.Content))
        {
            File.Delete(item.Content);
        }

        col.Delete(id);
    }

    public int ClearOldItems(int days)
    {
        using var db = new LiteDatabase(_connectionString);
        var limitDate = DateTime.UtcNow.AddDays(-days);
        return db.GetCollection<ClipboardItem>(_collectionName)
                 .DeleteMany(x => x.CreatedAt < limitDate);
    }

    public IEnumerable<ClipboardItem> Search(string query)
    {
        if (string.IsNullOrWhiteSpace(query)) return GetAll();

        using var db = new LiteDatabase(_connectionString);
        return db.GetCollection<ClipboardItem>(_collectionName)
                 .Find(x => x.Content.Contains(query, StringComparison.OrdinalIgnoreCase))
                 .OrderByDescending(x => x.CreatedAt)
                 .ToList();
    }
}
