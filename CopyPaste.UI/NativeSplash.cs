using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;

namespace CopyPaste.UI;

/// <summary>
/// Native Win32 splash screen that displays instantly before WinUI initializes.
/// Uses pure Win32 APIs with GDI+ for image loading to avoid framework cold-start delays.
/// </summary>
[System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA2216:Disposable types should declare finalizer", Justification = "No unmanaged resources owned directly")]
[System.Diagnostics.CodeAnalysis.SuppressMessage("Interoperability", "SYSLIB1054:Use LibraryImportAttribute", Justification = "Native splash screen requires DllImport for GDI+")]
public sealed partial class NativeSplash : IDisposable
{
    private const int _width = 320;
    private const int _height = 240;
    private const int _logoSize = 80;

    private nint _hwnd;
    private readonly nint _hInstance;
    private nint _gdiplusToken;
    private static nint _logoImage;
    private bool _disposed;
    private readonly Thread? _messageThread;
    private readonly ManualResetEventSlim _windowCreated = new(false);

    public bool IsVisible => _hwnd != 0;

    public NativeSplash()
    {
        _hInstance = GetModuleHandle(null);

        // Initialize GDI+ and load logo BEFORE creating window for faster display
        var input = new GdiplusStartupInput { GdiplusVersion = 1 };
        _ = GdiplusStartup(out _gdiplusToken, ref input, out _);
        LoadLogoImage();

        // Run message loop on separate thread
        _messageThread = new Thread(CreateAndShowWindow)
        {
            IsBackground = true,
            Name = "SplashThread"
        };
        _messageThread.SetApartmentState(ApartmentState.STA);
        _messageThread.Start();

        // Wait for window to be created (should be very fast now, max 500ms)
        _windowCreated.Wait(500);
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types", Justification = "Logo loading is non-critical")]
    private static void LoadLogoImage()
    {
        try
        {
            var exePath = Environment.ProcessPath;
            if (string.IsNullOrEmpty(exePath)) return;

            var logoPath = Path.Combine(Path.GetDirectoryName(exePath)!, "Assets", "CopyPasteLogo.png");
            if (File.Exists(logoPath))
            {
                _ = GdipLoadImageFromFile(logoPath, out _logoImage);
            }
        }
        catch
        {
            _logoImage = nint.Zero;
        }
    }

    private void CreateAndShowWindow()
    {
        const string className = "CopyPasteSplash";

        // Register window class
        var wc = new WNDCLASSEX
        {
            cbSize = (uint)Marshal.SizeOf<WNDCLASSEX>(),
            style = 0x0003, // CS_HREDRAW | CS_VREDRAW
            lpfnWndProc = Marshal.GetFunctionPointerForDelegate<WndProcDelegate>(WndProc),
            hInstance = _hInstance,
            hCursor = LoadCursor(nint.Zero, 32512), // IDC_ARROW
            hbrBackground = CreateSolidBrush(0x00282828), // Dark background
            lpszClassName = className
        };

        RegisterClassEx(ref wc);

        // Center on screen
        int screenWidth = GetSystemMetrics(0);
        int screenHeight = GetSystemMetrics(1);
        int x = (screenWidth - _width) / 2;
        int y = (screenHeight - _height) / 2;

        // Create popup window
        _hwnd = CreateWindowEx(
            0x00000008, // WS_EX_TOPMOST
            className,
            "CopyPaste",
            0x80000000, // WS_POPUP
            x, y, _width, _height,
            nint.Zero, nint.Zero, _hInstance, nint.Zero);

        if (_hwnd == nint.Zero)
        {
            _windowCreated.Set();
            return;
        }

        // Show window immediately (logo already loaded)
        ShowWindow(_hwnd, 5); // SW_SHOW
        UpdateWindow(_hwnd);
        _windowCreated.Set();

        // Message loop
        while (GetMessage(out var msg, nint.Zero, 0, 0) > 0)
        {
            TranslateMessage(ref msg);
            DispatchMessage(ref msg);
        }
    }

    private static nint WndProc(nint hwnd, uint msg, nint wParam, nint lParam)
    {
        if (msg == 0x000F) // WM_PAINT
        {
            BeginPaint(hwnd, out var ps);
            var hdc = ps.hdc;

            // Draw background
            var rect = new RECT { right = _width, bottom = _height };
            var brush = CreateSolidBrush(0x00282828);
            _ = FillRect(hdc, ref rect, brush);
            DeleteObject(brush);

            // Draw logo if available
            if (_logoImage != nint.Zero)
            {
                _ = GdipCreateFromHDC(hdc, out var graphics);
                if (graphics != nint.Zero)
                {
                    int logoX = (_width - _logoSize) / 2;
                    int logoY = 35;
                    _ = GdipDrawImageRectI(graphics, _logoImage, logoX, logoY, _logoSize, _logoSize);
                    _ = GdipDeleteGraphics(graphics);
                }
            }

            // Configure text
            _ = SetBkMode(hdc, 1); // TRANSPARENT
            _ = SetTextColor(hdc, 0x00FFFFFF); // White

            // Title font
            var hFont = CreateFont(24, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI");
            var oldFont = SelectObject(hdc, hFont);

            var titleRect = new RECT { top = 130, right = _width, bottom = 165 };
            _ = DrawText(hdc, "CopyPaste", -1, ref titleRect, 0x00000001); // DT_CENTER

            SelectObject(hdc, oldFont);
            DeleteObject(hFont);

            // Subtitle font
            hFont = CreateFont(13, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI");
            oldFont = SelectObject(hdc, hFont);
            _ = SetTextColor(hdc, 0x00888888); // Gray

            var subtitleRect = new RECT { top = 170, right = _width, bottom = 210 };
            _ = DrawText(hdc, "Compiling the APP...", -1, ref subtitleRect, 0x00000001);

            SelectObject(hdc, oldFont);
            DeleteObject(hFont);

            // Draw subtle border
            var borderBrush = CreateSolidBrush(0x00404040);
            var borderRect = new RECT { right = _width, bottom = _height };
            _ = FrameRect(hdc, ref borderRect, borderBrush);
            DeleteObject(borderBrush);

            EndPaint(hwnd, ref ps);
            return nint.Zero;
        }

        return DefWindowProc(hwnd, msg, wParam, lParam);
    }

    public void Close()
    {
        if (_hwnd != nint.Zero)
        {
            PostMessage(_hwnd, 0x0010, nint.Zero, nint.Zero); // WM_CLOSE
            _hwnd = nint.Zero;
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        Close();

        // Wait for message thread to finish (with timeout)
        _messageThread?.Join(100);

        if (_logoImage != nint.Zero)
        {
            _ = GdipDisposeImage(_logoImage);
            _logoImage = nint.Zero;
        }

        if (_gdiplusToken != nint.Zero)
        {
            GdiplusShutdown(_gdiplusToken);
            _gdiplusToken = nint.Zero;
        }

        _windowCreated.Dispose();
    }

    #region Win32 Imports

    private delegate nint WndProcDelegate(nint hWnd, uint msg, nint wParam, nint lParam);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern ushort RegisterClassEx(ref WNDCLASSEX lpWndClass);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern nint CreateWindowEx(uint dwExStyle, string lpClassName, string lpWindowName,
        uint dwStyle, int x, int y, int nWidth, int nHeight, nint hWndParent, nint hMenu, nint hInstance, nint lpParam);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ShowWindow(nint hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UpdateWindow(nint hWnd);

    [DllImport("user32.dll")]
    private static extern int GetMessage(out MSG lpMsg, nint hWnd, uint wMsgFilterMin, uint wMsgFilterMax);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool TranslateMessage(ref MSG lpMsg);

    [DllImport("user32.dll")]
    private static extern nint DispatchMessage(ref MSG lpMsg);

    [DllImport("user32.dll")]
    private static extern nint DefWindowProc(nint hWnd, uint msg, nint wParam, nint lParam);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool PostMessage(nint hWnd, uint msg, nint wParam, nint lParam);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool InvalidateRect(nint hWnd, nint lpRect, [MarshalAs(UnmanagedType.Bool)] bool bErase);

    [DllImport("user32.dll")]
    private static extern nint LoadCursor(nint hInstance, int lpCursorName);

    [DllImport("user32.dll")]
    private static extern int GetSystemMetrics(int nIndex);

    [DllImport("user32.dll")]
    private static extern nint BeginPaint(nint hWnd, out PAINTSTRUCT lpPaint);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool EndPaint(nint hWnd, ref PAINTSTRUCT lpPaint);

    [DllImport("user32.dll")]
    private static extern int FillRect(nint hDC, ref RECT lprc, nint hbr);

    [DllImport("user32.dll")]
    private static extern int FrameRect(nint hDC, ref RECT lprc, nint hbr);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int DrawText(nint hdc, string lpchText, int cchText, ref RECT lprc, uint format);

    [DllImport("gdi32.dll")]
    private static extern nint CreateSolidBrush(uint crColor);

    [DllImport("gdi32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DeleteObject(nint ho);

    [DllImport("gdi32.dll")]
    private static extern nint SelectObject(nint hdc, nint h);

    [DllImport("gdi32.dll")]
    private static extern int SetBkMode(nint hdc, int mode);

    [DllImport("gdi32.dll")]
    private static extern uint SetTextColor(nint hdc, uint color);

    [DllImport("gdi32.dll", CharSet = CharSet.Unicode)]
    private static extern nint CreateFont(int cHeight, int cWidth, int cEscapement, int cOrientation, int cWeight,
        uint bItalic, uint bUnderline, uint bStrikeOut, uint iCharSet, uint iOutPrecision, uint iClipPrecision,
        uint iQuality, uint iPitchAndFamily, string pszFaceName);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode)]
    private static extern nint GetModuleHandle(string? lpModuleName);

    // GDI+ imports for PNG loading
    [DllImport("gdiplus.dll")]
    private static extern int GdiplusStartup(out nint token, ref GdiplusStartupInput input, out nint output);

    [DllImport("gdiplus.dll")]
    private static extern void GdiplusShutdown(nint token);

    [DllImport("gdiplus.dll", CharSet = CharSet.Unicode)]
    private static extern int GdipLoadImageFromFile(string filename, out nint image);

    [DllImport("gdiplus.dll")]
    private static extern int GdipCreateFromHDC(nint hdc, out nint graphics);

    [DllImport("gdiplus.dll")]
    private static extern int GdipDrawImageRectI(nint graphics, nint image, int x, int y, int width, int height);

    [DllImport("gdiplus.dll")]
    private static extern int GdipDeleteGraphics(nint graphics);

    [DllImport("gdiplus.dll")]
    private static extern int GdipDisposeImage(nint image);

    [StructLayout(LayoutKind.Sequential)]
    private struct GdiplusStartupInput
    {
        public uint GdiplusVersion;
        public nint DebugEventCallback;
        public int SuppressBackgroundThread;
        public int SuppressExternalCodecs;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct WNDCLASSEX
    {
        public uint cbSize;
        public uint style;
        public nint lpfnWndProc;
        public int cbClsExtra;
        public int cbWndExtra;
        public nint hInstance;
        public nint hIcon;
        public nint hCursor;
        public nint hbrBackground;
        public string? lpszMenuName;
        public string lpszClassName;
        public nint hIconSm;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MSG
    {
        public nint hwnd;
        public uint message;
        public nint wParam;
        public nint lParam;
        public uint time;
        public int pt_x;
        public int pt_y;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct RECT
    {
        public int left, top, right, bottom;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct PAINTSTRUCT
    {
        public nint hdc;
        public bool fErase;
        public RECT rcPaint;
        public bool fRestore;
        public bool fIncUpdate;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 32)]
        public byte[]? rgbReserved;
    }

    #endregion
}
