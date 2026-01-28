using CopyPaste.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using System;
using System.Diagnostics;

namespace CopyPaste.UI;

/// <summary>
/// Configuration window for managing application settings.
/// Loads values from MyM.json, allows editing, and saves to MyM.json.
/// Implements proper resource cleanup on close.
/// </summary>
public sealed partial class ConfigWindow : Window
{
    private record ThumbnailPreset(int Width, int QualityPng, int QualityJpeg, int GCThreshold, int UIDecodeHeight);
    private record PastePreset(int DuplicateIgnoreMs, int DelayBeforeFocusMs, int DelayBeforePasteMs, int MaxFocusAttempts);

    private static readonly ThumbnailPreset _presetExcellent = new(280, 95, 95, 2_000_000, 157);
    private static readonly ThumbnailPreset _presetHigh = new(220, 90, 90, 1_500_000, 124);
    private static readonly ThumbnailPreset _presetMedium = new(170, 80, 80, 1_000_000, 95);
    private static readonly ThumbnailPreset _presetLow = new(140, 70, 70, 750_000, 79);
    private static readonly ThumbnailPreset _presetMinimal = new(110, 60, 60, 500_000, 62);

    private static readonly PastePreset _pasteRapido = new(250, 40, 80, 8);
    private static readonly PastePreset _pasteNormal = new(300, 60, 110, 10);
    private static readonly PastePreset _pasteSeguro = new(450, 100, 180, 15);
    private static readonly PastePreset _pasteLento = new(600, 150, 300, 15);

    private bool _isLoadingValues;
    private ThumbnailPreset? _originalThumbnailValues;
    private PastePreset? _originalPasteValues;

    public ConfigWindow()
    {
        InitializeComponent();

        // Set window size
        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
        var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hwnd);
        var appWindow = Microsoft.UI.Windowing.AppWindow.GetFromWindowId(windowId);
        appWindow.Resize(new Windows.Graphics.SizeInt32(620, 780));

        // Center window
        CenterWindow(appWindow);

        // Ensure config directory exists
        StorageConfig.Initialize();

        // Load current configuration values (single read, then cached)
        LoadCurrentValues();

        // Wire up hotkey preview updates
        UseCtrlCheck.Checked += OnHotkeyChanged;
        UseCtrlCheck.Unchecked += OnHotkeyChanged;
        UseWinCheck.Checked += OnHotkeyChanged;
        UseWinCheck.Unchecked += OnHotkeyChanged;
        UseAltCheck.Checked += OnHotkeyChanged;
        UseAltCheck.Unchecked += OnHotkeyChanged;
        UseShiftCheck.Checked += OnHotkeyChanged;
        UseShiftCheck.Unchecked += OnHotkeyChanged;
        HotkeyCombo.SelectionChanged += OnHotkeyComboChanged;

