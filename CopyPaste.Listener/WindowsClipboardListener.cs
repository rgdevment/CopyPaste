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

        try
        {
            if (IsClipboardFormatAvailable(_cF_HDROP))
            {
                ExtractFilePaths();
            }
            if (IsClipboardFormatAvailable(_cF_DIB))
            {
                ExtractImage();
            }
            if (IsClipboardFormatAvailable(_cF_HTML))
            {
                ExtractHtml();
            }
            else if (IsClipboardFormatAvailable(_cF_UNICODETEXT))
            {
                ExtractText();
            }
        }
        finally
        {
            CloseClipboard();
        }
    }

    private void ExtractText()
    {
        IntPtr hData = GetClipboardData(_cF_UNICODETEXT);
        if (hData == IntPtr.Zero) return;

        string? text = Marshal.PtrToStringUni(hData);
        if (!string.IsNullOrWhiteSpace(text))
        {
            service.AddItem(new ClipboardItem
            {
                Content = text,
                Type = ClipboardContentType.Text
            });
        }
    }

    private void ExtractImage()
    {
        IntPtr hData = GetClipboardData(_cF_DIB);
        if (hData == IntPtr.Zero) return;

        IntPtr pData = GlobalLock(hData);
        try
        {
            int size = (int)GlobalSize(hData);
            byte[] buffer = new byte[size];
            Marshal.Copy(pData, buffer, 0, size);

            service.AddItem(new ClipboardItem
            {
                Type = ClipboardContentType.Image
            }, buffer);
        }
        finally
        {
            GlobalUnlock(hData);
        }
    }

    private void ExtractHtml()
    {
        IntPtr hData = GetClipboardData(_cF_HTML);
        if (hData == IntPtr.Zero) return;

        IntPtr pData = GlobalLock(hData);
        try
        {
            nuint size = GlobalSize(hData);
            byte[] buffer = new byte[(int)size];
            Marshal.Copy(pData, buffer, 0, buffer.Length);

            // HTML Format is strictly UTF-8 encoded
            string rawHtml = Encoding.UTF8.GetString(buffer).TrimEnd('\0');

            if (!string.IsNullOrWhiteSpace(rawHtml))
            {
                service.AddItem(new ClipboardItem
                {
                    Content = rawHtml,
                    Type = ClipboardContentType.Html
                });
            }
        }
        finally
        {
            GlobalUnlock(hData);
        }
    }

    private void ExtractFilePaths()
    {
        IntPtr hData = GetClipboardData(_cF_HDROP);
        if (hData == IntPtr.Zero) return;

        uint fileCount = DragQueryFileW(hData, 0xFFFFFFFF, null, 0);
        List<string> fileList = [];

        for (uint i = 0; i < fileCount; i++)
        {
            uint pathLength = DragQueryFileW(hData, i, null, 0);
            char[] pathBuffer = new char[pathLength + 1];

            if (DragQueryFileW(hData, i, pathBuffer, (uint)pathBuffer.Length) > 0)
            {
                fileList.Add(new string(pathBuffer).TrimEnd('\0'));
            }
        }

        if (fileList.Count > 0)
        {
            string allPaths = string.Join(Environment.NewLine, fileList);

            var item = new ClipboardItem
            {
                Content = allPaths,
                Type = ClipboardContentType.File
            };

            var metadata = new Dictionary<string, object>
            {
                { "file_count", fileList.Count },
                { "is_multiple", fileList.Count > 1 },
                { "first_extension", Path.GetExtension(fileList[0]) }
            };

            item.Metadata = JsonSerializer.Serialize(metadata, MetadataJsonContext.Default.DictionaryStringObject);
            service.AddItem(item);
        }

        DragFinish(hData);
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
