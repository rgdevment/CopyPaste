using CopyPaste.Core;
using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace CopyPaste.UI.Themes;

public sealed class DefaultThemeSettings
{
    private const string _fileName = "DefaultTheme.json";


    public int WindowWidth { get; set; } = 400;
    public int WindowMarginTop { get; set; } = 8;
    public int WindowMarginBottom { get; set; } = 16;
    public int CardMinLines { get; set; } = 3;
    public int CardMaxLines { get; set; } = 9;
    public bool PinWindow { get; set; }
    public bool ScrollToTopOnPaste { get; set; } = true;
    public bool ResetScrollOnShow { get; set; } = true;
    public bool ResetFilterModeOnShow { get; set; } = true;
    public bool ResetContentFilterOnShow { get; set; } = true;
    public bool ResetCategoryFilterOnShow { get; set; } = true;
    public bool ResetTypeFilterOnShow { get; set; } = true;

    private static string FilePath => Path.Combine(StorageConfig.ConfigPath, _fileName);

    public static DefaultThemeSettings Load()
    {
        try
        {
            if (!File.Exists(FilePath))
            {
                var defaults = new DefaultThemeSettings();
                Save(defaults);
                return defaults;
            }

            var json = File.ReadAllText(FilePath);
            var settings = JsonSerializer.Deserialize(json, DefaultThemeSettingsJsonContext.Default.DefaultThemeSettings);
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

        return new DefaultThemeSettings();
    }

    public static bool Save(DefaultThemeSettings settings)
    {
        try
        {
            var configDir = StorageConfig.ConfigPath;
            if (!Directory.Exists(configDir))
                Directory.CreateDirectory(configDir);

            var json = JsonSerializer.Serialize(settings, DefaultThemeSettingsJsonContext.Default.DefaultThemeSettings);
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

[JsonSerializable(typeof(DefaultThemeSettings))]
[JsonSourceGenerationOptions(
    WriteIndented = true,
    PropertyNameCaseInsensitive = true,
    ReadCommentHandling = JsonCommentHandling.Skip,
    AllowTrailingCommas = true)]
internal sealed partial class DefaultThemeSettingsJsonContext : JsonSerializerContext
{
}
