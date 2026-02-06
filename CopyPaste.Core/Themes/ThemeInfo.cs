namespace CopyPaste.Core.Themes;

/// <summary>
/// Lightweight metadata about an available theme (without instantiating it).
/// Used by ConfigWindow to populate the theme selector.
/// </summary>
public sealed record ThemeInfo(
    string Id,
    string Name,
    string Version,
    string Author,
    bool IsCommunity);
