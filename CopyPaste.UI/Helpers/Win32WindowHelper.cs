using System;
using System.Runtime.InteropServices;

namespace CopyPaste.UI.Helpers;

/// <summary>
/// Helper class for Win32 window manipulation APIs.
/// </summary>
internal static partial class Win32WindowHelper
{
    #region P/Invoke Declarations

    [LibraryImport("user32.dll")]
    private static partial nint GetWindowLongPtrW(IntPtr hWnd, int nIndex);

    [LibraryImport("user32.dll")]
    private static partial nint SetWindowLongPtrW(IntPtr hWnd, int nIndex, nint dwNewLong);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int x, int y, int cx, int cy, uint uFlags);

    [LibraryImport("dwmapi.dll")]
    private static partial int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref uint attrValue, int attrSize);

    [LibraryImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);

    [LibraryImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static partial bool UnregisterHotKey(IntPtr hWnd, int id);

    #endregion

    #region Constants

    private const int _gWL_STYLE = -16;
    private const int _gWL_EXSTYLE = -20;

    private const nint _wS_BORDER = 0x00800000;
    private const nint _wS_DLGFRAME = 0x00400000;
    private const nint _wS_THICKFRAME = 0x00040000;

    private const nint _wS_EX_WINDOWEDGE = 0x00000100;
    private const nint _wS_EX_CLIENTEDGE = 0x00000200;
    private const nint _wS_EX_STATICEDGE = 0x00020000;

    private const uint _sWP_FRAMECHANGED = 0x0020;
    private const uint _sWP_NOMOVE = 0x0002;
    private const uint _sWP_NOSIZE = 0x0001;
    private const uint _sWP_NOZORDER = 0x0004;
    private const uint _sWP_NOACTIVATE = 0x0010;

    private const int _dWMWA_WINDOW_CORNER_PREFERENCE = 33;

    public const uint MOD_ALT = 0x0001;
    public const uint MOD_CONTROL = 0x0002;
    public const uint MOD_SHIFT = 0x0004;
    public const uint MOD_WIN = 0x0008;

    #endregion

    /// <summary>
    /// Removes the window border and applies rounded corners.
    /// </summary>
    public static void RemoveWindowBorder(IntPtr hWnd)
    {
        // Remove standard window styles
        nint style = GetWindowLongPtrW(hWnd, _gWL_STYLE);
        style &= ~_wS_BORDER;
        style &= ~_wS_DLGFRAME;
        style &= ~_wS_THICKFRAME;
        SetWindowLongPtrW(hWnd, _gWL_STYLE, style);

        // Remove extended window styles
        nint exStyle = GetWindowLongPtrW(hWnd, _gWL_EXSTYLE);
        exStyle &= ~_wS_EX_WINDOWEDGE;
        exStyle &= ~_wS_EX_CLIENTEDGE;
        exStyle &= ~_wS_EX_STATICEDGE;
        SetWindowLongPtrW(hWnd, _gWL_EXSTYLE, exStyle);

        // Apply changes
        SetWindowPos(hWnd, IntPtr.Zero, 0, 0, 0, 0,
            _sWP_FRAMECHANGED | _sWP_NOMOVE | _sWP_NOSIZE | _sWP_NOZORDER | _sWP_NOACTIVATE);

        // Set rounded corners
        uint cornerPreference = 2;
        _ = DwmSetWindowAttribute(hWnd, _dWMWA_WINDOW_CORNER_PREFERENCE, ref cornerPreference, sizeof(uint));
    }
}
