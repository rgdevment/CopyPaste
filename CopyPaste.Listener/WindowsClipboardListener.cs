using CopyPaste.Core;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

namespace CopyPaste.Listener;

public sealed partial class WindowsClipboardListener(ClipboardService service)
{
    private const uint _cF_UNICODETEXT = 13;
    private const uint _cF_HDROP = 15;
    private const uint _cF_DIB = 8;
    private static readonly uint _cF_HTML = RegisterClipboardFormatW("HTML Format");

    // Window handle to associate with clipboard operations
    private IntPtr _hwnd;

    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool AddClipboardFormatListener(IntPtr hwnd);

    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr CreateWindowExW(
        uint dwExStyle,
        string lpClassName,
        string lpWindowName,
        uint dwStyle,
        int x, int y,
        int nWidth, int nHeight,
        IntPtr hWndParent,
        IntPtr hMenu,
        IntPtr hInstance,
        IntPtr lpParam);

    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial uint RegisterClipboardFormatW(string lpszFormat);

    [LibraryImport("shell32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial uint DragQueryFileW(IntPtr hDrop, uint iFile, [Out] char[]? lpszFile, uint cch);

    [LibraryImport("shell32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial void DragFinish(IntPtr hDrop);

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

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool TranslateMessage(ref MSG lpMsg);

    [LibraryImport("user32.dll", EntryPoint = "GetMessageW")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GetMessage(out MSG lpMsg, IntPtr hWnd, uint wMsgFilterMin, uint wMsgFilterMax);

    [LibraryImport("user32.dll", EntryPoint = "DispatchMessageW")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr DispatchMessage(ref MSG lpMsg);

    public void Run()
    {
        // Store window handle for future clipboard operations
        _hwnd = CreateWindowExW(0, "Static", "CopyPasteListenerHost", 0, 0, 0, 0, 0, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);

        if (_hwnd == IntPtr.Zero) return;

        if (AddClipboardFormatListener(_hwnd))
        {
            while (GetMessage(out var msg, IntPtr.Zero, 0, 0))
            {
                if (msg.message == 0x031D) // WM_CLIPBOARDUPDATE
                {
                    OnClipboardChanged();
                }
                TranslateMessage(ref msg);
                DispatchMessage(ref msg);
            }
        }
    }

    private void OnClipboardChanged()
    {
        if (!OpenClipboard(_hwnd)) return;

        ClipboardContentType type = ClipboardContentType.Text;
        byte[]? rawBytes = null;
        string? plainText = null;
        List<string>? filePaths = null;

        try
        {
            if (IsClipboardFormatAvailable(_cF_HDROP))
            {
                filePaths = ExtractRawFilePaths();
                type = ClipboardContentType.File;
            }
            else if (IsClipboardFormatAvailable(_cF_DIB))
            {
                rawBytes = ExtractRawBytes(_cF_DIB);
                type = ClipboardContentType.Image;
            }
            else if (IsClipboardFormatAvailable(_cF_HTML))
            {
                rawBytes = ExtractRawBytes(_cF_HTML);
                type = ClipboardContentType.Html;
            }
            else if (IsClipboardFormatAvailable(_cF_UNICODETEXT))
            {
                plainText = ExtractRawText();
                type = ClipboardContentType.Text;
            }
        }
        finally
        {
            CloseClipboard();
        }

        if (type == ClipboardContentType.Image && rawBytes == null) return;

        NotifyService(type, plainText, rawBytes, filePaths);
    }

    private static string? ExtractRawText()
    {
        IntPtr hData = GetClipboardData(_cF_UNICODETEXT);
        if (hData == IntPtr.Zero) return null;

        IntPtr pData = GlobalLock(hData);
        try
        {
            return Marshal.PtrToStringUni(pData);
        }
        finally { GlobalUnlock(hData); }
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

    private void NotifyService(ClipboardContentType type, string? text, byte[]? bytes, List<string>? files)
    {
        if (type == ClipboardContentType.Text && !string.IsNullOrWhiteSpace(text))
        {
            service.AddItem(new ClipboardItem { Content = text, Type = type });
        }
        else if (type == ClipboardContentType.Html && bytes != null)
        {
            service.AddItem(new ClipboardItem { Content = Encoding.UTF8.GetString(bytes).TrimEnd('\0'), Type = type });
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

    private async Task ProcessDbQueueAsync()
    {
        await foreach (var item in _dbQueue.Reader.ReadAllAsync())
        {
            try
            {
                service.AddItem(item);
            }
            catch (IOException) // DB busy
            {
                // Re-queue or retry after delay
                await Task.Delay(100);
                // logic to handle retry...
            }
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct MSG
    {
        public IntPtr hwnd;
        public uint message;
        public IntPtr wParam;
        public IntPtr lParam;
        public uint time;
        public System.Drawing.Point pt;
    }
}
