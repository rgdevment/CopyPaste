namespace CopyPaste.Core;

public class ClipboardItem
{
    /// <summary>Maximum length for user-defined labels.</summary>
    public const int MaxLabelLength = 40;

    public Guid Id { get; set; } = Guid.NewGuid();
    public string Content { get; set; } = string.Empty;
    public ClipboardContentType Type { get; set; }
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
    public DateTime ModifiedAt { get; set; } = DateTime.UtcNow;
    public string? AppSource { get; set; }
    public bool IsPinned { get; set; }

    /// <summary>
    /// Optional user-defined label for easy identification.
    /// Limited to <see cref="MaxLabelLength"/> characters.
    /// </summary>
    public string? Label { get; set; }

    /// <summary>
    /// Optional color for visual organization.
    /// </summary>
    public CardColor CardColor { get; set; } = CardColor.None;

    /// <summary>
    /// Multi-purpose field for thumbnails, file info, or text tags.
    /// </summary>
    public string? Metadata { get; set; }
}
