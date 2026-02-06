namespace CopyPaste.Core;

public sealed class CopyPasteEngine : IDisposable
{
    private readonly SqliteRepository _repository;
    private readonly IClipboardListener _listener;
    private readonly CleanupService _cleanupService;
    private bool _isDisposed;

    public IClipboardService Service { get; }
    public MyMConfig Config { get; }

    public CopyPasteEngine(Func<IClipboardService, IClipboardListener> listenerFactory)
    {
        ArgumentNullException.ThrowIfNull(listenerFactory);

        AppLogger.Initialize();
        AppLogger.Info("CopyPasteEngine starting...");

        var config = ConfigLoader.Config;
        Config = config;

        _repository = new SqliteRepository(StorageConfig.DatabasePath);
        var clipboardService = new ClipboardService(_repository);
        clipboardService.PasteIgnoreWindowMs = config.DuplicateIgnoreWindowMs;

        Service = clipboardService;
        _listener = listenerFactory(Service);
        _cleanupService = new CleanupService(_repository, () => config.RetentionDays);

        AppLogger.Info("CopyPasteEngine initialized");
    }

    public void Start()
    {
        Task.Run(() => _listener.Run());
        AppLogger.Info("Clipboard listener started");
    }

    public void Dispose()
    {
        if (_isDisposed) return;

        try
        {
            _listener.Dispose();
            _cleanupService.Dispose();
            _repository.Dispose();
        }
        catch (ObjectDisposedException)
        {
        }

        _isDisposed = true;
    }
}
