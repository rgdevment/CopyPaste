namespace CopyPaste.Core;

/// <summary>
/// Manages application startup registration across packaging modes.
/// <list type="bullet">
///   <item><description>Packaged (MSIX/Store): Uses <c>Windows.ApplicationModel.StartupTask</c> API.</description></item>
///   <item><description>Unpackaged (standalone): Uses <c>HKCU\Software\Microsoft\Windows\CurrentVersion\Run</c> registry key.</description></item>
/// </list>
/// </summary>
public static class StartupHelper
{
    private const string _registryKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string _appRegistryName = "CopyPaste";

    /// <summary>
    /// The StartupTask ID declared in Package.appxmanifest for packaged builds.
    /// Must match the <c>TaskId</c> attribute in the <c>uap5:StartupTask</c> extension.
    /// </summary>
    internal const string StartupTaskId = "CopyPasteStartupTask";

    /// <summary>
    /// Applies the startup setting based on the current packaging mode.
    /// Enables or disables automatic launch at Windows startup.
    /// All exceptions are handled internally - this method never throws.
    /// </summary>
    /// <param name="runOnStartup">True to enable auto-start, false to disable.</param>
    public static async Task ApplyStartupSettingAsync(bool runOnStartup)
    {
        try
        {
            if (PackageHelper.IsPackaged)
            {
                await ApplyPackagedStartupAsync(runOnStartup).ConfigureAwait(false);
            }
            else
            {
                ApplyUnpackagedStartup(runOnStartup);
            }
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to apply startup setting");
        }
    }

    private static async Task ApplyPackagedStartupAsync(bool enable)
    {
        try
        {
            var startupTask = await Windows.ApplicationModel.StartupTask.GetAsync(StartupTaskId);

            if (enable)
            {
                switch (startupTask.State)
                {
                    case Windows.ApplicationModel.StartupTaskState.Disabled:
                        var newState = await startupTask.RequestEnableAsync();
                        AppLogger.Info($"Startup task enable requested, new state: {newState}");
                        break;

                    case Windows.ApplicationModel.StartupTaskState.DisabledByUser:
                        // User disabled via Task Manager - cannot re-enable programmatically.
                        // User must re-enable via Task Manager > Startup Apps.
                        AppLogger.Info("Startup task disabled by user via Task Manager - cannot enable programmatically");
                        break;

                    case Windows.ApplicationModel.StartupTaskState.Enabled:
                        AppLogger.Info("Startup task already enabled");
                        break;
                }
            }
            else
            {
                if (startupTask.State == Windows.ApplicationModel.StartupTaskState.Enabled)
                {
                    startupTask.Disable();
                    AppLogger.Info("Startup task disabled");
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to manage packaged startup task");
        }
    }

    /// <summary>
    /// Applies startup setting using the Windows Registry (unpackaged/standalone mode).
    /// When enabling, registers the launcher (or app executable) in HKCU\...\Run.
    /// When disabling, removes the registry entry.
    /// </summary>
    internal static void ApplyUnpackagedStartup(bool enable)
    {
        using var key = Microsoft.Win32.Registry.CurrentUser.CreateSubKey(_registryKeyPath, true);
        if (key == null) return;

        if (enable)
        {
            // Only register if not already registered
            if (key.GetValue(_appRegistryName) == null)
            {
                var startupPath = GetStartupPath();
                if (startupPath != null)
                {
                    key.SetValue(_appRegistryName, $"\"{startupPath}\"");
                    AppLogger.Info($"Registered for Windows startup: {startupPath}");
                }
            }
        }
        else
        {
            // Remove registry entry to disable startup
            if (key.GetValue(_appRegistryName) != null)
            {
                key.DeleteValue(_appRegistryName, throwOnMissingValue: false);
                AppLogger.Info("Unregistered from Windows startup");
            }
        }
    }

    /// <summary>
    /// Gets the startup executable path for registry registration.
    /// Prefers the native launcher (CopyPaste.exe) if present, otherwise uses the .NET app executable.
    /// </summary>
    internal static string? GetStartupPath()
    {
        var appExePath = Environment.ProcessPath;
        if (string.IsNullOrEmpty(appExePath)) return null;

        var appDir = Path.GetDirectoryName(appExePath)!;
        var launcherPath = Path.Combine(appDir, "CopyPaste.exe");
        return File.Exists(launcherPath) ? launcherPath : appExePath;
    }
}
