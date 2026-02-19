using System.Collections.ObjectModel;

namespace CopyPaste.Core;

public interface IClipboardService
{
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1003", Justification = "Action<T> is simpler and sufficient for internal eventing")]
    event Action<ClipboardItem>? OnItemAdded;

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1003", Justification = "Action<T> is simpler and sufficient for internal eventing")]
    event Action<ClipboardItem>? OnThumbnailReady;

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1003", Justification = "Action<T> is simpler and sufficient for internal eventing")]
    event Action<ClipboardItem>? OnItemReactivated;

    void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null, byte[]? htmlBytes = null);
    void AddImage(byte[]? dibData, string? source);
    void AddFiles(Collection<string>? files, ClipboardContentType type, string? source);

    IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? query = null, bool? isPinned = null);

    IEnumerable<ClipboardItem> GetHistoryAdvanced(
        int limit,
        int skip,
        string? query,
        IReadOnlyCollection<ClipboardContentType>? types,
        IReadOnlyCollection<CardColor>? colors,
        bool? isPinned);

    void RemoveItem(Guid id);
    void UpdatePin(Guid id, bool isPinned);
    void UpdateLabelAndColor(Guid id, string? label, CardColor color);
    ClipboardItem? MarkItemUsed(Guid id);
    void NotifyPasteInitiated(Guid itemId);

    int PasteIgnoreWindowMs { get; set; }
}
