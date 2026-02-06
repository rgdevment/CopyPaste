using CopyPaste.Core.Themes;
using CopyPaste.UI.Helpers;
using Microsoft.UI.Windowing;
using System;
using WinRT.Interop;

namespace CopyPaste.UI.Themes;

internal sealed class CompactTheme : ITheme
{
    public string Id => "copypaste.compact";
    public string Name => "Compact";
    public string Version => "1.0.0";
    public string Author => "CopyPaste";

    private CompactWindow? _window;
    private AppWindow? _appWindow;
    private IntPtr _hWnd;
    private CompactSettings _themeSettings = new();
    private CompactSettingsPanel? _settingsPanel;

    public IntPtr CreateWindow(ThemeContext context)
    {
        _themeSettings = CompactSettings.Load();
        _window = new CompactWindow(context, _themeSettings);
        _hWnd = WindowNative.GetWindowHandle(_window);
        _appWindow = AppWindow.GetFromWindowId(Microsoft.UI.Win32Interop.GetWindowIdFromWindow(_hWnd));
        return _hWnd;
    }

    public bool IsVisible => _appWindow?.IsVisible ?? false;

    public void Show()
    {
        _window?.PositionAtCursor();
        _window?.Activate();
        if (_hWnd != IntPtr.Zero)
            Win32WindowHelper.SetForegroundWindow(_hWnd);
    }

    public void Hide()
    {
        _window?.CollapseAllCards();
        _appWindow?.Hide();
    }

    public void Toggle()
    {
        if (IsVisible)
            Hide();
        else
            Show();
    }

    public object? CreateSettingsSection()
    {
        _settingsPanel = new CompactSettingsPanel();
        return _settingsPanel.Build(_themeSettings);
    }

    public void SaveThemeSettings()
    {
        if (_settingsPanel != null)
        {
            _themeSettings = _settingsPanel.ToSettings();
            CompactSettings.Save(_themeSettings);
        }
    }

    public void ResetThemeSettings() => _settingsPanel?.Reset();

    public void Dispose()
    {
        _window?.Close();
        _window = null;
        _appWindow = null;
    }
}
