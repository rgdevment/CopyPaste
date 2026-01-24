using LiteDB;

namespace CopyPaste.Core;

public class LiteDbRepository(string dbPath) : IClipboardRepository
{
    private readonly string _connectionString = $"Filename={dbPath};Connection=shared";

    public void Save(ClipboardItem item)
    {
        using var db = new LiteDatabase(_connectionString);
        var col = db.GetCollection<ClipboardItem>("items");

        // Ensure index for search performance
        col.EnsureIndex(x => x.Content);
        col.Upsert(item);
    }

    public IEnumerable<ClipboardItem> GetAll()
    {
        using var db = new LiteDatabase(_connectionString);
        return db.GetCollection<ClipboardItem>("items")
                 .FindAll()
                 .OrderByDescending(x => x.CreatedAt)
                 .ToList();
    }

    public void Delete(Guid id)
    {
        using var db = new LiteDatabase(_connectionString);
        var col = db.GetCollection<ClipboardItem>("items");

        var item = col.FindById(id);
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
        return db.GetCollection<ClipboardItem>("items")
                 .DeleteMany(x => x.CreatedAt < limitDate);
    }

    public IEnumerable<ClipboardItem> Search(string query)
    {
        if (string.IsNullOrWhiteSpace(query)) return GetAll();

        using var db = new LiteDatabase(_connectionString);
        return db.GetCollection<ClipboardItem>("items")
                 .Find(x => x.Content.Contains(query, StringComparison.OrdinalIgnoreCase))
                 .OrderByDescending(x => x.CreatedAt)
                 .ToList();
    }
}
