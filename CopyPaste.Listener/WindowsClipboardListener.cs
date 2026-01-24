using CopyPaste.Core;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
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
    private sealed record ClipboardTask(ClipboardContentType Type, string? Text, byte[]? Bytes, List<string>? Files);
    private readonly CancellationTokenSource _cts = new();

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
                bool success = false;
                while (!success && !_cts.IsCancellationRequested)
                {
                    try
                    {
                        NotifyService(task.Type, task.Text, task.Bytes, task.Files);
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
        }
        catch (OperationCanceledException)
        {
            Debug.WriteLine("Queue processing stopped gracefully.");
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
                var files = ExtractRawFilePaths();
                _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.File, null, null, files));
            }
            else if (IsClipboardFormatAvailable(_cF_DIB))
            {
                var bytes = ExtractRawBytes(_cF_DIB);
                if (bytes != null)
                {
                    var bmp = FixDibHeader(bytes);
                    _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.Image, null, bmp, null));
                }
            }
            else if (IsClipboardFormatAvailable(_cF_HTML))
            {
                var bytes = ExtractRawBytes(_cF_HTML);
                _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.Html, null, bytes, null));
            }
            else if (IsClipboardFormatAvailable(_cF_UNICODETEXT))
            {
                var text = ExtractRawText();
                _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.Text, text, null, null));
            }
        }
        finally { CloseClipboard(); }
    }

    private static byte[]? ExtractRawBytes(uint format)
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

    private static string? ExtractRawText()
    {
        IntPtr hData = GetClipboardData(_cF_UNICODETEXT);
        if (hData == IntPtr.Zero) return null;
        IntPtr pData = GlobalLock(hData);
        try { return Marshal.PtrToStringUni(pData); }
        finally { GlobalUnlock(hData); }
    }

    private static List<string> ExtractRawFilePaths()
    {
        IntPtr hData = GetClipboardData(_cF_HDROP);
        List<string> files = [];
        if (hData == IntPtr.Zero) return files;

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
        return files;
    }

    private static byte[]? FixDibHeader(byte[] dibData)
    {
        if (dibData.Length < 40) return null;
        int headerSize = BitConverter.ToInt32(dibData, 0);
        int bitCount = BitConverter.ToInt16(dibData, 14);
        int compression = BitConverter.ToInt32(dibData, 16);
        int colorsUsed = BitConverter.ToInt32(dibData, 32);

        int paletteSize = 0;
        if (headerSize == 40 && compression == 3) paletteSize = 12;
        else if (colorsUsed > 0) paletteSize = colorsUsed * 4;
        else if (bitCount <= 8) paletteSize = (1 << bitCount) * 4;

        int pixelOffset = 14 + headerSize + paletteSize;
        int fileSize = 14 + dibData.Length;

        byte[] fileHeader = new byte[14];
        fileHeader[0] = 0x42; fileHeader[1] = 0x4D;
        BitConverter.TryWriteBytes(fileHeader.AsSpan(2), fileSize);
        BitConverter.TryWriteBytes(fileHeader.AsSpan(10), pixelOffset);

        byte[] bmp = new byte[fileSize];
        Buffer.BlockCopy(fileHeader, 0, bmp, 0, 14);
        Buffer.BlockCopy(dibData, 0, bmp, 14, dibData.Length);
        return bmp;
    }

    private static string ExtractHtmlFragment(string rawHtml)
    {
        if (string.IsNullOrWhiteSpace(rawHtml)) return string.Empty;

        // Standard markers for CF_HTML format
        const string startFragment = "";
        const string endFragment = "";

        int startIndex = rawHtml.IndexOf(startFragment, StringComparison.OrdinalIgnoreCase);
        int endIndex = rawHtml.LastIndexOf(endFragment, StringComparison.OrdinalIgnoreCase);

        if (startIndex != -1 && endIndex != -1)
        {
            startIndex += startFragment.Length;
            return rawHtml[startIndex..endIndex].Trim();
        }

        int htmlStart = rawHtml.IndexOf("<html", StringComparison.OrdinalIgnoreCase);
        return htmlStart != -1 ? rawHtml[htmlStart..].Trim() : rawHtml;
    }

    private void NotifyService(ClipboardContentType type, string? text, byte[]? bytes, List<string>? files)
    {
        if (type == ClipboardContentType.Text && !string.IsNullOrWhiteSpace(text))
        {
            service.AddItem(new ClipboardItem { Content = text, Type = type });
        }
        else if (type == ClipboardContentType.Html && bytes != null)
        {
            string rawHtml = Encoding.UTF8.GetString(bytes).TrimEnd('\0');
            string html = ExtractHtmlFragment(rawHtml);
            service.AddItem(new ClipboardItem { Content = html, Type = type });
        }
        else if (type == ClipboardContentType.Image && bytes != null)
        {
            service.AddItem(new ClipboardItem { Type = type }, bytes);
        }
        else if (type == ClipboardContentType.File && files?.Count > 0)
        {
            string paths = string.Join(Environment.NewLine, files);
            var meta = new Dictionary<string, object> { { "file_count", files.Count }, { "first_ext", Path.GetExtension(files[0]) } };
            string json = JsonSerializer.Serialize(meta, MetadataJsonContext.Default.DictionaryStringObject);
            service.AddItem(new ClipboardItem { Content = paths, Type = type, Metadata = json });
        }
    }

    // Explicit disposal of native and managed resources
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
