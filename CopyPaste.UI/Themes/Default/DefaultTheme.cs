using CopyPaste.Core.Themes;
using CopyPaste.UI.Helpers;
using Microsoft.UI.Windowing;
using System;
using WinRT.Interop;

namespace CopyPaste.UI.Themes;

internal sealed class DefaultTheme : ITheme
{
    public string Id => "copypaste.default";
    public string Name => "Default";
    public string Version => "1.0.0";
    public string Author => "CopyPaste";

    private DefaultThemeWindow? _window;
    private AppWindow? _appWindow;
    private IntPtr _hWnd;
    private DefaultThemeSettings _themeSettings = new();
    private DefaultThemeSettingsPanel? _settingsPanel;

    public IntPtr CreateWindow(ThemeContext context)
    {
        _themeSettings = DefaultThemeSettings.Load();
        _window = new DefaultThemeWindow(context, _themeSettings);
        _hWnd = WindowNative.GetWindowHandle(_window);
        _appWindow = AppWindow.GetFromWindowId(Microsoft.UI.Win32Interop.GetWindowIdFromWindow(_hWnd));
        return _hWnd;
    }

    public bool IsVisible => _appWindow?.IsVisible ?? false;

    public void Show()
    {
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
        _settingsPanel = new DefaultThemeSettingsPanel();
        return _settingsPanel.Build(_themeSettings);
    }

    public void SaveThemeSettings()
    {
        if (_settingsPanel != null)
        {
            _themeSettings = _settingsPanel.ToSettings();
            DefaultThemeSettings.Save(_themeSettings);
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
