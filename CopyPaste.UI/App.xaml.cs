using CopyPaste.Core;
using CopyPaste.Listener;
using Microsoft.UI.Xaml;
using System;
using System.Threading.Tasks;

namespace CopyPaste.UI;

public sealed partial class App : Application, IDisposable
{
    private Window? _window;
    private readonly WindowsClipboardListener? _listener;
    private readonly ClipboardService? _service;
    private bool _isDisposed;

    public App()
    {
        InitializeComponent();
        StorageConfig.Initialize();

        // Initialize core components once
        var repository = new LiteDbRepository(StorageConfig.DatabasePath);
        _service = new ClipboardService(repository);
        _listener = new WindowsClipboardListener(_service);

        // Run listener in background
        Task.Run(() => _listener.Run());
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _window = new MainWindow(_service!);

        _window.Activate();
    }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    private void Dispose(bool disposing)
    {
        if (_isDisposed) return;
        if (disposing) _listener?.Dispose();
        _isDisposed = true;
    }
}
