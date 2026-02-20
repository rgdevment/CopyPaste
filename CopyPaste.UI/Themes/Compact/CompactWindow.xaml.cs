using CopyPaste.Core;
using CopyPaste.Core.Themes;
using CopyPaste.UI.Helpers;
using CopyPaste.UI.Localization;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using System;
using System.Collections.Generic;
using System.Globalization;
using System.Runtime.InteropServices;
using WinRT.Interop;

namespace CopyPaste.UI.Themes;

internal sealed partial class CompactWindow : Window
{
    private readonly AppWindow _appWindow;
    private readonly IntPtr _hWnd;
    private readonly MyMConfig _config;
    private readonly CompactSettings _themeSettings;
    private readonly ThemeContext _context;
    private ClipboardItemViewModel? _currentExpandedItem;
    private bool _isDialogOpen;

    internal CompactViewModel ViewModel { get; }

    [StructLayout(LayoutKind.Sequential)]
    private struct POINT
    {
        public int X;
        public int Y;
    }

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GetCursorPos(out POINT lpPoint);

    public CompactWindow(ThemeContext context, CompactSettings themeSettings)
    {
        _context = context;
        _config = context.Config;
        _themeSettings = themeSettings;

        ViewModel = new CompactViewModel(context.Service, _config, themeSettings);

        InitializeComponent();

        _hWnd = WindowNative.GetWindowHandle(this);
        _appWindow = AppWindow.GetFromWindowId(Microsoft.UI.Win32Interop.GetWindowIdFromWindow(_hWnd));

        SetWindowIcon();
        ConfigurePopupStyle();
        ViewModel.Initialize(this);
        RegisterEventHandlers();
        ApplyLocalizedStrings();
    }

    private void SetWindowIcon() => ClipboardWindowHelpers.SetWindowIcon(_appWindow);

    private void ConfigurePopupStyle()
    {
        if (_appWindow.Presenter is OverlappedPresenter presenter)
        {
            presenter.IsMaximizable = false;
            presenter.IsMinimizable = false;
            presenter.IsResizable = false;
            presenter.SetBorderAndTitleBar(false, false);
            presenter.IsAlwaysOnTop = true;
        }
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(TitleBarGrid);

        PositionAtCursor();
    }

    internal void PositionAtCursor()
    {
        int width = _themeSettings.PopupWidth;
        int height = _themeSettings.PopupHeight;

        // Get cursor position first to determine which monitor to use
        if (GetCursorPos(out var cursor))
        {
            var display = DisplayArea.GetFromPoint(
                new Windows.Graphics.PointInt32(cursor.X, cursor.Y),
                DisplayAreaFallback.Primary);
            var workArea = display.WorkArea;

            int x = cursor.X - (width / 2);
            int y = cursor.Y - height - 12;

            if (x < workArea.X) x = workArea.X + 8;
            if (x + width > workArea.X + workArea.Width) x = workArea.X + workArea.Width - width - 8;
            if (y < workArea.Y) y = workArea.Y + 8;
            if (y + height > workArea.Y + workArea.Height) y = workArea.Y + workArea.Height - height - 8;

            _appWindow.MoveAndResize(new Windows.Graphics.RectInt32(x, y, width, height));
        }
        else
        {
            // Fallback: center on primary monitor
            var display = DisplayArea.GetFromWindowId(_appWindow.Id, DisplayAreaFallback.Primary);
            var workArea = display.WorkArea;
            int x = workArea.X + (workArea.Width - width) / 2;
            int y = workArea.Y + workArea.Height - height - 48;
            _appWindow.MoveAndResize(new Windows.Graphics.RectInt32(x, y, width, height));
        }
    }

    private void RegisterEventHandlers()
    {
        Activated += Window_Activated;
        Closed += Window_Closed;
        _appWindow.Changed += AppWindow_Changed;
        ClipboardListView.Loaded += ClipboardListView_Loaded;
        SearchBox.KeyDown += SearchBox_KeyDown;
        ViewModel.OnEditRequested += ShowEditDialog;
        ViewModel.OnScrollToTopRequested += ViewModel_OnScrollToTopRequested;
    }

