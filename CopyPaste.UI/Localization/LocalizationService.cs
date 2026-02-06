using CopyPaste.Core;
using System;
using System.Collections.Frozen;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text.Json;

namespace CopyPaste.UI.Localization;

public sealed class LocalizationService : IDisposable
{
    private static readonly string[] _availableLanguages = ["en-US", "es-CL"];
    private readonly FrozenDictionary<string, string> _strings;
    private bool _isDisposed;

    public string CurrentLanguage { get; }

    public LocalizationService(string? preferredLanguage = null)
    {
        var osCulture = CultureInfo.CurrentCulture.Name;
        AppLogger.Info($"Localization init - OS culture: {osCulture}, Preferred: {preferredLanguage ?? "auto"}", "LocalizationService");

        CurrentLanguage = ResolveLanguage(preferredLanguage, osCulture);
        _strings = BuildAndFreeze(CurrentLanguage);

        AppLogger.Info($"Localization ready - Resolved language: {CurrentLanguage}, Keys loaded: {_strings.Count}", "LocalizationService");
    }

    public string Get(string key, string? defaultValue = null)
    {
        ObjectDisposedException.ThrowIf(_isDisposed, this);
        if (string.IsNullOrEmpty(key)) return defaultValue ?? string.Empty;
        return _strings.TryGetValue(key, out var value) ? value : defaultValue ?? $"[{key}]";
    }

    private static string ResolveLanguage(string? preferred, string osCulture)
    {
        if (!string.IsNullOrEmpty(preferred) && !string.Equals(preferred, "auto", StringComparison.OrdinalIgnoreCase))
        {
            if (LanguageExists(preferred))
            {
                AppLogger.Info($"Resolution: Manual preference '{preferred}' found directly", "LocalizationService");
                return preferred;
            }

            var fallback = GetRegionalFallback(preferred);
            if (fallback != null && LanguageExists(fallback))
            {
                AppLogger.Info($"Resolution: Manual '{preferred}' → regional fallback '{fallback}'", "LocalizationService");
                return fallback;
            }

            AppLogger.Warn($"Resolution: Manual preference '{preferred}' not found, falling back to OS detection", "LocalizationService");
        }

        if (LanguageExists(osCulture))
        {
            AppLogger.Info($"Resolution: OS culture '{osCulture}' found directly", "LocalizationService");
            return osCulture;
        }

        var regionalFallback = GetRegionalFallback(osCulture);
        if (regionalFallback != null && LanguageExists(regionalFallback))
        {
            AppLogger.Info($"Resolution: OS '{osCulture}' → regional fallback '{regionalFallback}'", "LocalizationService");
            return regionalFallback;
        }

        var globalFallback = GetGlobalFallback();
        AppLogger.Info($"Resolution: No match for OS '{osCulture}' → global fallback '{globalFallback}'", "LocalizationService");
        return globalFallback;
    }

    private static string? GetRegionalFallback(string languageTag)
    {
        var baseLang = languageTag.Split('-')[0];
        var config = LoadConfig();
        return config.TryGetValue($"regional.{baseLang}", out var fallback) ? fallback : null;
    }

    private static string GetGlobalFallback()
    {
        var config = LoadConfig();
        return config.TryGetValue("globalFallback", out var fallback) ? fallback : "en-US";
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Config loading is best-effort - failures are logged and defaults returned")]
    private static Dictionary<string, string> LoadConfig()
    {
        var result = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        try
        {
            var json = LoadEmbeddedResource("language-config.json");
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            if (root.TryGetProperty("globalFallback", out var globalFallbackValue))
                result["globalFallback"] = globalFallbackValue.GetString() ?? "en-US";

            if (root.TryGetProperty("regionalFallbacks", out var regionalFallbacks))
            {
                foreach (var prop in regionalFallbacks.EnumerateObject())
                    result[$"regional.{prop.Name}"] = prop.Value.GetString() ?? "en-US";
            }

            AppLogger.Info($"Config loaded - globalFallback: {result.GetValueOrDefault("globalFallback", "en-US")}, regional mappings: {result.Count - 1}", "LocalizationService");
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to load language-config.json - Using default fallback 'en-US'");
            result["globalFallback"] = "en-US";
        }
        return result;
    }

    private static FrozenDictionary<string, string> BuildAndFreeze(string language)
    {
        var merged = new Dictionary<string, string>(200, StringComparer.OrdinalIgnoreCase);
        LoadLanguageInto("en-US", merged);
        if (!string.Equals(language, "en-US", StringComparison.OrdinalIgnoreCase))
            LoadLanguageInto(language, merged);
        return merged.ToFrozenDictionary(StringComparer.OrdinalIgnoreCase);
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Language loading is best-effort - failures are logged and English fallback used")]
    private static void LoadLanguageInto(string lang, Dictionary<string, string> target)
    {
        try
        {
            var json = LoadEmbeddedResource($"Languages.{lang}.json");
            using var doc = JsonDocument.Parse(json);
            var countBefore = target.Count;
            FlattenJson(doc.RootElement, target, "");
            AppLogger.Info($"Loaded language '{lang}' - {target.Count - countBefore} new keys, {target.Count} total", "LocalizationService");
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, $"Failed to load language file '{lang}.json'");
        }
    }

    private static bool LanguageExists(string lang)
    {
        foreach (var available in _availableLanguages)
            if (string.Equals(available, lang, StringComparison.OrdinalIgnoreCase)) return true;
        return false;
    }

    private static string LoadEmbeddedResource(string path)
    {
        var assembly = typeof(LocalizationService).Assembly;
        using var stream = assembly.GetManifestResourceStream($"CopyPaste.UI.Localization.{path}")
            ?? throw new FileNotFoundException(path);
        using var reader = new StreamReader(stream);
        return reader.ReadToEnd();
    }

    private static void FlattenJson(JsonElement element, Dictionary<string, string> dictionary, string prefix)
    {
        if (element.ValueKind == JsonValueKind.Object)
        {
            foreach (var prop in element.EnumerateObject())
            {
                var key = string.IsNullOrEmpty(prefix) ? prop.Name : $"{prefix}.{prop.Name}";
                FlattenJson(prop.Value, dictionary, key);
            }
        }
        else if (element.ValueKind == JsonValueKind.String)
        {
            dictionary[prefix] = element.GetString() ?? "";
        }
    }

    public void Dispose() => _isDisposed = true;
}

