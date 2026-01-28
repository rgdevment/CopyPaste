using System;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using CopyPaste.Core;

namespace CopyPaste.UI.Helpers;

/// <summary>
/// Helper class for managing window focus and simulating keyboard input.
/// Used to restore focus to the previous window and simulate paste (Ctrl+V).
/// </summary>
internal static partial class FocusHelper
{
    #region P/Invoke Declarations

    [LibraryImport("user32.dll")]
    private static partial IntPtr GetForegroundWindow();

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool SetForegroundWindow(IntPtr hWnd);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool IsWindow(IntPtr hWnd);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool IsWindowVisible(IntPtr hWnd);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool BringWindowToTop(IntPtr hWnd);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [LibraryImport("user32.dll")]
    private static partial uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool AttachThreadInput(uint idAttach, uint idAttachTo, [MarshalAs(UnmanagedType.Bool)] bool fAttach);

    [LibraryImport("kernel32.dll")]
    private static partial uint GetCurrentThreadId();

    [LibraryImport("user32.dll")]
    private static partial void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);

    [LibraryImport("user32.dll")]
    private static partial IntPtr GetWindowLongPtrW(IntPtr hWnd, int nIndex);

    #endregion

    #region Constants

    private const uint _kEYEVENTF_KEYUP = 0x0002;
    private const byte _vK_CONTROL = 0x11;
    private const byte _vK_V = 0x56;
    private const int _sW_RESTORE = 9;
    private const int _gWL_STYLE = -16;
    private const long _wS_MINIMIZE = 0x20000000;

    #endregion

    private static IntPtr _previousForegroundWindow = IntPtr.Zero;
    private static uint _previousWindowThreadId;

    /// <summary>
    /// Captures the currently focused window. Call this before showing our window.
    /// </summary>
    public static void CapturePreviousWindow()
    {
        var hwnd = GetForegroundWindow();

        if (hwnd != IntPtr.Zero && IsWindow(hwnd) && IsWindowVisible(hwnd))
        {
            _previousForegroundWindow = hwnd;
            _previousWindowThreadId = GetWindowThreadProcessId(hwnd, out _);
        }
        else
        {
            AppLogger.Warn("Failed to capture valid window");
            _previousForegroundWindow = IntPtr.Zero;
        }
    }

    /// <summary>
    /// Restores focus to the previously captured window.
    /// </summary>
    public static bool RestorePreviousWindow()
    {
        if (_previousForegroundWindow == IntPtr.Zero)
        {
            AppLogger.Warn("No previous window to restore");
            return false;
        }

        if (!IsWindow(_previousForegroundWindow))
        {
            AppLogger.Warn("Window no longer exists");
            _previousForegroundWindow = IntPtr.Zero;
            return false;
        }

        try
        {
            uint currentThreadId = GetCurrentThreadId();
            bool attached = false;

            if (currentThreadId != _previousWindowThreadId && _previousWindowThreadId != 0)
            {
                attached = AttachThreadInput(currentThreadId, _previousWindowThreadId, true);
            }

            try
            {
                // Restore if minimized
                var style = GetWindowLongPtrW(_previousForegroundWindow, _gWL_STYLE);
                if ((style.ToInt64() & _wS_MINIMIZE) != 0)
                {
                    ShowWindow(_previousForegroundWindow, _sW_RESTORE);
                }

                BringWindowToTop(_previousForegroundWindow);
                return SetForegroundWindow(_previousForegroundWindow);
            }
            finally
            {
                if (attached)
                {
                    AttachThreadInput(currentThreadId, _previousWindowThreadId, false);
                }
            }
        }
        catch (System.ComponentModel.Win32Exception ex)
        {
            AppLogger.Exception(ex, "Restore window failed");
            throw;
        }
        catch (InvalidOperationException ex)
        {
            AppLogger.Exception(ex, "Restore window failed");
            throw;
        }
    }

    /// <summary>
    /// Waits for focus to be on the target window using active polling instead of fixed delay.
    /// </summary>
    private static async Task<bool> WaitForFocusAsync(int maxAttempts)
    {
        for (int i = 0; i < maxAttempts; i++)
        {
            if (GetForegroundWindow() == _previousForegroundWindow)
            {
                return true;
            }
            await Task.Delay(10).ConfigureAwait(false);
        }
        return false;
    }

    /// <summary>
    /// Simulates Ctrl+V keystroke.
    /// </summary>
    public static void SimulatePaste()
    {
        keybd_event(_vK_CONTROL, 0, 0, UIntPtr.Zero);
        keybd_event(_vK_V, 0, 0, UIntPtr.Zero);
        keybd_event(_vK_V, 0, _kEYEVENTF_KEYUP, UIntPtr.Zero);
        keybd_event(_vK_CONTROL, 0, _kEYEVENTF_KEYUP, UIntPtr.Zero);
    }

    /// <summary>
    /// Restores focus and simulates paste using configured timing.
    /// </summary>
    public static async Task RestoreAndPasteAsync()
    {
        if (_previousForegroundWindow == IntPtr.Zero)
        {
            AppLogger.Warn("No window to restore for paste");
            return;
        }

        var config = ConfigLoader.Config;

        // Wait for our window to hide
        await Task.Delay(config.DelayBeforeFocusMs).ConfigureAwait(false);

        if (!RestorePreviousWindow())
        {
            AppLogger.Warn("Could not restore window for paste");
            return;
        }

        // Use active polling to verify focus, with fallback to delay
        bool focusConfirmed = await WaitForFocusAsync(config.MaxFocusVerifyAttempts).ConfigureAwait(false);

        if (!focusConfirmed)
        {
            // Fallback: wait the configured delay
            await Task.Delay(config.DelayBeforePasteMs).ConfigureAwait(false);
        }

        SimulatePaste();
    }

    /// <summary>
    /// Clears the captured window reference.
    /// </summary>
    public static void ClearPreviousWindow()
    {
        _previousForegroundWindow = IntPtr.Zero;
        _previousWindowThreadId = 0;
    }
}