    private void ApplyLocalizedStrings()
    {
        SearchBox.PlaceholderText = L.Get("ui.search.placeholder", "Search...");
        EmptyStateText.Text = L.Get("ui.search.empty", "No items");

        TrayIcon.ToolTipText = L.Get("tray.tooltip");
        TrayMenuSettings.Text = L.Get("tray.settings", "Settings");
        TrayMenuExit.Text = L.Get("tray.exit");

        ToolTipService.SetToolTip(RecentTab, L.Get("ui.tab.recent", "Recent"));
        ToolTipService.SetToolTip(PinnedTab, L.Get("ui.tab.pinned", "Pinned"));
        ToolTipService.SetToolTip(SettingsButton, L.Get("ui.tooltip.settings", "Settings"));
        ToolTipService.SetToolTip(HelpButton, L.Get("ui.tooltip.help", "Help"));
        ToolTipService.SetToolTip(ReportBugButton, L.Get("ui.tooltip.reportBug", "Report a bug"));

        FilterModeContent.Text = L.Get("ui.filterMode.content", "Content");
        FilterModeCategory.Text = L.Get("ui.filterMode.category", "Category");
        FilterModeType.Text = L.Get("ui.filterMode.type", "Type");

        ColorLabelRed.Text = L.Get("clipboard.editDialog.colorRed", "Red");
        ColorLabelGreen.Text = L.Get("clipboard.editDialog.colorGreen", "Green");
        ColorLabelPurple.Text = L.Get("clipboard.editDialog.colorPurple", "Purple");
        ColorLabelYellow.Text = L.Get("clipboard.editDialog.colorYellow", "Yellow");
        ColorLabelBlue.Text = L.Get("clipboard.editDialog.colorBlue", "Blue");
        ColorLabelOrange.Text = L.Get("clipboard.editDialog.colorOrange", "Orange");

        TypeLabelText.Text = L.Get("ui.type.text", "Text");
        TypeLabelImage.Text = L.Get("ui.type.image", "Image");
        TypeLabelFile.Text = L.Get("ui.type.file", "File");
        TypeLabelFolder.Text = L.Get("ui.type.folder", "Folder");
        TypeLabelLink.Text = L.Get("ui.type.link", "Link");
        TypeLabelAudio.Text = L.Get("ui.type.audio", "Audio");
        TypeLabelVideo.Text = L.Get("ui.type.video", "Video");

        ColorPlaceholder.Text = L.Get("ui.filter.colorsPlaceholder", "Colors...");
        TypePlaceholder.Text = L.Get("ui.filter.typesPlaceholder", "Types...");
    }

    private void RecentTab_Click(object sender, RoutedEventArgs e) => SelectTab(0);

    private void PinnedTab_Click(object sender, RoutedEventArgs e) => SelectTab(1);

    private void SelectTab(int index)
    {
        ViewModel.SelectedTabIndex = index;
        UpdateTabVisuals(index);
    }

    private void UpdateTabVisuals(int activeIndex)
    {
        RecentTabIcon.Opacity = activeIndex == 0 ? 1.0 : 0.45;
        PinnedTabIcon.Opacity = activeIndex == 1 ? 1.0 : 0.45;
    }


    internal void CollapseAllCards()
    {
        foreach (var item in ViewModel.Items)
            item.Collapse();
        _currentExpandedItem = null;
    }

    private void Window_Closed(object? _, WindowEventArgs args)
    {
        CollapseAllCards();
        ViewModel.Cleanup();
        ViewModel.OnEditRequested -= ShowEditDialog;
        ViewModel.OnScrollToTopRequested -= ViewModel_OnScrollToTopRequested;

        Activated -= Window_Activated;
        _appWindow.Changed -= AppWindow_Changed;
        ClipboardListView.Loaded -= ClipboardListView_Loaded;
        SearchBox.KeyDown -= SearchBox_KeyDown;

        var scrollViewer = ClipboardWindowHelpers.FindScrollViewer(ClipboardListView);
        if (scrollViewer != null)
            scrollViewer.ViewChanged -= ScrollViewer_ViewChanged;

        foreach (var item in ViewModel.Items)
            item.ImagePathChanged -= OnImagePathChanged;

        if (Application.Current is App { IsExiting: true })
        {
            args.Handled = false;
            return;
        }

        args.Handled = true;
        _appWindow.Hide();
    }

    private void ClipboardListView_Loaded(object? _, RoutedEventArgs __)
    {
        var scrollViewer = ClipboardWindowHelpers.FindScrollViewer(ClipboardListView);
        if (scrollViewer != null)
            scrollViewer.ViewChanged += ScrollViewer_ViewChanged;
    }

    private void ScrollViewer_ViewChanged(object? _, ScrollViewerViewChangedEventArgs e)
    {
        if (_ is not ScrollViewer sv) return;

        if (sv.ScrollableHeight > 0 && sv.VerticalOffset >= sv.ScrollableHeight - _config.ScrollLoadThreshold)
            ViewModel.LoadMoreItems();
    }

