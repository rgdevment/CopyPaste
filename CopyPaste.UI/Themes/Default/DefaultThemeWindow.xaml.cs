using CopyPaste.Core;
using CopyPaste.Core.Themes;
using CopyPaste.UI.Helpers;
using CopyPaste.UI.Localization;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using System;
using System.Collections.Generic;
using Windows.UI;
using WinRT.Interop;

namespace CopyPaste.UI.Themes;

internal sealed partial class DefaultThemeWindow : Window
{
    public DefaultThemeViewModel ViewModel { get; }
    private readonly AppWindow _appWindow;
    private readonly IntPtr _hWnd;
    private readonly MyMConfig _config;
    private readonly DefaultThemeSettings _themeSettings;
    private readonly ThemeContext _context;
    private ClipboardItemViewModel? _currentExpandedItem;
    private ClipboardItemViewModel? _previousSelectedItem;
    private bool _isDialogOpen;

    private static readonly SolidColorBrush _orangeBrush = new(Color.FromArgb(255, 210, 120, 50));
    private static readonly SolidColorBrush _blueBrush = new(Color.FromArgb(255, 91, 155, 213));
    private static readonly SolidColorBrush _inactiveBrush = new(Color.FromArgb(255, 120, 120, 120));

    public DefaultThemeWindow(ThemeContext context, DefaultThemeSettings themeSettings)
    {
        _context = context;
        _config = context.Config;
        _themeSettings = themeSettings;
        ViewModel = new DefaultThemeViewModel(context.Service, _config, _themeSettings);
        ViewModel.Initialize(this);
        InitializeComponent();

        _hWnd = WindowNative.GetWindowHandle(this);
        _appWindow = AppWindow.GetFromWindowId(Win32Interop.GetWindowIdFromWindow(_hWnd));

        InitializeWindow();
        RegisterEventHandlers();
        ApplyLocalizedStrings();
        ApplyPinWindowVisuals();
    }

    private void InitializeWindow()
    {
        SetWindowIcon();
        ConfigureSidebarStyle();
        Win32WindowHelper.RemoveWindowBorder(_hWnd);
    }

    private void ApplyPinWindowVisuals() =>
        HideWindowButton.Visibility = _themeSettings.PinWindow
            ? Visibility.Visible
            : Visibility.Collapsed;

    private void HideWindowButton_Click(object sender, RoutedEventArgs e)
    {
        CollapseAllCards();
        ViewModel.OnWindowDeactivated();
        _appWindow.Hide();
    }

    private void ApplyLocalizedStrings()
    {
        TrayIcon.ToolTipText = L.Get("tray.tooltip");
        TrayMenuSettings.Text = L.Get("tray.settings", "Settings");
        TrayMenuExit.Text = L.Get("tray.exit");
        ToolTipService.SetToolTip(HideWindowButton, L.Get("ui.sidebar.hide", "Hide"));
        ToolTipService.SetToolTip(RecentTab, L.Get("ui.section.recent"));
        ToolTipService.SetToolTip(PinnedTab, L.Get("ui.section.pinned"));
        ToolTipService.SetToolTip(HelpButton, L.Get("ui.sidebar.help"));
        ToolTipService.SetToolTip(SettingsButton, L.Get("ui.sidebar.settings"));
        ToolTipService.SetToolTip(ReportButton, L.Get("ui.sidebar.report"));
        SearchBox.PlaceholderText = L.Get("ui.search.placeholder");
        FilterModeContent.Text = L.Get("ui.filters.modeContent");
        FilterModeCategory.Text = L.Get("ui.filters.modeCategory");
        FilterModeType.Text = L.Get("ui.filters.modeType");
        EmptyStateText.Text = L.Get("ui.emptyState");
        SectionTitle.Text = L.Get("ui.section.recent");

        // Color filter dropdown
        ColorPlaceholder.Text = L.Get("ui.filters.selectColors");
        ColorLabelRed.Text = GetColorLabelWithFallback("Red", "clipboard.editDialog.colorRed");
        ColorLabelGreen.Text = GetColorLabelWithFallback("Green", "clipboard.editDialog.colorGreen");
        ColorLabelPurple.Text = GetColorLabelWithFallback("Purple", "clipboard.editDialog.colorPurple");
        ColorLabelYellow.Text = GetColorLabelWithFallback("Yellow", "clipboard.editDialog.colorYellow");
        ColorLabelBlue.Text = GetColorLabelWithFallback("Blue", "clipboard.editDialog.colorBlue");
        ColorLabelOrange.Text = GetColorLabelWithFallback("Orange", "clipboard.editDialog.colorOrange");

        // Type filter dropdown
        TypePlaceholder.Text = L.Get("ui.filters.selectTypes");
        TypeLabelText.Text = L.Get("clipboard.itemTypes.text");
        TypeLabelImage.Text = L.Get("clipboard.itemTypes.image");
        TypeLabelFile.Text = L.Get("clipboard.itemTypes.file");
        TypeLabelFolder.Text = L.Get("clipboard.itemTypes.folder");
        TypeLabelLink.Text = L.Get("clipboard.itemTypes.link");
        TypeLabelAudio.Text = L.Get("clipboard.itemTypes.audio");
        TypeLabelVideo.Text = L.Get("clipboard.itemTypes.video");
    }

