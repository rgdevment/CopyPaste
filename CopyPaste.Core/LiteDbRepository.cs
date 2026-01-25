using LiteDB;

namespace CopyPaste.Core;

public class LiteDbRepository : IClipboardRepository
{
    private readonly string _connectionString;
    private const string _collectionName = "items";

    public LiteDbRepository(string dbPath)
    {
        _connectionString = $"Filename={dbPath};Connection=shared";
        ConfigureMapper();
    }

    private static void ConfigureMapper() =>
        BsonMapper.Global.RegisterType<ClipboardContentType>(
            serialize: type => type.ToString(),
            deserialize: bson => Enum.TryParse<ClipboardContentType>(bson.AsString, out var result) ? result : ClipboardContentType.Unknown
        );

    public void Save(ClipboardItem item)
    {
        using var db = new LiteDatabase(_connectionString);
        var col = db.GetCollection<ClipboardItem>(_collectionName);

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

        return col.Query()
                  .OrderByDescending(x => x.CreatedAt)
                  .ToEnumerable()
                  .FirstOrDefault(x => x.Type != ClipboardContentType.Unknown);
    }

    public IEnumerable<ClipboardItem> GetAll()
    {
        using var db = new LiteDatabase(_connectionString);
        return [.. db.GetCollection<ClipboardItem>(_collectionName)
                 .FindAll()
                 .Where(x => x.Type != ClipboardContentType.Unknown)
                 .OrderByDescending(x => x.CreatedAt)];
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
        return [.. db.GetCollection<ClipboardItem>(_collectionName)
                 .Find(x => x.Content.Contains(query, StringComparison.OrdinalIgnoreCase))
                 .Where(x => x.Type != ClipboardContentType.Unknown)
                 .OrderByDescending(x => x.CreatedAt)];
    }
}