    private void Window_Activated(object? _, WindowActivatedEventArgs args)
    {
        if (args.WindowActivationState == WindowActivationState.Deactivated)
        {
            if (_isDialogOpen) return;

            FocusHelper.UpdatePasteTarget(_hWnd);

            if (_themeSettings.PinWindow) return;
            if (!_themeSettings.HideOnDeactivate) return;

            CollapseAllCards();
            ViewModel.OnWindowDeactivated();
            _appWindow.Hide();
        }
        else
        {
            Win32WindowHelper.RemoveWindowBorder(_hWnd);
            if (!_themeSettings.PinWindow)
                PositionAtCursor();
            if (!_themeSettings.PinWindow && _themeSettings.ResetScrollOnShow)
                ResetScrollToTop();
            ResetFiltersOnShow();
            ViewModel.RefreshFileAvailability();
            FocusSearchBox();
        }
    }

    private void AppWindow_Changed(AppWindow sender, AppWindowChangedEventArgs args)
    {
        if ((args.DidPresenterChange || args.DidVisibilityChange) && !sender.IsVisible)
            ViewModel.OnWindowDeactivated();
    }

    private void CloseButton_Click(object sender, RoutedEventArgs e) => _appWindow.Hide();

    private void SettingsButton_Click(object sender, RoutedEventArgs e) =>
        _context.OpenSettings();

    private void HelpButton_Click(object sender, RoutedEventArgs e) =>
        _context.OpenHelp();

    private async void ReportBugButton_Click(object sender, RoutedEventArgs e) =>
        await ViewModel.OpenRepoCommand.ExecuteAsync(null);

    private void TrayMenuSettings_Click(object sender, RoutedEventArgs e) =>
        _context.OpenSettings();

    private void Card_PointerEntered(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (sender is not Border border) return;
        var timestamp = ClipboardWindowHelpers.FindChild<TextBlock>(border, "TimestampText");
        var actions = ClipboardWindowHelpers.FindChild<StackPanel>(border, "HoverActions");
        if (timestamp != null) timestamp.Opacity = 0;
        if (actions != null) actions.Opacity = 1;
    }

    private void Card_PointerExited(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (sender is not Border border) return;
        var timestamp = ClipboardWindowHelpers.FindChild<TextBlock>(border, "TimestampText");
        var actions = ClipboardWindowHelpers.FindChild<StackPanel>(border, "HoverActions");
        if (timestamp != null) timestamp.Opacity = 0.35;
        if (actions != null) actions.Opacity = 0;
    }

    private void ResetFiltersOnShow()
    {
        ViewModel.ResetFilters(
            _themeSettings.ResetFilterModeOnShow,
            _themeSettings.ResetSearchOnShow,
            _themeSettings.ResetCategoryFilterOnShow,
            _themeSettings.ResetTypeFilterOnShow);

        SyncFilterChipsState();
    }

    private void FilterModeItem_Click(object sender, RoutedEventArgs e)
    {
        if (sender is RadioMenuFlyoutItem item && int.TryParse(item.Tag?.ToString(), out var mode))
        {
            ViewModel.ActiveFilterMode = mode;
            SyncFilterChipsState();
            if (mode == 0)
                FocusSearchBox();
        }
    }

    private void ColorCheckBox_Changed(object sender, RoutedEventArgs e)
    {
        if (sender is CheckBox cb && Enum.TryParse<CardColor>(cb.Tag?.ToString(), out var color))
        {
            if (cb.IsChecked == true && !ViewModel.IsColorSelected(color))
                ViewModel.ToggleColorFilter(color);
            else if (cb.IsChecked == false && ViewModel.IsColorSelected(color))
                ViewModel.ToggleColorFilter(color);

            UpdateSelectedColorsDisplay();
        }
    }

    private void TypeCheckBox_Changed(object sender, RoutedEventArgs e)
    {
        if (sender is CheckBox cb && Enum.TryParse<ClipboardContentType>(cb.Tag?.ToString(), out var type))
        {
            if (cb.IsChecked == true && !ViewModel.IsTypeSelected(type))
                ViewModel.ToggleTypeFilter(type);
            else if (cb.IsChecked == false && ViewModel.IsTypeSelected(type))
                ViewModel.ToggleTypeFilter(type);

            UpdateSelectedTypesDisplay();
        }
    }

    private void FilterChips_KeyDown(object _, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs e)
    {
        if (e.Key == Windows.System.VirtualKey.Escape)
        {
            if (ViewModel.ActiveFilterMode == 1)
            {
                ViewModel.ClearColorFilters();
                SyncFilterChipsState();
            }
            else if (ViewModel.ActiveFilterMode == 2)
            {
                ViewModel.ClearTypeFilters();
                SyncFilterChipsState();
            }
            e.Handled = true;
        }
    }

