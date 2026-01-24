using CopyPaste.Core;
using CopyPaste.Listener;
using Microsoft.UI.Xaml;
using System;
using System.Threading.Tasks;

namespace CopyPaste.UI;

public sealed partial class App : Application, IDisposable
{
    private Window? _window;
    private WindowsClipboardListener? _listener;
    private bool _isDisposed;

    public App()
    {
        InitializeComponent();

        StorageConfig.Initialize();

        var repository = new LiteDbRepository(StorageConfig.DatabasePath);
        var service = new ClipboardService(repository);

        _listener = new WindowsClipboardListener(service);

        Task.Run(() => _listener.Run());
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _window = new MainWindow();
        _window.Activate();

        AppDomain.CurrentDomain.ProcessExit += (s, e) => Dispose();
    }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    private void Dispose(bool disposing)
    {
        if (_isDisposed) return;

        if (disposing)
        {
            _listener?.Dispose();
        }

        _isDisposed = true;
    }
}
