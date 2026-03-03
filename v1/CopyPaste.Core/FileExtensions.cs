using System.Collections.Frozen;

namespace CopyPaste.Core;

public static class FileExtensions
{
    private static readonly FrozenDictionary<string, ClipboardContentType> _map = new Dictionary<string, ClipboardContentType>
    {
        // Audio
        [".MP3"] = ClipboardContentType.Audio,
        [".WAV"] = ClipboardContentType.Audio,
        [".FLAC"] = ClipboardContentType.Audio,
        [".AAC"] = ClipboardContentType.Audio,
        [".OGG"] = ClipboardContentType.Audio,
        [".WMA"] = ClipboardContentType.Audio,
        [".M4A"] = ClipboardContentType.Audio,

        // Video
        [".MP4"] = ClipboardContentType.Video,
        [".AVI"] = ClipboardContentType.Video,
        [".MKV"] = ClipboardContentType.Video,
        [".MOV"] = ClipboardContentType.Video,
        [".WMV"] = ClipboardContentType.Video,
        [".FLV"] = ClipboardContentType.Video,
        [".WEBM"] = ClipboardContentType.Video,

        // Image
        [".PNG"] = ClipboardContentType.Image,
        [".JPG"] = ClipboardContentType.Image,
        [".JPEG"] = ClipboardContentType.Image,
        [".GIF"] = ClipboardContentType.Image,
        [".BMP"] = ClipboardContentType.Image,
        [".WEBP"] = ClipboardContentType.Image,
        [".SVG"] = ClipboardContentType.Image,
        [".ICO"] = ClipboardContentType.Image,
    }.ToFrozenDictionary();

    public static ClipboardContentType GetContentType(string? extension) =>
        string.IsNullOrEmpty(extension)
            ? ClipboardContentType.File
            : _map.GetValueOrDefault(extension.ToUpperInvariant(), ClipboardContentType.File);
}
