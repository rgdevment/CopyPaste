namespace CopyPaste.Core.Themes;

/// <summary>
/// Everything the Host provides to a theme.
/// The theme uses these to interact with the clipboard engine and host-level windows.
/// </summary>
public sealed class ThemeContext(
    IClipboardService service,
    MyMConfig config,
    Action openSettings,
    Action openHelp,
    Action requestExit)
{
    /// <summary>Clipboard service for data access, events, and mutations.</summary>
    public IClipboardService Service { get; } = service;

    /// <summary>
    /// User configuration. The theme MUST respect these values.
    /// See <see cref="ITheme"/> documentation for the full list of required config properties.
    /// </summary>
    public MyMConfig Config { get; } = config;

    /// <summary>Opens the ConfigWindow (host-owned, theme just calls this).</summary>
    public Action OpenSettings { get; } = openSettings;

    /// <summary>Opens the HelpWindow (host-owned, theme just calls this).</summary>
    public Action OpenHelp { get; } = openHelp;

    /// <summary>Requests the Host to begin app shutdown.</summary>
    public Action RequestExit { get; } = requestExit;
}
