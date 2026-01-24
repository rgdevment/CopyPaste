using CopyPaste.Core;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading.Channels;

namespace CopyPaste.Listener;

public sealed partial class WindowsClipboardListener(ClipboardService service) : IDisposable
{
    private const uint _cF_UNICODETEXT = 13;
    private const uint _cF_HDROP = 15;
    private const uint _cF_DIB = 8;
    private static readonly uint _cF_HTML = RegisterClipboardFormatW("HTML Format");

    private IntPtr _hwnd;
    private bool _disposed;

    private readonly Channel<ClipboardTask> _taskQueue = Channel.CreateUnbounded<ClipboardTask>();
    private readonly CancellationTokenSource _cts = new();

    private sealed record ClipboardTask(ClipboardContentType Type, string? Text, byte[]? Bytes, Collection<string>? Files);

    #region Win32 API
    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr CreateWindowExW(uint dwExStyle, string lpClassName, string lpWindowName, uint dwStyle, int x, int y, int nWidth, int nHeight, IntPtr hWndParent, IntPtr hMenu, IntPtr hInstance, IntPtr lpParam);

    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial uint RegisterClipboardFormatW(string lpszFormat);

    [LibraryImport("user32.dll", EntryPoint = "GetMessageW")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GetMessage(out MSG lpMsg, IntPtr hWnd, uint wMsgFilterMin, uint wMsgFilterMax);

    [LibraryImport("user32.dll", EntryPoint = "DispatchMessageW")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr DispatchMessage(ref MSG lpMsg);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool TranslateMessage(ref MSG lpMsg);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool AddClipboardFormatListener(IntPtr hwnd);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool OpenClipboard(IntPtr hWndNewOwner);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool CloseClipboard();

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr GetClipboardData(uint uFormat);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool IsClipboardFormatAvailable(uint format);

    [LibraryImport("kernel32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr GlobalLock(IntPtr hMem);

    [LibraryImport("kernel32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GlobalUnlock(IntPtr hMem);

    [LibraryImport("kernel32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial nuint GlobalSize(IntPtr hMem);

    [LibraryImport("shell32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial uint DragQueryFileW(IntPtr hDrop, uint iFile, [Out] char[]? lpszFile, uint cch);
    #endregion

    public void Stop()
    {
        _cts.Cancel();
        _taskQueue.Writer.Complete();
    }

    public void Run()
    {
        _hwnd = CreateWindowExW(0, "Static", "CopyPasteHost", 0, 0, 0, 0, 0, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
        if (_hwnd == IntPtr.Zero) return;

        if (AddClipboardFormatListener(_hwnd))
        {
            _ = Task.Run(ProcessQueueAsync);

            while (GetMessage(out var msg, IntPtr.Zero, 0, 0))
            {
                if (msg.message == 0x031D) OnClipboardChanged();
                TranslateMessage(ref msg);
                DispatchMessage(ref msg);
            }
        }
    }

    private async Task ProcessQueueAsync()
    {
        try
        {
            await foreach (var task in _taskQueue.Reader.ReadAllAsync(_cts.Token).ConfigureAwait(false))
            {
                await ProcessTaskWithRetryAsync(task).ConfigureAwait(false);
            }
        }
        catch (OperationCanceledException)
        {
            Debug.WriteLine("Queue processing stopped gracefully.");
        }
    }

    private async Task ProcessTaskWithRetryAsync(ClipboardTask task)
    {
        bool success = false;
        while (!success && !_cts.IsCancellationRequested)
        {
            try
            {
                DispatchToService(task);
                success = true;
            }
            catch (IOException)
            {
                await Task.Delay(500, _cts.Token).ConfigureAwait(false);
            }
            catch (Exception ex) when (ex is not OperationCanceledException)
            {
                Debug.WriteLine($"Critical error: {ex.Message}");
                success = true;
            }
        }
    }

    private void DispatchToService(ClipboardTask task)
    {
        switch (task.Type)
        {
            case ClipboardContentType.Text:
                service.AddText(task.Text);
                break;
            case ClipboardContentType.Html:
                service.AddHtml(task.Bytes);
                break;
            case ClipboardContentType.Image:
                service.AddImage(task.Bytes);
                break;
            case ClipboardContentType.File:
                service.AddFiles(task.Files);
                break;
        }
    }

    private void OnClipboardChanged()
    {
        Thread.Sleep(50);

        if (!OpenClipboard(_hwnd)) return;

        try
        {
            if (IsClipboardFormatAvailable(_cF_HDROP))
            {
                var files = ExtractFilePaths();
                _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.File, null, null, files));
            }
            else if (IsClipboardFormatAvailable(_cF_DIB))
            {
                var bytes = ExtractBytes(_cF_DIB);
                if (bytes != null)
                {
                    _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.Image, null, bytes, null));
                }
            }
            else if (IsClipboardFormatAvailable(_cF_HTML))
            {
                var bytes = ExtractBytes(_cF_HTML);
                _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.Html, null, bytes, null));
            }
            else if (IsClipboardFormatAvailable(_cF_UNICODETEXT))
            {
                var text = ExtractText();
                _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.Text, text, null, null));
            }
        }
        finally { CloseClipboard(); }
    }

    private static byte[]? ExtractBytes(uint format)
    {
        IntPtr hData = GetClipboardData(format);
        if (hData == IntPtr.Zero) return null;
        IntPtr pData = GlobalLock(hData);
        try
        {
            int size = (int)GlobalSize(hData);
            if (size <= 0) return null;
            byte[] buffer = new byte[size];
            Marshal.Copy(pData, buffer, 0, size);
            return buffer;
        }
        finally { GlobalUnlock(hData); }
    }

    private static string? ExtractText()
    {
        IntPtr hData = GetClipboardData(_cF_UNICODETEXT);
        if (hData == IntPtr.Zero) return null;
        IntPtr pData = GlobalLock(hData);
        try { return Marshal.PtrToStringUni(pData); }
        finally { GlobalUnlock(hData); }
    }

    private static Collection<string> ExtractFilePaths()
    {
        IntPtr hData = GetClipboardData(_cF_HDROP);
        List<string> files = [];
        if (hData == IntPtr.Zero) return new Collection<string>(files);

        uint count = DragQueryFileW(hData, 0xFFFFFFFF, null, 0);
        for (uint i = 0; i < count; i++)
        {
            uint length = DragQueryFileW(hData, i, null, 0);
            char[] buffer = new char[length + 1];
            if (DragQueryFileW(hData, i, buffer, (uint)buffer.Length) > 0)
            {
                files.Add(new string(buffer).TrimEnd('\0'));
            }
        }
        return new Collection<string>(files);
    }

    public void Dispose()
    {
        if (_disposed) return;
        _cts.Cancel();
        _cts.Dispose();
        _disposed = true;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct MSG { public IntPtr hwnd; public uint message; public IntPtr wParam; public IntPtr lParam; public uint time; public System.Drawing.Point pt; }
}
