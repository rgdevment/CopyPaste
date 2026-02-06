using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using CopyPaste.Core;
using CopyPaste.Core.Themes;

namespace CopyPaste.UI.Themes;

/// <summary>
/// Discovers, registers, and instantiates available themes.
/// Internal themes are registered explicitly; community themes are loaded from DLLs.
/// </summary>
internal sealed class ThemeRegistry
{
    private readonly Dictionary<string, Func<ITheme>> _factories = new(StringComparer.OrdinalIgnoreCase);
    private readonly List<ThemeInfo> _themes = [];

    /// <summary>All discovered themes (internal + community).</summary>
    public IReadOnlyList<ThemeInfo> AvailableThemes => _themes;

    /// <summary>
    /// Registers a built-in (internal) theme.
    /// </summary>
    public void RegisterInternal<T>() where T : ITheme, new()
    {
        using var probe = new T();
        var info = new ThemeInfo(probe.Id, probe.Name, probe.Version, probe.Author, IsCommunity: false);
        _themes.Add(info);
        _factories[probe.Id] = static () => new T();
    }

    /// <summary>
    /// Scans the themes folder for community DLLs implementing <see cref="ITheme"/>.
    /// Each DLL is loaded, types are inspected, and valid themes are registered.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Community DLL loading must not crash the app")]
    public void DiscoverCommunityThemes()
    {
        var themesDir = StorageConfig.ThemesPath;
        if (!Directory.Exists(themesDir))
        {
            Directory.CreateDirectory(themesDir);
            return;
        }

        foreach (var dll in Directory.GetFiles(themesDir, "*.dll"))
        {
            try
            {
                var assembly = Assembly.LoadFrom(dll);

                foreach (var type in assembly.GetExportedTypes())
                {
                    if (!typeof(ITheme).IsAssignableFrom(type) || type.IsAbstract || type.IsInterface)
                        continue;

                    using var probe = (ITheme)Activator.CreateInstance(type)!;
                    var info = new ThemeInfo(probe.Id, probe.Name, probe.Version, probe.Author, IsCommunity: true);

                    if (_factories.ContainsKey(probe.Id))
                    {
                        AppLogger.Warn($"Skipping duplicate theme ID '{probe.Id}' from {Path.GetFileName(dll)}");
                        continue;
                    }

                    _themes.Add(info);
                    var capturedType = type;
                    _factories[probe.Id] = () => (ITheme)Activator.CreateInstance(capturedType)!;

                    AppLogger.Info($"Loaded community theme: {info.Name} v{info.Version} by {info.Author} [{Path.GetFileName(dll)}]");
                }
            }
            catch (Exception ex)
            {
                AppLogger.Error($"Failed to load theme from {Path.GetFileName(dll)}: {ex.Message}");
            }
        }
    }

    /// <summary>
    /// Creates a theme instance by ID. Falls back to the first registered theme if not found.
    /// </summary>
    public ITheme Create(string themeId)
    {
        if (_factories.TryGetValue(themeId, out var factory))
            return factory();

        AppLogger.Warn($"Theme '{themeId}' not found, falling back to default");
        return _factories.Values.First()();
    }
}
