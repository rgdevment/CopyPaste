using System.Globalization;
using System.Text;

namespace CopyPaste.Core;

/// <summary>
/// Helper class for advanced clipboard search functionality.
/// Handles text normalization, pattern matching, and filter logic.
/// </summary>
internal static class SearchHelper
{
    private static readonly char[] _wordSeparators = [' ', '\n', '\r', '\t', '\\', '/', '.', ',', ';', ':', '|', '-', '_'];
    private static readonly char[] _fileNameSeparators = ['\\', '/', '\n', '\r'];

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

        var content = NormalizeText(item.Content ?? string.Empty).ToUpperInvariant();
        var appSource = NormalizeText(item.AppSource ?? string.Empty).ToUpperInvariant();

        // Type-based search (keywords)
        if (IsTypeKeyword(normalizedQuery, out var matchedType))
            return item.Type == matchedType;

        var fileName = ExtractFileName(content);

        // Extension search (.ext or *.ext)
        if (IsExtensionPattern(normalizedQuery, out var extension))
        {
            return content.Contains($".{extension}", StringComparison.Ordinal) ||
                   fileName.EndsWith($".{extension}", StringComparison.Ordinal) ||
                   appSource.Contains($".{extension}", StringComparison.Ordinal);
        }

        // Wildcard patterns
        if (normalizedQuery.Length > 2 && normalizedQuery.StartsWith('*') && normalizedQuery.EndsWith('*'))
        {
            var searchTerm = normalizedQuery.AsSpan(1, normalizedQuery.Length - 2).ToString();
            return content.Contains(searchTerm, StringComparison.Ordinal) ||
                   appSource.Contains(searchTerm, StringComparison.Ordinal) ||
                   fileName.Contains(searchTerm, StringComparison.Ordinal);
        }

        if (normalizedQuery.Length > 1 && normalizedQuery.EndsWith('*'))
        {
            var searchTerm = normalizedQuery.AsSpan(0, normalizedQuery.Length - 1).ToString();
            return ContainsWord(content, searchTerm, isPrefix: true) ||
                   appSource.StartsWith(searchTerm, StringComparison.Ordinal) ||
                   fileName.StartsWith(searchTerm, StringComparison.Ordinal);
        }

        if (normalizedQuery.Length > 1 && normalizedQuery.StartsWith('*'))
        {
            var searchTerm = normalizedQuery.AsSpan(1).ToString();
            return ContainsWord(content, searchTerm, isSuffix: true) ||
                   appSource.EndsWith(searchTerm, StringComparison.Ordinal) ||
                   fileName.EndsWith(searchTerm, StringComparison.Ordinal);
        }

        // Default: exact word or substring match
        return ContainsWord(content, normalizedQuery) ||
               appSource.Contains(normalizedQuery, StringComparison.Ordinal) ||
               fileName.Contains(normalizedQuery, StringComparison.Ordinal);
    }

    private static bool IsTypeKeyword(string query, out ClipboardContentType type)
    {
        type = ClipboardContentType.Unknown;

        if (query.Contains("IMAGE", StringComparison.Ordinal) || query.Contains("IMAGEN", StringComparison.Ordinal))
        {
            type = ClipboardContentType.Image;
            return true;
        }

        if (query.Contains("VIDEO", StringComparison.Ordinal))
        {
            type = ClipboardContentType.Video;
            return true;
        }

        if (query.Contains("AUDIO", StringComparison.Ordinal) || query.Contains("MUSICA", StringComparison.Ordinal) || query.Contains("CANCION", StringComparison.Ordinal))
        {
            type = ClipboardContentType.Audio;
            return true;
        }

        if (query.Contains("FILE", StringComparison.Ordinal) || query.Contains("ARCHIVO", StringComparison.Ordinal))
        {
            type = ClipboardContentType.File;
            return true;
        }

        if (query.Contains("FOLDER", StringComparison.Ordinal) || query.Contains("CARPETA", StringComparison.Ordinal) || query.Contains("DIRECTORIO", StringComparison.Ordinal))
        {
            type = ClipboardContentType.Folder;
            return true;
        }

        if (query.Contains("LINK", StringComparison.Ordinal) || query.Contains("ENLACE", StringComparison.Ordinal) || query.Contains("URL", StringComparison.Ordinal))
        {
            type = ClipboardContentType.Link;
            return true;
        }

        if (query.Contains("TEXT", StringComparison.Ordinal) || query.Contains("TEXTO", StringComparison.Ordinal))
        {
            type = ClipboardContentType.Text;
            return true;
        }

        return false;
    }

    private static bool IsExtensionPattern(string query, out string extension)
    {
        extension = string.Empty;

        if ((query.Length > 1 && query.StartsWith('*')) || (query.Length > 0 && query.StartsWith('.')))
        {
            extension = query.TrimStart('*', '.').ToUpperInvariant();
            return true;
        }

        return false;
    }

    private static string ExtractFileName(string path)
    {
        if (string.IsNullOrEmpty(path))
            return string.Empty;

        var fileName = path.Split(_fileNameSeparators, StringSplitOptions.RemoveEmptyEntries).LastOrDefault() ?? string.Empty;
        return fileName.ToUpperInvariant();
    }

    private static bool ContainsWord(string content, string searchTerm, bool isPrefix = false, bool isSuffix = false)
    {
        if (string.IsNullOrEmpty(content) || string.IsNullOrEmpty(searchTerm))
            return false;

        if (isPrefix)
        {
            var words = content.Split(_wordSeparators, StringSplitOptions.RemoveEmptyEntries);
            return words.Any(w => w.StartsWith(searchTerm, StringComparison.Ordinal));
        }

        if (isSuffix)
        {
            var words = content.Split(_wordSeparators, StringSplitOptions.RemoveEmptyEntries);
            return words.Any(w => w.EndsWith(searchTerm, StringComparison.Ordinal));
        }

        var tokens = content.Split(_wordSeparators, StringSplitOptions.RemoveEmptyEntries);
        if (tokens.Any(w => w.Equals(searchTerm, StringComparison.Ordinal)))
            return true;

        return content.Contains(searchTerm, StringComparison.Ordinal);
    }
}
