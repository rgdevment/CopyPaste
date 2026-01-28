using CopyPaste.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using System;
using System.Diagnostics;

namespace CopyPaste.UI;

/// <summary>
/// Configuration window for managing application settings.
/// Loads values from MyM.json, allows editing, and saves to MyM.json.
/// </summary>
public sealed partial class ConfigWindow : Window
{
    public ConfigWindow()
    {
        InitializeComponent();

        // Set window size
        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
        var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hwnd);
        var appWindow = Microsoft.UI.Windowing.AppWindow.GetFromWindowId(windowId);
        appWindow.Resize(new Windows.Graphics.SizeInt32(620, 900));

        // Center window
        CenterWindow(appWindow);

        // Ensure config directory exists
        StorageConfig.Initialize();

        // Load current configuration values
        LoadCurrentValues();

        // Wire up hotkey preview updates
        UseCtrlCheck.Checked += (_, _) => UpdateHotkeyPreview();
        UseCtrlCheck.Unchecked += (_, _) => UpdateHotkeyPreview();
        UseWinCheck.Checked += (_, _) => UpdateHotkeyPreview();
        UseWinCheck.Unchecked += (_, _) => UpdateHotkeyPreview();
        UseAltCheck.Checked += (_, _) => UpdateHotkeyPreview();
        UseAltCheck.Unchecked += (_, _) => UpdateHotkeyPreview();
        UseShiftCheck.Checked += (_, _) => UpdateHotkeyPreview();
        UseShiftCheck.Unchecked += (_, _) => UpdateHotkeyPreview();
        HotkeyCombo.SelectionChanged += (_, _) => UpdateHotkeyPreview();
    }

    private static void CenterWindow(Microsoft.UI.Windowing.AppWindow appWindow)
    {
        var displayArea = Microsoft.UI.Windowing.DisplayArea.GetFromWindowId(
            appWindow.Id,
            Microsoft.UI.Windowing.DisplayAreaFallback.Primary);

        var centerX = (displayArea.WorkArea.Width - appWindow.Size.Width) / 2;
        var centerY = (displayArea.WorkArea.Height - appWindow.Size.Height) / 2;

        appWindow.Move(new Windows.Graphics.PointInt32(centerX, centerY));
    }

    /// <summary>
    /// Loads current configuration values from MyM.json into the UI controls.
    /// </summary>
    private void LoadCurrentValues()
    {
        ConfigLoader.ClearCache();
        var config = ConfigLoader.Load();

        // Startup
        RunOnStartupSwitch.IsOn = config.RunOnStartup;

        // Hotkey modifiers
        UseCtrlCheck.IsChecked = config.UseCtrlKey;
        UseWinCheck.IsChecked = config.UseWinKey;
        UseAltCheck.IsChecked = config.UseAltKey;
        UseShiftCheck.IsChecked = config.UseShiftKey;

        // Hotkey key - find by Tag value
        SelectHotkeyByVirtualKey(config.VirtualKey);

        // UI / Appearance
        WindowWidthBox.Value = config.WindowWidth;
        WindowMarginTopBox.Value = config.WindowMarginTop;
        WindowMarginBottomBox.Value = config.WindowMarginBottom;

        // Performance
        PageSizeBox.Value = config.PageSize;
        MaxItemsBeforeCleanupBox.Value = config.MaxItemsBeforeCleanup;
        ScrollLoadThresholdBox.Value = config.ScrollLoadThreshold;

        // Storage
        RetentionDaysBox.Value = config.RetentionDays;

        // Paste
        DuplicateIgnoreWindowMsBox.Value = config.DuplicateIgnoreWindowMs;
        DelayBeforeFocusMsBox.Value = config.DelayBeforeFocusMs;
        DelayBeforePasteMsBox.Value = config.DelayBeforePasteMs;
        MaxFocusVerifyAttemptsBox.Value = config.MaxFocusVerifyAttempts;

        // Thumbnail
        ThumbnailWidthBox.Value = config.ThumbnailWidth;
        ThumbnailQualityPngBox.Value = config.ThumbnailQualityPng;
        ThumbnailQualityJpegBox.Value = config.ThumbnailQualityJpeg;
        ThumbnailGCThresholdBox.Value = config.ThumbnailGCThreshold;
        ThumbnailUIDecodeHeightBox.Value = config.ThumbnailUIDecodeHeight;

        UpdateHotkeyPreview();
    }

    private void SelectHotkeyByVirtualKey(uint vk)
    {
        for (int i = 0; i < HotkeyCombo.Items.Count; i++)
        {
            if (HotkeyCombo.Items[i] is ComboBoxItem item &&
                item.Tag is string tag &&
                uint.TryParse(tag, out var itemVk) &&
                itemVk == vk)
            {
                HotkeyCombo.SelectedIndex = i;
                return;
            }
        }
        HotkeyCombo.SelectedIndex = 0; // Default to V
    }

    private void UpdateHotkeyPreview()
    {
        var parts = new System.Collections.Generic.List<string>();

        if (UseCtrlCheck.IsChecked == true) parts.Add("Ctrl");
        if (UseWinCheck.IsChecked == true) parts.Add("Win");
        if (UseAltCheck.IsChecked == true) parts.Add("Alt");
        if (UseShiftCheck.IsChecked == true) parts.Add("Shift");

        var selectedKey = (HotkeyCombo.SelectedItem as ComboBoxItem)?.Content?.ToString() ?? "V";
        parts.Add(selectedKey);

        HotkeyPreview.Text = $"Atajo actual: {string.Join(" + ", parts)}";
    }

    private (uint VirtualKey, string KeyName) GetSelectedHotkey()
    {
        if (HotkeyCombo.SelectedItem is ComboBoxItem item)
        {
            var keyName = item.Content?.ToString() ?? "V";
            if (item.Tag is string tag && uint.TryParse(tag, out var vk))
                return (vk, keyName);
        }
        return (0x56, "V"); // Default
    }

    private void ResetButton_Click(object sender, RoutedEventArgs e)
    {
        var d = new MyMConfig();

        RunOnStartupSwitch.IsOn = d.RunOnStartup;
        UseCtrlCheck.IsChecked = d.UseCtrlKey;
        UseWinCheck.IsChecked = d.UseWinKey;
        UseAltCheck.IsChecked = d.UseAltKey;
        UseShiftCheck.IsChecked = d.UseShiftKey;
        SelectHotkeyByVirtualKey(d.VirtualKey);

        WindowWidthBox.Value = d.WindowWidth;
        WindowMarginTopBox.Value = d.WindowMarginTop;
        WindowMarginBottomBox.Value = d.WindowMarginBottom;

        PageSizeBox.Value = d.PageSize;
        MaxItemsBeforeCleanupBox.Value = d.MaxItemsBeforeCleanup;
        ScrollLoadThresholdBox.Value = d.ScrollLoadThreshold;
        RetentionDaysBox.Value = d.RetentionDays;

        DuplicateIgnoreWindowMsBox.Value = d.DuplicateIgnoreWindowMs;
        DelayBeforeFocusMsBox.Value = d.DelayBeforeFocusMs;
        DelayBeforePasteMsBox.Value = d.DelayBeforePasteMs;
        MaxFocusVerifyAttemptsBox.Value = d.MaxFocusVerifyAttempts;

        ThumbnailWidthBox.Value = d.ThumbnailWidth;
        ThumbnailQualityPngBox.Value = d.ThumbnailQualityPng;
        ThumbnailQualityJpegBox.Value = d.ThumbnailQualityJpeg;
        ThumbnailGCThresholdBox.Value = d.ThumbnailGCThreshold;
        ThumbnailUIDecodeHeightBox.Value = d.ThumbnailUIDecodeHeight;

        UpdateHotkeyPreview();
    }

    private void CancelButton_Click(object sender, RoutedEventArgs e) => Close();

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Save operation should not crash - errors are logged")]
    private void SaveButton_Click(object sender, RoutedEventArgs e)
    {
        // Validate: at least 2 modifiers required
        int modifierCount = 0;
        if (UseCtrlCheck.IsChecked == true) modifierCount++;
        if (UseWinCheck.IsChecked == true) modifierCount++;
        if (UseAltCheck.IsChecked == true) modifierCount++;
        if (UseShiftCheck.IsChecked == true) modifierCount++;

        if (modifierCount < 2)
        {
            ShowErrorDialog("Debes seleccionar al menos 2 teclas modificadoras (Ctrl, Win, Alt o Shift).");
            return;
        }

        try
        {
            var (virtualKey, keyName) = GetSelectedHotkey();

            var config = new MyMConfig
            {
                // Startup
                RunOnStartup = RunOnStartupSwitch.IsOn,

                // Hotkey
                UseCtrlKey = UseCtrlCheck.IsChecked == true,
                UseWinKey = UseWinCheck.IsChecked == true,
                UseAltKey = UseAltCheck.IsChecked == true,
                UseShiftKey = UseShiftCheck.IsChecked == true,
                VirtualKey = virtualKey,
                KeyName = keyName,

                // UI / Appearance
                WindowWidth = (int)WindowWidthBox.Value,
                WindowMarginTop = (int)WindowMarginTopBox.Value,
                WindowMarginBottom = (int)WindowMarginBottomBox.Value,

                // Performance
                PageSize = (int)PageSizeBox.Value,
                MaxItemsBeforeCleanup = (int)MaxItemsBeforeCleanupBox.Value,
                ScrollLoadThreshold = (int)ScrollLoadThresholdBox.Value,

                // Storage
                RetentionDays = (int)RetentionDaysBox.Value,

                // Paste
                DuplicateIgnoreWindowMs = (int)DuplicateIgnoreWindowMsBox.Value,
                DelayBeforeFocusMs = (int)DelayBeforeFocusMsBox.Value,
                DelayBeforePasteMs = (int)DelayBeforePasteMsBox.Value,
                MaxFocusVerifyAttempts = (int)MaxFocusVerifyAttemptsBox.Value,

                // Thumbnail
                ThumbnailWidth = (int)ThumbnailWidthBox.Value,
                ThumbnailQualityPng = (int)ThumbnailQualityPngBox.Value,
                ThumbnailQualityJpeg = (int)ThumbnailQualityJpegBox.Value,
                ThumbnailGCThreshold = (int)ThumbnailGCThresholdBox.Value,
                ThumbnailUIDecodeHeight = (int)ThumbnailUIDecodeHeightBox.Value
            };

            AppLogger.Info($"Attempting to save config to: {ConfigLoader.ConfigFilePath}");

            if (ConfigLoader.Save(config))
            {
                RestartApplication();
            }
            else
            {
                ShowErrorDialog("Error al guardar. Revisa los permisos de la carpeta de configuración.");
            }
        }
        catch (Exception ex)
        {
            AppLogger.Error($"Save failed: {ex.Message}\n{ex.StackTrace}");
            ShowErrorDialog($"Error: {ex.Message}");
        }
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types",
        Justification = "Restart is best-effort")]
    private static void RestartApplication()
    {
        try
        {
            var exePath = Environment.ProcessPath;
            if (!string.IsNullOrEmpty(exePath))
            {
                Process.Start(new ProcessStartInfo { FileName = exePath, UseShellExecute = true });
                if (App.Current is App app) app.BeginExit();
            }
        }
        catch (Exception ex)
        {
            AppLogger.Error($"Failed to restart: {ex.Message}");
        }
    }

    private async void ShowErrorDialog(string message)
    {
        var dialog = new ContentDialog
        {
            Title = "Error",
            Content = message,
            CloseButtonText = "Aceptar",
            XamlRoot = Content.XamlRoot
        };
        await dialog.ShowAsync();
    }
}
