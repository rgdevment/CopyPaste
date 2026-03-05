namespace CopyPaste.Core;

public interface IClipboardRepository
{
    void Save(ClipboardItem item);
    void Update(ClipboardItem item);
    ClipboardItem? GetById(Guid id);
    ClipboardItem? GetLatest();
    ClipboardItem? FindByContentAndType(string content, ClipboardContentType type);
    IEnumerable<ClipboardItem> GetAll();
    void Delete(Guid id);
    int ClearOldItems(int days, bool excludePinned = true);
    IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0);

    /// <summary>
    /// Advanced search with all filters applied at DB level.
    /// When any filter is active, searches ALL items (ignores isPinned tab filter).
    /// </summary>
    IEnumerable<ClipboardItem> SearchAdvanced(
        string? query,
        IReadOnlyCollection<ClipboardContentType>? types,
        IReadOnlyCollection<CardColor>? colors,
        bool? isPinned,
        int limit,
        int skip);

    /// <summary>
    /// Finds an item by its ContentHash (indexed for fast lookups).
    /// Primarily used for image deduplication.
    /// </summary>
    ClipboardItem? FindByContentHash(string contentHash);
}
