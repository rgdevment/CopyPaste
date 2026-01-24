using CopyPaste.Core;
using CopyPaste.Listener;
using Microsoft.UI.Xaml;
using System.Threading.Tasks;

namespace CopyPaste.UI;

public partial class App : Application
{
    private Window? _window;

    public App()
    {
        InitializeComponent();

        StorageConfig.Initialize();

        var repository = new LiteDbRepository(StorageConfig.DatabasePath);
        var service = new ClipboardService(repository);
        var listener = new WindowsClipboardListener(service);

        Task.Run(() => listener.Run());
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Entry point for UI initialization
        _window = new MainWindow();
        _window.Activate();
    }
}