    private string GetColorLabelWithFallback(string colorName, string localizationKey) =>
        _config.ColorLabels?.TryGetValue(colorName, out var label) == true && !string.IsNullOrWhiteSpace(label)
            ? label
            : L.Get(localizationKey);

    private void RegisterEventHandlers()
    {
        Activated += Window_Activated;
        Closed += Window_Closed;
        _appWindow.Changed += AppWindow_Changed;
        ClipboardListView.Loaded += ClipboardListView_Loaded;
        ClipboardListView.SelectionChanged += ClipboardListView_SelectionChanged;
        SearchBox.KeyDown += SearchBox_KeyDown;
        ViewModel.OnEditRequested += ShowEditDialog;
        ViewModel.OnScrollToTopRequested += ViewModel_OnScrollToTopRequested;
    }

    private void TabChanged(object sender, RoutedEventArgs e)
    {
        if (sender is RadioButton rb)
        {
            ViewModel.SelectedTabIndex = rb.Name switch
            {
                nameof(RecentTab) => 0,
                nameof(PinnedTab) => 1,
                _ => 0
            };

            UpdateSectionIndicator(rb.Name);
        }
    }

    private void UpdateSectionIndicator(string? tabName)
    {
        if (SectionTitle == null || SectionIcon == null)
            return;

        if (tabName == nameof(PinnedTab))
        {
            SectionTitle.Text = L.Get("ui.section.pinned");
            SectionIcon.Glyph = "\uE718";
            SectionTitle.Foreground = _orangeBrush;
            SectionIcon.Foreground = _orangeBrush;

            if (PinnedTabIcon != null) PinnedTabIcon.Foreground = _orangeBrush;
            if (RecentTabIcon != null) RecentTabIcon.Foreground = _inactiveBrush;
        }
        else
        {
            SectionTitle.Text = L.Get("ui.section.recent");
            SectionIcon.Glyph = "\uE823";
            SectionTitle.Foreground = _blueBrush;
            SectionIcon.Foreground = _blueBrush;

            if (RecentTabIcon != null) RecentTabIcon.Foreground = _blueBrush;
            if (PinnedTabIcon != null) PinnedTabIcon.Foreground = _inactiveBrush;
        }
    }

    internal void CollapseAllCards()
    {
        foreach (var item in ViewModel.Items)
        {
            item.Collapse();
        }
        _currentExpandedItem = null;
        _previousSelectedItem = null;
    }

