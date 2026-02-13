namespace CopyPaste.Core;

/// <summary>
/// Provides runtime detection of application packaging mode (MSIX packaged vs standalone).
/// Used to branch behavior for startup registration, update checking, and other
/// platform-specific operations that differ between Store and standalone distributions.
/// </summary>
public static class PackageHelper
{
    private static readonly Lazy<bool> _isPackaged = new(DetectIsPackaged);

    /// <summary>
    /// Gets whether the application is running as an MSIX packaged app
    /// (e.g., installed from Microsoft Store or sideloaded MSIX).
    /// When true, Windows manages storage virtualization, startup tasks, and updates.
    /// When false, the app runs as an unpackaged standalone installation.
    /// </summary>
    public static bool IsPackaged => _isPackaged.Value;

    private static bool DetectIsPackaged()
    {
        try
        {
            // Windows.ApplicationModel.Package.Current throws InvalidOperationException
            // when the app is not running in a packaged context (standalone/unpackaged).
            // This is the documented detection pattern for MSIX packaging.
            return Windows.ApplicationModel.Package.Current?.Id != null;
        }
        catch
        {
            return false;
        }
    }
}