        // Register for window close to cleanup resources
        this.Closed += OnWindowClosed;
    }

    private void OnHotkeyChanged(object sender, RoutedEventArgs e) => UpdateHotkeyPreview();
    private void OnHotkeyComboChanged(object sender, SelectionChangedEventArgs e) => UpdateHotkeyPreview();

    private void OnWindowClosed(object sender, WindowEventArgs e)
    {
        // Unsubscribe all event handlers to prevent memory leaks
        UseCtrlCheck.Checked -= OnHotkeyChanged;
        UseCtrlCheck.Unchecked -= OnHotkeyChanged;
        UseWinCheck.Checked -= OnHotkeyChanged;
        UseWinCheck.Unchecked -= OnHotkeyChanged;
        UseAltCheck.Checked -= OnHotkeyChanged;
        UseAltCheck.Unchecked -= OnHotkeyChanged;
        UseShiftCheck.Checked -= OnHotkeyChanged;
        UseShiftCheck.Unchecked -= OnHotkeyChanged;
        HotkeyCombo.SelectionChanged -= OnHotkeyComboChanged;
        this.Closed -= OnWindowClosed;

        // Clear references to allow GC
        _originalThumbnailValues = null;
        _originalPasteValues = null;
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
    /// Forces a fresh read from disk to show actual saved values.
    /// </summary>
    private void LoadCurrentValues()
    {
        _isLoadingValues = true;

        // Force fresh read from JSON file (not cached values)
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

        // Paste - detect preset
        SelectPastePreset(config);

        // Thumbnail - detect closest preset
        SelectThumbnailPreset(config);

        UpdateHotkeyPreview();
        _isLoadingValues = false;
    }

    private void SelectPastePreset(MyMConfig config)
    {
        // Store original values for custom preset
        _originalPasteValues = new PastePreset(
            config.DuplicateIgnoreWindowMs,
            config.DelayBeforeFocusMs,
            config.DelayBeforePasteMs,
            config.MaxFocusVerifyAttempts);

        // Remove existing "Personalizado" item if present
        if (PastePresetCombo.Items.Count > 4)
        {
            PastePresetCombo.Items.RemoveAt(4);
        }

        // Match exact presets
        if (_originalPasteValues == _pasteRapido) PastePresetCombo.SelectedIndex = 0;
        else if (_originalPasteValues == _pasteNormal) PastePresetCombo.SelectedIndex = 1;
        else if (_originalPasteValues == _pasteSeguro) PastePresetCombo.SelectedIndex = 2;
        else if (_originalPasteValues == _pasteLento) PastePresetCombo.SelectedIndex = 3;
        else
        {
            // Add "Personalizado" dynamically
            PastePresetCombo.Items.Add(new ComboBoxItem
            {
                Content = "Personalizado (desde JSON)",
                Tag = "custom",
                IsEnabled = false
            });
            PastePresetCombo.SelectedIndex = 4;
        }
    }

    private void SelectThumbnailPreset(MyMConfig config)
    {
        // Store original values for custom preset
        _originalThumbnailValues = new ThumbnailPreset(
            config.ThumbnailWidth,
            config.ThumbnailQualityPng,
            config.ThumbnailQualityJpeg,
            config.ThumbnailGCThreshold,
            config.ThumbnailUIDecodeHeight);

        // Remove existing "Personalizado" item if present
        if (ThumbnailPresetCombo.Items.Count > 5)
        {
            ThumbnailPresetCombo.Items.RemoveAt(5);
        }

        // Match exact presets
        if (_originalThumbnailValues == _presetExcellent) ThumbnailPresetCombo.SelectedIndex = 0;
        else if (_originalThumbnailValues == _presetHigh) ThumbnailPresetCombo.SelectedIndex = 1;
        else if (_originalThumbnailValues == _presetMedium) ThumbnailPresetCombo.SelectedIndex = 2;
        else if (_originalThumbnailValues == _presetLow) ThumbnailPresetCombo.SelectedIndex = 3;
        else if (_originalThumbnailValues == _presetMinimal) ThumbnailPresetCombo.SelectedIndex = 4;
        else
        {
            // Add "Personalizado" dynamically
            ThumbnailPresetCombo.Items.Add(new ComboBoxItem
            {
                Content = "Personalizado (desde JSON)",
                Tag = "custom",
                IsEnabled = false
            });
            ThumbnailPresetCombo.SelectedIndex = 5;
        }
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

    private void ThumbnailPresetCombo_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_isLoadingValues) return;
        // Preset will be applied on save
    }

    private void PastePresetCombo_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_isLoadingValues) return;
        // Preset will be applied on save
    }

    private ThumbnailPreset GetSelectedThumbnailPreset() =>
        ThumbnailPresetCombo.SelectedIndex switch
        {
            0 => _presetExcellent,
            1 => _presetHigh,
            2 => _presetMedium,
            3 => _presetLow,
            4 => _presetMinimal,
            5 => _originalThumbnailValues ?? _presetMedium,
            _ => _presetMedium
        };

    private PastePreset GetSelectedPastePreset() =>
        PastePresetCombo.SelectedIndex switch
        {
            0 => _pasteRapido,
            1 => _pasteNormal,
            2 => _pasteSeguro,
            3 => _pasteLento,
            4 => _originalPasteValues ?? _pasteNormal,
            _ => _pasteNormal
        };


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

        SelectPastePreset(d);
        SelectThumbnailPreset(d);

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
            var thumbnailPreset = GetSelectedThumbnailPreset();
            var pastePreset = GetSelectedPastePreset();

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

                // Paste - apply selected preset
                DuplicateIgnoreWindowMs = pastePreset.DuplicateIgnoreMs,
                DelayBeforeFocusMs = pastePreset.DelayBeforeFocusMs,
                DelayBeforePasteMs = pastePreset.DelayBeforePasteMs,
                MaxFocusVerifyAttempts = pastePreset.MaxFocusAttempts,

                // Thumbnail - apply selected preset
                ThumbnailWidth = thumbnailPreset.Width,
                ThumbnailQualityPng = thumbnailPreset.QualityPng,
                ThumbnailQualityJpeg = thumbnailPreset.QualityJpeg,
                ThumbnailGCThreshold = thumbnailPreset.GCThreshold,
                ThumbnailUIDecodeHeight = thumbnailPreset.UIDecodeHeight
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
