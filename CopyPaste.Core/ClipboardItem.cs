namespace CopyPaste.Core;

public class ClipboardItem
{
    public const int MaxLabelLength = 40;

    public Guid Id { get; set; } = Guid.NewGuid();
    public string Content { get; set; } = string.Empty;
    public ClipboardContentType Type { get; set; }
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
    public DateTime ModifiedAt { get; set; } = DateTime.UtcNow;
    public string? AppSource { get; set; }
    public bool IsPinned { get; set; }

    public string? Label { get; set; }

    public CardColor CardColor { get; set; } = CardColor.None;

    public string? Metadata { get; set; }

    public int PasteCount { get; set; }

    public string? ContentHash { get; set; }

    public bool IsFileBasedType =>
        Type is ClipboardContentType.File
            or ClipboardContentType.Folder
            or ClipboardContentType.Audio
            or ClipboardContentType.Video;

    public bool IsFileAvailable()
    {
        if (!IsFileBasedType) return true;
        if (string.IsNullOrEmpty(Content)) return false;

        var paths = Content.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries);
        if (paths.Length == 0) return false;

        return File.Exists(paths[0]) || Directory.Exists(paths[0]);
    }
}
