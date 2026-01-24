using System.Text.Json.Serialization;

namespace CopyPaste.Core;

// This enables NativeAOT-compatible JSON serialization for Metadata
[JsonSerializable(typeof(Dictionary<string, object>))]
internal sealed partial class MetadataJsonContext : JsonSerializerContext
{
}
