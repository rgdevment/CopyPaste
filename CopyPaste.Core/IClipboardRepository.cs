namespace CopyPaste.Core;

public interface IClipboardRepository
{
    void Save(ClipboardItem item);
    void Update(ClipboardItem item);
    ClipboardItem? GetLatest();
    IEnumerable<ClipboardItem> GetAll();
    void Delete(Guid id);
    int ClearOldItems(int days);
    IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0);
}
