using System.Text.Json.Serialization;

namespace CopyPaste.Core;

[JsonSerializable(typeof(ClipboardItem))]
[JsonSerializable(typeof(Dictionary<string, object>))]
[JsonSerializable(typeof(int))]
[JsonSerializable(typeof(string))]
[JsonSerializable(typeof(long))]
public partial class MetadataJsonContext : JsonSerializerContext
{
}
