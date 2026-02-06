using CopyPaste.Core;
using System;
using System.Runtime.InteropServices;

namespace CopyPaste.UI.Helpers;

/// <summary>
/// Helper class to handle Windows messages for hotkey support in WinUI 3.
/// </summary>
internal static partial class HotkeyHelper
{
    private const int _wM_HOTKEY = 0x0312;
    private static SubclassProc? _subclassProcDelegate;
    private static Action? _hotkeyAction;
    private const nuint _sUBCLASS_ID = 1;

    public static void RegisterMessageHandler(IntPtr hwnd, Action onHotkeyPressed)
    {
        _hotkeyAction = onHotkeyPressed;
        _subclassProcDelegate = SubclassProcedure;
        SetWindowSubclass(hwnd, _subclassProcDelegate, _sUBCLASS_ID, IntPtr.Zero);
    }

    public static void UnregisterMessageHandler(IntPtr hwnd)
    {
        RemoveWindowSubclass(hwnd, _subclassProcDelegate, _sUBCLASS_ID);
        _subclassProcDelegate = null;
        _hotkeyAction = null;
    }

    private static nint SubclassProcedure(IntPtr hWnd, uint uMsg, IntPtr wParam, IntPtr lParam, nuint uIdSubclass, IntPtr dwRefData)
    {
        if (uMsg == _wM_HOTKEY)
        {
            // IMPORTANT: Capture the foreground window BEFORE invoking the action
            // At this point, the previous window still has focus
            FocusHelper.CapturePreviousWindow();

            _hotkeyAction?.Invoke();
            return IntPtr.Zero;
        }

        return DefSubclassProc(hWnd, uMsg, wParam, lParam);
    }

    private delegate nint SubclassProc(IntPtr hWnd, uint uMsg, IntPtr wParam, IntPtr lParam, nuint uIdSubclass, IntPtr dwRefData);

    [LibraryImport("comctl32.dll")]
    [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.Bool)]
    private static partial bool SetWindowSubclass(IntPtr hWnd, SubclassProc pfnSubclass, nuint uIdSubclass, IntPtr dwRefData);

    [LibraryImport("comctl32.dll")]
    [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.Bool)]
    private static partial bool RemoveWindowSubclass(IntPtr hWnd, SubclassProc? pfnSubclass, nuint uIdSubclass);

    [LibraryImport("comctl32.dll")]
    private static partial nint DefSubclassProc(IntPtr hWnd, uint uMsg, IntPtr wParam, IntPtr lParam);
}
