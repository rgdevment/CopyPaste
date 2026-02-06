namespace CopyPaste.Core;

public interface IClipboardListener : IDisposable
{
    void Run();
    void Shutdown();
}
