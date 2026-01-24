namespace CopyPaste.Core;

public interface IClipboardRepository
{
    void Save(ClipboardItem item);
    IEnumerable<ClipboardItem> GetAll();
    void Delete(Guid id);

    int ClearOldItems(int days);

    IEnumerable<ClipboardItem> Search(string query);
}
