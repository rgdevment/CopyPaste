using System.Text.Json.Serialization;

namespace CopyPaste.Core;

public sealed class BackupManifest
{
    public int Version { get; set; } = 1;
    public string AppVersion { get; set; } = string.Empty;
    public DateTime CreatedAtUtc { get; set; }
    public int ItemCount { get; set; }
    public int ImageCount { get; set; }
    public int ThumbnailCount { get; set; }
    public bool HasPinnedItems { get; set; }
    public string MachineName { get; set; } = string.Empty;
}

[JsonSerializable(typeof(BackupManifest))]
[JsonSourceGenerationOptions(
    WriteIndented = true,
    PropertyNameCaseInsensitive = true)]
public partial class BackupManifestJsonContext : JsonSerializerContext
{
}
