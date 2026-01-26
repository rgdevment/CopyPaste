using System.Globalization;
using System.Text;

namespace CopyPaste.Core;

/// <summary>
/// Helper class for clipboard search functionality.
/// Simple text search with type filtering based on database Type field.
/// </summary>
internal static class SearchHelper
{
    public static string NormalizeText(string text)
    {
        if (string.IsNullOrEmpty(text))
            return text;

        var normalizedString = text.Normalize(NormalizationForm.FormD);
        var stringBuilder = new StringBuilder();

        foreach (var c in normalizedString)
        {
            var unicodeCategory = CharUnicodeInfo.GetUnicodeCategory(c);
            if (unicodeCategory != UnicodeCategory.NonSpacingMark)
            {
                stringBuilder.Append(c);
            }
        }

        return stringBuilder.ToString().Normalize(NormalizationForm.FormC);
    }

    public static bool MatchesQuery(ClipboardItem item, string query)
    {
        var normalizedQuery = NormalizeText(query.Trim()).ToUpperInvariant();

        if (string.IsNullOrWhiteSpace(normalizedQuery))
            return true;

        // Check if query is a type keyword
        var matchedType = GetTypeFromKeyword(normalizedQuery);
        if (matchedType != null)
        {
            // Filter by Type field from database (source of truth)
            return item.Type == matchedType.Value;
        }

        // Not a type keyword - search in content as plain text
        var content = NormalizeText(item.Content ?? string.Empty).ToUpperInvariant();
        return content.Contains(normalizedQuery, StringComparison.Ordinal);
    }

    private static ClipboardContentType? GetTypeFromKeyword(string query) => query switch
    {
        "IMAGE" or "IMAGEN" or "IMG" => ClipboardContentType.Image,
        "VIDEO" => ClipboardContentType.Video,
        "AUDIO" or "MUSICA" or "CANCION" => ClipboardContentType.Audio,
        "FILE" or "ARCHIVO" => ClipboardContentType.File,
        "FOLDER" or "CARPETA" or "DIRECTORIO" => ClipboardContentType.Folder,
        "LINK" or "ENLACE" or "URL" => ClipboardContentType.Link,
        "TEXT" or "TEXTO" => ClipboardContentType.Text,
        _ => null
    };
}

