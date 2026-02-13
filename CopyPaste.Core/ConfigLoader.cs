using System.Text.Json;

namespace CopyPaste.Core;

/// <summary>
/// Configuration loader for MyM.json - the unified configuration file.
/// MyMConfig is the SINGLE source of truth for all settings.
/// </summary>
public static class ConfigLoader
{
    private const string _configFileName = "MyM.json";
    private static MyMConfig? _cachedConfig;

    /// <summary>
    /// Gets the current configuration. Loads from MyM.json if not cached.
    /// This is the SINGLE source of truth - use this everywhere.
    /// </summary>
    public static MyMConfig Config => _cachedConfig ??= Load();

    /// <summary>
    /// Gets the path to the MyM.json configuration file.
    /// </summary>
    public static string ConfigFilePath => Path.Combine(StorageConfig.ConfigPath, _configFileName);

    /// <summary>
    /// Checks if MyM.json exists.
    /// </summary>
    public static bool ConfigFileExists => File.Exists(ConfigFilePath);

    /// <summary>
    /// Loads configuration from MyM.json. Missing properties use MyMConfig defaults.
    /// </summary>
    public static MyMConfig Load()
    {
        if (_cachedConfig != null)
            return _cachedConfig;

        if (!ConfigFileExists)
        {
            _cachedConfig = new MyMConfig();
            return _cachedConfig;
        }

        try
        {
            var jsonContent = File.ReadAllText(ConfigFilePath);
            var config = JsonSerializer.Deserialize(jsonContent, MyMConfigJsonContext.Default.MyMConfig);

            if (config != null)
            {
                AppLogger.Info($"Loaded configuration from {_configFileName}");
                _cachedConfig = config;
                return config;
            }
        }
        catch (Exception ex)
        {
            AppLogger.Error($"Failed to load {_configFileName}: {ex.Message}");
        }

        _cachedConfig = new MyMConfig();
        return _cachedConfig;
    }

    /// <summary>
    /// Saves configuration to MyM.json.
    /// </summary>
    public static bool Save(MyMConfig config)
    {
        try
        {
            var configDir = StorageConfig.ConfigPath;

            if (!Directory.Exists(configDir))
            {
                Directory.CreateDirectory(configDir);
                AppLogger.Info($"Created config directory: {configDir}");
            }

            var jsonContent = JsonSerializer.Serialize(config, MyMConfigJsonContext.Default.MyMConfig);
            File.WriteAllText(ConfigFilePath, jsonContent);
            _cachedConfig = config;
            AppLogger.Info($"Saved configuration to {ConfigFilePath}");
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Error($"Failed to save config: {ex.Message}");
            return false;
        }
    }

    /// <summary>
    /// Clears the cached configuration, forcing a reload on next access.
    /// </summary>
    public static void ClearCache() => _cachedConfig = null;

    /// <summary>
    /// Gets the custom label for a color, or null if not configured.
    /// UI should call this and fallback to localization if null.
    /// </summary>
    /// <param name="colorName">The color name: "Red", "Green", "Purple", "Yellow", "Blue", "Orange"</param>
    /// <returns>Custom label if configured, otherwise null.</returns>
    public static string? GetColorLabel(string colorName)
    {
        var labels = Config.ColorLabels;
        if (labels != null && labels.TryGetValue(colorName, out var customLabel) && !string.IsNullOrWhiteSpace(customLabel))
            return customLabel;
        return null;
    }

    /// <summary>
    /// Gets the custom label for a CardColor enum value, or null if not configured.
    /// </summary>
    public static string? GetColorLabel(CardColor color) =>
        color == CardColor.None ? null : GetColorLabel(color.ToString());
}



