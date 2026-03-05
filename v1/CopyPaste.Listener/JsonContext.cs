using System.Text.Json.Serialization;

namespace CopyPaste.Listener;

[JsonSerializable(typeof(Dictionary<string, object>))]
internal sealed partial class MetadataJsonContext : JsonSerializerContext
{
}