    private void SyncFilterChipsState() =>
        ClipboardFilterChipsHelper.SyncFilterChipsState(ViewModel, (FrameworkElement)Content);

    private void UpdateSelectedColorsDisplay() =>
        ClipboardFilterChipsHelper.UpdateSelectedColorsDisplay(ViewModel, (FrameworkElement)Content);

    private void UpdateSelectedTypesDisplay() =>
        ClipboardFilterChipsHelper.UpdateSelectedTypesDisplay(ViewModel, (FrameworkElement)Content);

    private void ClipboardListView_ContainerContentChanging(ListViewBase _, ContainerContentChangingEventArgs args)
    {
        if (args.InRecycleQueue)
        {
            if (args.Item is ClipboardItemViewModel recycledVm)
                recycledVm.ImagePathChanged -= OnImagePathChanged;
            return;
        }

        var container = args.ItemContainer;
        container.Tapped -= Container_Tapped;
        container.DoubleTapped -= Container_DoubleTapped;
        container.Tapped += Container_Tapped;
        container.DoubleTapped += Container_DoubleTapped;

        if (args.Item is ClipboardItemViewModel vm)
        {
            vm.ImagePathChanged -= OnImagePathChanged;
            vm.ImagePathChanged += OnImagePathChanged;
        }

        args.RegisterUpdateCallback(LoadClipboardImage);
    }

    private void OnImagePathChanged(object? sender, EventArgs e)
    {
        if (sender is not ClipboardItemViewModel vm) return;

        var container = ClipboardListView.ContainerFromItem(vm) as ListViewItem;
        if (container?.ContentTemplateRoot is FrameworkElement root &&
            root.FindName("ClipboardImage") is Image image)
        {
            LoadImageSource(image, vm.ImagePath);
        }
    }

    private void Container_Tapped(object sender, Microsoft.UI.Xaml.Input.TappedRoutedEventArgs e)
    {
        if (sender is ListViewItem item && item.Content is ClipboardItemViewModel vm)
        {
            if (_currentExpandedItem != null && _currentExpandedItem != vm)
                _currentExpandedItem.Collapse();
            vm.ToggleExpanded();
            _currentExpandedItem = vm.IsExpanded ? vm : null;
        }
    }

    private void Container_DoubleTapped(object sender, Microsoft.UI.Xaml.Input.DoubleTappedRoutedEventArgs e)
    {
        if (sender is ListViewItem item && item.Content is ClipboardItemViewModel vm)
        {
            vm.Collapse();
            _currentExpandedItem = null;
            vm.PasteCommand.Execute(null);
        }
    }

    private void LoadClipboardImage(ListViewBase sender, ContainerContentChangingEventArgs args)
    {
        if (args.Item is not ClipboardItemViewModel vm) return;

        if (args.ItemContainer.ContentTemplateRoot is FrameworkElement root &&
            root.FindName("ClipboardImage") is Image image)
        {
            LoadImageSource(image, vm.ImagePath);
        }
    }

    private void LoadImageSource(Image image, string? imagePath) =>
        ClipboardWindowHelpers.LoadImageSource(image, imagePath, _config.ThumbnailUIDecodeHeight);

    private void SearchBox_TextChanged(object? sender, TextChangedEventArgs _)
    {
        if (SearchBox is TextBox textBox)
            ViewModel.SearchQuery = textBox.Text ?? string.Empty;
    }

    private void SearchBox_KeyDown(object sender, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs e)
    {
        if (e.Key is Windows.System.VirtualKey.Enter or Windows.System.VirtualKey.Down)
        {
            if (ClipboardListView.Items.Count > 0)
            {
                ClipboardListView.SelectedIndex = 0;

                if (e.Key == Windows.System.VirtualKey.Enter &&
                    ClipboardListView.SelectedItem is ClipboardItemViewModel vm)
                {
                    vm.PasteCommand.Execute(null);
                }
                else
                {
                    ClipboardListView.Focus(FocusState.Keyboard);
                }
                e.Handled = true;
            }
        }
    }

