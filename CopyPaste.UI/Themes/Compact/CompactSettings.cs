using CopyPaste.Core;
using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace CopyPaste.UI.Themes;

public sealed class CompactSettings
{
    private const string _fileName = "CompactTheme.json";

    public int PopupWidth { get; set; } = 368;
    public int PopupHeight { get; set; } = 480;
    public int CardMinLines { get; set; } = 2;
    public int CardMaxLines { get; set; } = 5;
    public bool HideOnDeactivate { get; set; } = true;
    public bool ResetScrollOnShow { get; set; } = true;
    public bool ResetSearchOnShow { get; set; } = true;
    public bool ResetFilterModeOnShow { get; set; } = true;
    public bool ResetCategoryFilterOnShow { get; set; } = true;
    public bool ResetTypeFilterOnShow { get; set; } = true;

    private static string FilePath => Path.Combine(StorageConfig.ConfigPath, _fileName);

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Settings loading should not crash app - failures are logged and defaults returned")]
    public static CompactSettings Load()
    {
        try
        {
            if (!File.Exists(FilePath))
            {
                var defaults = new CompactSettings();
                Save(defaults);
                return defaults;
            }

            var json = File.ReadAllText(FilePath);
            var settings = JsonSerializer.Deserialize(json, CompactSettingsJsonContext.Default.CompactSettings);
            if (settings != null)
            {
                AppLogger.Info($"Loaded theme settings from {_fileName}");
                return settings;
            }
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, $"Failed to load {_fileName}");
        }

        return new CompactSettings();
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Settings saving should not crash app - failures are logged")]
    public static bool Save(CompactSettings settings)
    {
        try
        {
            var configDir = StorageConfig.ConfigPath;
            if (!Directory.Exists(configDir))
                Directory.CreateDirectory(configDir);

            var json = JsonSerializer.Serialize(settings, CompactSettingsJsonContext.Default.CompactSettings);
            File.WriteAllText(FilePath, json);
            AppLogger.Info($"Saved theme settings to {FilePath}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to save theme settings");
            return false;
        }
    }
}

[JsonSerializable(typeof(CompactSettings))]
[JsonSourceGenerationOptions(
    WriteIndented = true,
    PropertyNameCaseInsensitive = true,
    ReadCommentHandling = JsonCommentHandling.Skip,
    AllowTrailingCommas = true)]
internal sealed partial class CompactSettingsJsonContext : JsonSerializerContext
{
}
