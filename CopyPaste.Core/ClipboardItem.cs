namespace CopyPaste.Core;

public class ClipboardItem
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Content { get; set; } = string.Empty;
    public ClipboardContentType Type { get; set; }
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
    public DateTime ModifiedAt { get; set; } = DateTime.UtcNow;
    public string? AppSource { get; set; }
    public bool IsPinned { get; set; }

    // Multi-purpose field for thumbnails, file info, or text tags
    public string? Metadata { get; set; }
}