    private void RootGrid_PreviewKeyDown(object sender, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs e)
    {
        // ESC clears current filter
        if (e.Key == Windows.System.VirtualKey.Escape)
        {
            switch (ViewModel.ActiveFilterMode)
            {
                case 0:
                    if (!string.IsNullOrEmpty(SearchBox.Text))
                    {
                        SearchBox.Text = string.Empty;
                        e.Handled = true;
                    }
                    break;
                case 1:
                    ViewModel.ClearColorFilters();
                    SyncFilterChipsState();
                    e.Handled = true;
                    break;
                case 2:
                    ViewModel.ClearTypeFilters();
                    SyncFilterChipsState();
                    e.Handled = true;
                    break;
            }
            return;
        }

        // Filter mode switching: Alt+C (Content), Alt+G (cateGory), Alt+T (Type)
        var altPressed = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(Windows.System.VirtualKey.Menu)
            .HasFlag(Windows.UI.Core.CoreVirtualKeyStates.Down);

        if (altPressed)
        {
            if (e.Key == Windows.System.VirtualKey.C)
            {
                SetFilterMode(0);
                e.Handled = true;
                return;
            }
            if (e.Key == Windows.System.VirtualKey.G)
            {
                SetFilterMode(1);
                e.Handled = true;
                return;
            }
            if (e.Key == Windows.System.VirtualKey.T)
            {
                SetFilterMode(2);
                e.Handled = true;
                return;
            }
        }

        // Quick tab switching: Ctrl+1 (Recent), Ctrl+2 (Pinned)
        var ctrlPressed = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(Windows.System.VirtualKey.Control)
            .HasFlag(Windows.UI.Core.CoreVirtualKeyStates.Down);

        if (ctrlPressed)
        {
            if (e.Key == Windows.System.VirtualKey.Number1)
            {
                SelectTab(0);
                e.Handled = true;
            }
            else if (e.Key == Windows.System.VirtualKey.Number2)
            {
                SelectTab(1);
                e.Handled = true;
            }
        }
    }

    private void SetFilterMode(int mode)
    {
        ViewModel.ActiveFilterMode = mode;
        SyncFilterChipsState();
        if (mode == 0)
            FocusSearchBox();
    }

    private void ClipboardListView_PreviewKeyDown(object sender, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs e)
    {
        // Shift+Tab: return focus to SearchBox
        var shiftPressed = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(Windows.System.VirtualKey.Shift)
            .HasFlag(Windows.UI.Core.CoreVirtualKeyStates.Down);

        if (shiftPressed && e.Key == Windows.System.VirtualKey.Tab)
        {
            FocusSearchBox();
            e.Handled = true;
            return;
        }

        if (ClipboardListView.SelectedItem is not ClipboardItemViewModel vm) return;

        switch (e.Key)
        {
            case Windows.System.VirtualKey.Enter:
                vm.PasteCommand.Execute(null);
                e.Handled = true;
                break;

            case Windows.System.VirtualKey.Delete:
                vm.DeleteCommand.Execute(null);
                e.Handled = true;
                break;

            case Windows.System.VirtualKey.P:
                vm.TogglePinCommand.Execute(null);
                e.Handled = true;
                break;

            case Windows.System.VirtualKey.E:
                vm.EditCommand.Execute(null);
                e.Handled = true;
                break;

            case Windows.System.VirtualKey.Right:
                if (_currentExpandedItem != null && _currentExpandedItem != vm)
                    _currentExpandedItem.Collapse();
                vm.ToggleExpanded();
                _currentExpandedItem = vm.IsExpanded ? vm : null;
                e.Handled = true;
                break;
        }
    }

    private void FocusSearchBox() =>
        DispatcherQueue.TryEnqueue(Microsoft.UI.Dispatching.DispatcherQueuePriority.Low, () =>
            SearchBox.Focus(FocusState.Programmatic));

    private void ResetScrollToTop()
    {
        var scrollViewer = ClipboardWindowHelpers.FindScrollViewer(ClipboardListView);
        scrollViewer?.ChangeView(null, 0, null, disableAnimation: true);

        if (ClipboardListView.Items.Count > 0)
            ClipboardListView.SelectedIndex = 0;
    }

    private void ViewModel_OnScrollToTopRequested(object? sender, EventArgs e)
    {
        if (!_themeSettings.ScrollToTopOnPaste) return;

        DispatcherQueue.TryEnqueue(() =>
        {
            var scrollViewer = ClipboardWindowHelpers.FindScrollViewer(ClipboardListView);
            scrollViewer?.ChangeView(null, 0, null, disableAnimation: false);

            if (ClipboardListView.Items.Count > 0)
                ClipboardListView.SelectedIndex = 0;
        });
    }

    private async void ShowEditDialog(object? sender, ClipboardItemViewModel itemVM)
    {
        _isDialogOpen = true;
        try
        {
            var result = await ClipboardWindowHelpers.ShowEditDialogAsync(Content.XamlRoot, itemVM);
            if (result is { } r)
                ViewModel.SaveItemLabelAndColor(itemVM, r.label, r.color);
        }
        finally
        {
            _isDialogOpen = false;
        }
    }
}
