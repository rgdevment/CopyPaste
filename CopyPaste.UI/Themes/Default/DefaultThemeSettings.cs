using CopyPaste.Core;
using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace CopyPaste.UI.Themes;

/// <summary>
/// Settings exclusive to the DefaultTheme.
/// Persisted in DefaultTheme.json alongside MyM.json.
/// Other themes define their own settings class.
/// </summary>
public sealed class DefaultThemeSettings
{
    private const string _fileName = "DefaultTheme.json";

    // ═══════════════════════════════════════════════════════════════
    // Geometry (sidebar layout)
    // ═══════════════════════════════════════════════════════════════

    /// <summary>Width of the sidebar window in pixels.</summary>
    public int WindowWidth { get; set; } = 400;

    /// <summary>Vertical margin from top of work area in pixels.</summary>
    public int WindowMarginTop { get; set; } = 8;

    /// <summary>Vertical margin from bottom of work area in pixels.</summary>
    public int WindowMarginBottom { get; set; } = 16;

    // ═══════════════════════════════════════════════════════════════
    // Card display
    // ═══════════════════════════════════════════════════════════════

    /// <summary>Number of text lines shown when card is collapsed.</summary>
    public int CardMinLines { get; set; } = 3;

    /// <summary>Number of text lines shown when card is expanded.</summary>
    public int CardMaxLines { get; set; } = 9;

    // ═══════════════════════════════════════════════════════════════
    // Show behavior (reset flags)
    // ═══════════════════════════════════════════════════════════════

    /// <summary>Whether to reset scroll position to top when window is shown.</summary>
    public bool ResetScrollOnShow { get; set; } = true;

    /// <summary>Return to Content filter mode when window is shown.</summary>
    public bool ResetFilterModeOnShow { get; set; } = true;

    /// <summary>Clear text search filter when window is shown.</summary>
    public bool ResetContentFilterOnShow { get; set; } = true;

    /// <summary>Clear category (color) filter when window is shown.</summary>
    public bool ResetCategoryFilterOnShow { get; set; } = true;

    /// <summary>Clear type filter when window is shown.</summary>
    public bool ResetTypeFilterOnShow { get; set; } = true;

    // ═══════════════════════════════════════════════════════════════
    // Persistence
    // ═══════════════════════════════════════════════════════════════

    private static string FilePath => Path.Combine(StorageConfig.ConfigPath, _fileName);

    /// <summary>
    /// Loads settings from DefaultTheme.json. Returns defaults if file doesn't exist.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Settings loading should not crash app")]
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
            AppLogger.Error($"Failed to load {_fileName}: {ex.Message}");
        }

        return new DefaultThemeSettings();
    }

    /// <summary>
    /// Saves settings to DefaultTheme.json.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Settings saving should not crash app")]
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
            AppLogger.Error($"Failed to save theme settings: {ex.Message}");
            return false;
        }
    }
}

/// <summary>
/// JSON serialization context for DefaultThemeSettings (required for trimming/AOT).
/// </summary>
[JsonSerializable(typeof(DefaultThemeSettings))]
[JsonSourceGenerationOptions(
    WriteIndented = true,
    PropertyNameCaseInsensitive = true,
    ReadCommentHandling = JsonCommentHandling.Skip,
    AllowTrailingCommas = true)]
internal sealed partial class DefaultThemeSettingsJsonContext : JsonSerializerContext
{
}