    private void Window_Closed(object? _, WindowEventArgs args)
    {
        CollapseAllCards();

        ViewModel.Cleanup();
        ViewModel.OnEditRequested -= ShowEditDialog;
        ViewModel.OnScrollToTopRequested -= ViewModel_OnScrollToTopRequested;

        // Unsubscribe from framework events
        Activated -= Window_Activated;
        _appWindow.Changed -= AppWindow_Changed;
        ClipboardListView.Loaded -= ClipboardListView_Loaded;
        ClipboardListView.SelectionChanged -= ClipboardListView_SelectionChanged;
        SearchBox.KeyDown -= SearchBox_KeyDown;

        // Unsubscribe from ScrollViewer
        var scrollViewer = ClipboardWindowHelpers.FindScrollViewer(ClipboardListView);
        if (scrollViewer != null)
            scrollViewer.ViewChanged -= ScrollViewer_ViewChanged;

        // Unsubscribe from all items' ImagePathChanged events
        foreach (var item in ViewModel.Items)
        {
            item.ImagePathChanged -= OnImagePathChanged;
        }

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

            CollapseAllCards();
            ViewModel.OnWindowDeactivated();
            _appWindow.Hide();
        }
        else
        {
            Win32WindowHelper.RemoveWindowBorder(_hWnd);
            ApplyPinWindowVisuals();
            if (!_themeSettings.PinWindow && _themeSettings.ResetScrollOnShow)
                ResetScrollToTop();
            ResetFiltersOnShow();
            ViewModel.RefreshFileAvailability();
            FocusSearchBox();
        }
    }

    private void ConfigureSidebarStyle()
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
        MoveToRightEdge();
    }

    private void SetWindowIcon() => ClipboardWindowHelpers.SetWindowIcon(_appWindow);

    private void MoveToRightEdge()
    {
        var workArea = DisplayArea.GetFromWindowId(_appWindow.Id, DisplayAreaFallback.Primary).WorkArea;
        int width = _themeSettings.WindowWidth;
        int height = workArea.Height - _themeSettings.WindowMarginBottom;
        int x = workArea.X + workArea.Width - width;
        int y = workArea.Y + _themeSettings.WindowMarginTop;
        _appWindow.MoveAndResize(new Windows.Graphics.RectInt32(x, y, width, height));
    }

    private void AppWindow_Changed(AppWindow sender, AppWindowChangedEventArgs args)
    {
        if ((args.DidPresenterChange || args.DidVisibilityChange) && !sender.IsVisible)
            ViewModel.OnWindowDeactivated();
    }

    private static void Container_PointerEntered(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (sender is ListViewItem item && item.ContentTemplateRoot is FrameworkElement root)
        {
            if (ClipboardWindowHelpers.FindDescendant(root, "ActionPanel") is UIElement panel)
                panel.Opacity = 1;
            if (ClipboardWindowHelpers.FindDescendant(root, "LabelTimestamp") is UIElement timestamp)
                timestamp.Opacity = 0;
            if (ClipboardWindowHelpers.FindDescendant(root, "ImageBorder") is UIElement imageBorder)
                imageBorder.Opacity = 1;
            if (ClipboardWindowHelpers.FindDescendant(root, "MediaBorder") is UIElement mediaBorder)
                mediaBorder.Opacity = 1;
        }
    }

    private static void Container_PointerExited(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (sender is ListViewItem item && item.ContentTemplateRoot is FrameworkElement root)
        {
            if (ClipboardWindowHelpers.FindDescendant(root, "ActionPanel") is UIElement panel)
                panel.Opacity = 0;
            if (ClipboardWindowHelpers.FindDescendant(root, "LabelTimestamp") is UIElement timestamp)
                timestamp.Opacity = 0.5;
            if (ClipboardWindowHelpers.FindDescendant(root, "ImageBorder") is UIElement imageBorder)
                imageBorder.Opacity = 0.6;
            if (ClipboardWindowHelpers.FindDescendant(root, "MediaBorder") is UIElement mediaBorder)
                mediaBorder.Opacity = 0.6;
        }
    }

    private void ClipboardListView_ContainerContentChanging(ListViewBase _, ContainerContentChangingEventArgs args)
    {
        if (args.InRecycleQueue)
        {
            // Unsubscribe when recycling
            if (args.Item is ClipboardItemViewModel recycledVm)
            {
                recycledVm.ImagePathChanged -= OnImagePathChanged;
            }
            return;
        }

        var container = args.ItemContainer;
        container.PointerEntered -= Container_PointerEntered;
        container.PointerExited -= Container_PointerExited;
        container.Tapped -= Container_Tapped;
        container.DoubleTapped -= Container_DoubleTapped;
        container.PointerEntered += Container_PointerEntered;
        container.PointerExited += Container_PointerExited;
        container.Tapped += Container_Tapped;
        container.DoubleTapped += Container_DoubleTapped;

        // Subscribe to image path changes for live thumbnail updates
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

        // Find the container for this item and reload its image
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

    private void ResetFiltersOnShow()
    {
        ViewModel.ResetFilters(
            _themeSettings.ResetFilterModeOnShow,
            _themeSettings.ResetContentFilterOnShow,
            _themeSettings.ResetCategoryFilterOnShow,
            _themeSettings.ResetTypeFilterOnShow);

        // Sync UI state with ViewModel
        SyncFilterChipsState();
    }

    private void FilterModeItem_Click(object sender, RoutedEventArgs e)
    {
        if (sender is RadioMenuFlyoutItem item && int.TryParse(item.Tag?.ToString(), out var mode))
        {
            ViewModel.ActiveFilterMode = mode;
            SyncFilterChipsState();
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

    private void SyncFilterChipsState() =>
        ClipboardFilterChipsHelper.SyncFilterChipsState(ViewModel, (FrameworkElement)Content);

    private void UpdateSelectedColorsDisplay() =>
        ClipboardFilterChipsHelper.UpdateSelectedColorsDisplay(ViewModel, (FrameworkElement)Content);

    private void UpdateSelectedTypesDisplay() =>
        ClipboardFilterChipsHelper.UpdateSelectedTypesDisplay(ViewModel, (FrameworkElement)Content);

    private void MainContent_PreviewKeyDown(object sender, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs e)
    {
        // ESC clears current filter
        if (e.Key == Windows.System.VirtualKey.Escape)
        {
            switch (ViewModel.ActiveFilterMode)
            {
                case 0: // Content - clear search
                    if (!string.IsNullOrEmpty(SearchBox.Text))
                    {
                        SearchBox.Text = string.Empty;
                        e.Handled = true;
                    }
                    break;
                case 1: // Category - clear color filters
                    ViewModel.ClearColorFilters();
                    SyncFilterChipsState();
                    e.Handled = true;
                    break;
                case 2: // Type - clear type filters
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
    }

    private void SetFilterMode(int mode)
    {
        ViewModel.ActiveFilterMode = mode;
        SyncFilterChipsState();
        if (mode == 0)
            FocusSearchBox();
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

    private void SearchBox_KeyDown(object sender, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs e)
    {
        var ctrlPressed = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(Windows.System.VirtualKey.Control)
            .HasFlag(Windows.UI.Core.CoreVirtualKeyStates.Down);

        // Quick tab switching: Ctrl+1 (Recent), Ctrl+2 (Pinned)
        if (ctrlPressed)
        {
            if (e.Key == Windows.System.VirtualKey.Number1)
            {
                RecentTab.IsChecked = true;
                e.Handled = true;
                return;
            }
            if (e.Key == Windows.System.VirtualKey.Number2)
            {
                PinnedTab.IsChecked = true;
                e.Handled = true;
                return;
            }
        }

        // Navigate to ListView when pressing Enter or Down arrow in SearchBox
        if (e.Key is Windows.System.VirtualKey.Enter or Windows.System.VirtualKey.Down)
        {
            if (ClipboardListView.Items.Count > 0)
            {
                ClipboardListView.SelectedIndex = 0;

                // If Enter was pressed and there's an item, paste it directly (same as double-click)
                if (e.Key == Windows.System.VirtualKey.Enter &&
                    ClipboardListView.SelectedItem is ClipboardItemViewModel vm)
                {
                    vm.PasteCommand.Execute(null);
                }
                else
                {
                    // Just navigate to list with Down arrow
                    ClipboardListView.Focus(FocusState.Keyboard);
                }

                e.Handled = true;
            }
        }
    }

    private void ClipboardListView_PreviewKeyDown(object sender, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs e)
    {
        var ctrlPressed = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(Windows.System.VirtualKey.Control)
            .HasFlag(Windows.UI.Core.CoreVirtualKeyStates.Down);

        // Quick tab switching: Ctrl+1 (Recent), Ctrl+2 (Pinned)
        if (ctrlPressed)
        {
            if (e.Key == Windows.System.VirtualKey.Number1)
            {
                RecentTab.IsChecked = true;
                e.Handled = true;
                return;
            }
            if (e.Key == Windows.System.VirtualKey.Number2)
            {
                PinnedTab.IsChecked = true;
                e.Handled = true;
                return;
            }
        }

        if (ClipboardListView.SelectedItem is not ClipboardItemViewModel vm)
            return;

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
                {
                    _currentExpandedItem.Collapse();
                }
                vm.ToggleExpanded();
                _currentExpandedItem = vm.IsExpanded ? vm : null;
                e.Handled = true;
                break;
        }
    }

    private void ClipboardListView_SelectionChanged(object sender, Microsoft.UI.Xaml.Controls.SelectionChangedEventArgs e)
    {
        // Dim previous images/media
        if (e.RemovedItems.Count > 0)
        {
            foreach (var removed in e.RemovedItems)
            {
                var removedContainer = ClipboardListView.ContainerFromItem(removed) as ListViewItem;
                if (removedContainer?.ContentTemplateRoot is FrameworkElement removedRoot)
                {
                    if (ClipboardWindowHelpers.FindDescendant(removedRoot, "ImageBorder") is UIElement imageBorder)
                        imageBorder.Opacity = 0.6;
                    if (ClipboardWindowHelpers.FindDescendant(removedRoot, "MediaBorder") is UIElement mediaBorder)
                        mediaBorder.Opacity = 0.6;
                }
            }
        }

        // Highlight current images/media
        if (e.AddedItems.Count > 0)
        {
            foreach (var added in e.AddedItems)
            {
                var addedContainer = ClipboardListView.ContainerFromItem(added) as ListViewItem;
                if (addedContainer?.ContentTemplateRoot is FrameworkElement addedRoot)
                {
                    if (ClipboardWindowHelpers.FindDescendant(addedRoot, "ImageBorder") is UIElement imageBorder)
                        imageBorder.Opacity = 1;
                    if (ClipboardWindowHelpers.FindDescendant(addedRoot, "MediaBorder") is UIElement mediaBorder)
                        mediaBorder.Opacity = 1;
                }
            }
        }

        // Auto-collapse previous card on navigation
        if (ClipboardListView.SelectedItem is ClipboardItemViewModel currentVm)
        {
            if (_previousSelectedItem != null && _previousSelectedItem != currentVm && _previousSelectedItem.IsExpanded)
            {
                _previousSelectedItem.Collapse();
                if (_currentExpandedItem == _previousSelectedItem)
                {
                    _currentExpandedItem = null;
                }
            }
            _previousSelectedItem = currentVm;
        }
    }

    private void FocusSearchBox() =>
        // Use dispatcher to ensure focus happens after window is fully rendered
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

    private void OpenHelp_Click(object sender, RoutedEventArgs e) =>
        _context.OpenHelp();

    private void OpenSettings_Click(object sender, RoutedEventArgs e) =>
        _context.OpenSettings();

    private void TrayMenuSettings_Click(object sender, RoutedEventArgs e) =>
        _context.OpenSettings();

    private async void ShowEditDialog(object? sender, ClipboardItemViewModel itemVM)
    {
        _isDialogOpen = true;
        try
        {
            var result = await ClipboardWindowHelpers.ShowEditDialogAsync(
                Content.XamlRoot, itemVM, GetColorLabelWithFallback);
            if (result is { } r)
                ViewModel.SaveItemLabelAndColor(itemVM, r.label, r.color);
        }
        finally
        {
            _isDialogOpen = false;
        }
    }
}
