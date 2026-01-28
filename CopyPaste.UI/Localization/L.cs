using System;

namespace CopyPaste.UI.Localization;

public static class L
{
    private static LocalizationService? _instance;

    public static void Initialize(LocalizationService service) =>
        _instance = service ?? throw new ArgumentNullException(nameof(service));

    public static string Get(string key, string? defaultValue = null) =>
        _instance?.Get(key, defaultValue) ?? $"[{key}]";

    public static string CurrentLanguage => _instance?.CurrentLanguage ?? "en-US";

    internal static void Dispose() => _instance?.Dispose();
}
