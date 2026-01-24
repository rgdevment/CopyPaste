using Microsoft.UI.Xaml;

namespace CopyPaste.UI;

public partial class App : Application
{
    private Window? _window;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Entry point for UI initialization
        _window = new MainWindow();
        _window.Activate();
    }
}
