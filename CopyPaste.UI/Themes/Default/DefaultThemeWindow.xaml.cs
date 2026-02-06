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
using System.Linq;
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
    }

    private void InitializeWindow()
    {
        SetWindowIcon();
        ConfigureSidebarStyle();
        Win32WindowHelper.RemoveWindowBorder(_hWnd);
    }

    private void ApplyLocalizedStrings()
    {
        TrayIcon.ToolTipText = L.Get("tray.tooltip");
        TrayMenuExit.Text = L.Get("tray.exit");
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

        // Unsubscribe from framework events
        Activated -= Window_Activated;
        _appWindow.Changed -= AppWindow_Changed;
        ClipboardListView.Loaded -= ClipboardListView_Loaded;
        ClipboardListView.SelectionChanged -= ClipboardListView_SelectionChanged;
        SearchBox.KeyDown -= SearchBox_KeyDown;

        // Unsubscribe from ScrollViewer
        var scrollViewer = FindScrollViewer(ClipboardListView);
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
        var scrollViewer = FindScrollViewer(ClipboardListView);
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

            CollapseAllCards();
            ViewModel.OnWindowDeactivated();
            _appWindow.Hide();
        }
        else
        {
            Win32WindowHelper.RemoveWindowBorder(_hWnd);
            if (_themeSettings.ResetScrollOnShow)
                ResetScrollToTop();
            ResetFiltersOnShow();
            RefreshFileAvailability();
            FocusSearchBox();
        }
    }

    private void RefreshFileAvailability()
    {
        foreach (var item in ViewModel.Items.Where(i => i.IsFileType))
        {
            item.RefreshFileStatus();
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

    private void SetWindowIcon()
    {
        var iconPath = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "CopyPasteLogoSimple.ico");
        if (System.IO.File.Exists(iconPath))
            _appWindow.SetIcon(iconPath);
    }

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

    private static ScrollViewer? FindScrollViewer(DependencyObject parent)
    {
        for (int i = 0; i < VisualTreeHelper.GetChildrenCount(parent); i++)
        {
            var child = VisualTreeHelper.GetChild(parent, i);
            if (child is ScrollViewer sv)
                return sv;

            var result = FindScrollViewer(child);
            if (result != null)
                return result;
        }
        return null;
    }

    private static void Container_PointerEntered(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (sender is ListViewItem item && item.ContentTemplateRoot is FrameworkElement root)
        {
            if (FindDescendant(root, "ActionPanel") is UIElement panel)
                panel.Opacity = 1;
            if (FindDescendant(root, "LabelTimestamp") is UIElement timestamp)
                timestamp.Opacity = 0;
            if (FindDescendant(root, "ImageBorder") is UIElement imageBorder)
                imageBorder.Opacity = 1;
            if (FindDescendant(root, "MediaBorder") is UIElement mediaBorder)
                mediaBorder.Opacity = 1;
        }
    }

    private static void Container_PointerExited(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (sender is ListViewItem item && item.ContentTemplateRoot is FrameworkElement root)
        {
            if (FindDescendant(root, "ActionPanel") is UIElement panel)
                panel.Opacity = 0;
            if (FindDescendant(root, "LabelTimestamp") is UIElement timestamp)
                timestamp.Opacity = 0.5;
            if (FindDescendant(root, "ImageBorder") is UIElement imageBorder)
                imageBorder.Opacity = 0.6;
            if (FindDescendant(root, "MediaBorder") is UIElement mediaBorder)
                mediaBorder.Opacity = 0.6;
        }
    }

    private static DependencyObject? FindDescendant(DependencyObject parent, string name)
    {
        if (parent is FrameworkElement fe && fe.Name == name)
            return parent;

        for (int i = 0; i < VisualTreeHelper.GetChildrenCount(parent); i++)
        {
            var child = VisualTreeHelper.GetChild(parent, i);
            var result = FindDescendant(child, name);
            if (result != null)
                return result;
        }
        return null;
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
        container.DoubleTapped -= Container_DoubleTapped;
        container.Tapped -= Container_Tapped;
        container.PointerEntered += Container_PointerEntered;
        container.PointerExited += Container_PointerExited;
        container.DoubleTapped += Container_DoubleTapped;
        container.Tapped += Container_Tapped;

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
            {
                _currentExpandedItem.Collapse();
            }

            vm.ToggleExpanded();
            _currentExpandedItem = vm.IsExpanded ? vm : null;

            e.Handled = true;
        }
    }

    private void Container_DoubleTapped(object sender, Microsoft.UI.Xaml.Input.DoubleTappedRoutedEventArgs e)
    {
        if (sender is ListViewItem item && item.Content is ClipboardItemViewModel vm)
        {
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

    private void LoadImageSource(Image image, string? imagePath)
    {
        if (string.IsNullOrEmpty(imagePath)) return;

        if (!imagePath.StartsWith("ms-appx://", StringComparison.OrdinalIgnoreCase) &&
            !System.IO.File.Exists(imagePath))
        {
            // Check if it's a thumbnail file (could be _t.png, _t.jpg, _t.webp, etc.)
            if (imagePath.Contains("_t.", StringComparison.Ordinal))
            {
                try
                {
                    image.Source = new Microsoft.UI.Xaml.Media.Imaging.BitmapImage
                    {
                        UriSource = new Uri("ms-appx:///Assets/thumb/image.png")
                    };
                }
                catch { /* Silently fail */ }
            }
            return;
        }

        // Skip if already showing the same image (avoid creating duplicate BitmapImages)
        if (image.Source is Microsoft.UI.Xaml.Media.Imaging.BitmapImage currentBitmap)
        {
            var currentPath = currentBitmap.UriSource?.LocalPath;
            if (currentPath != null && imagePath.EndsWith(System.IO.Path.GetFileName(currentPath), StringComparison.OrdinalIgnoreCase))
                return;
        }

        try
        {
            image.Source = new Microsoft.UI.Xaml.Media.Imaging.BitmapImage
            {
                UriSource = new Uri(imagePath),
                CreateOptions = Microsoft.UI.Xaml.Media.Imaging.BitmapCreateOptions.None,
                DecodePixelHeight = _config.ThumbnailUIDecodeHeight
            };
        }
        catch { /* Silently fail */ }
    }

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

    private void SyncFilterChipsState()
    {
        // Sync color checkboxes
        ColorCheckRed.IsChecked = ViewModel.IsColorSelected(CardColor.Red);
        ColorCheckGreen.IsChecked = ViewModel.IsColorSelected(CardColor.Green);
        ColorCheckPurple.IsChecked = ViewModel.IsColorSelected(CardColor.Purple);
        ColorCheckYellow.IsChecked = ViewModel.IsColorSelected(CardColor.Yellow);
        ColorCheckBlue.IsChecked = ViewModel.IsColorSelected(CardColor.Blue);
        ColorCheckOrange.IsChecked = ViewModel.IsColorSelected(CardColor.Orange);

        // Sync type checkboxes
        TypeCheckText.IsChecked = ViewModel.IsTypeSelected(ClipboardContentType.Text);
        TypeCheckImage.IsChecked = ViewModel.IsTypeSelected(ClipboardContentType.Image);
        TypeCheckFile.IsChecked = ViewModel.IsTypeSelected(ClipboardContentType.File);
        TypeCheckFolder.IsChecked = ViewModel.IsTypeSelected(ClipboardContentType.Folder);
        TypeCheckLink.IsChecked = ViewModel.IsTypeSelected(ClipboardContentType.Link);
        TypeCheckAudio.IsChecked = ViewModel.IsTypeSelected(ClipboardContentType.Audio);
        TypeCheckVideo.IsChecked = ViewModel.IsTypeSelected(ClipboardContentType.Video);

        // Sync filter mode menu
        FilterModeContent.IsChecked = ViewModel.ActiveFilterMode == 0;
        FilterModeCategory.IsChecked = ViewModel.ActiveFilterMode == 1;
        FilterModeType.IsChecked = ViewModel.ActiveFilterMode == 2;

        // Update visual displays
        UpdateSelectedColorsDisplay();
        UpdateSelectedTypesDisplay();
    }

    private void UpdateSelectedColorsDisplay()
    {
        // Clear existing chips (except placeholder)
        while (SelectedColorsPanel.Children.Count > 1)
            SelectedColorsPanel.Children.RemoveAt(1);

        var selectedColors = new List<(CardColor color, string hex)>();
        if (ViewModel.IsColorSelected(CardColor.Red)) selectedColors.Add((CardColor.Red, "#E74C3C"));
        if (ViewModel.IsColorSelected(CardColor.Green)) selectedColors.Add((CardColor.Green, "#2ECC71"));
        if (ViewModel.IsColorSelected(CardColor.Purple)) selectedColors.Add((CardColor.Purple, "#9B59B6"));
        if (ViewModel.IsColorSelected(CardColor.Yellow)) selectedColors.Add((CardColor.Yellow, "#F1C40F"));
        if (ViewModel.IsColorSelected(CardColor.Blue)) selectedColors.Add((CardColor.Blue, "#3498DB"));
        if (ViewModel.IsColorSelected(CardColor.Orange)) selectedColors.Add((CardColor.Orange, "#E67E22"));

        if (selectedColors.Count == 0)
        {
            ColorPlaceholder.Visibility = Visibility.Visible;
        }
        else
        {
            ColorPlaceholder.Visibility = Visibility.Collapsed;
            foreach (var (_, hex) in selectedColors)
            {
                // Circular color indicator with better visibility
                var chip = new Ellipse
                {
                    Width = 16,
                    Height = 16,
                    Fill = new Microsoft.UI.Xaml.Media.SolidColorBrush(ParseColor(hex)),
                    Stroke = new Microsoft.UI.Xaml.Media.SolidColorBrush(Windows.UI.Color.FromArgb(40, 0, 0, 0)),
                    StrokeThickness = 1,
                    Margin = new Thickness(0, 0, 2, 0)
                };
                SelectedColorsPanel.Children.Add(chip);
            }
        }
    }

    private void UpdateSelectedTypesDisplay()
    {
        // Clear existing chips (except placeholder)
        while (SelectedTypesPanel.Children.Count > 1)
            SelectedTypesPanel.Children.RemoveAt(1);

        var selectedTypes = new List<(ClipboardContentType type, string glyph)>();
        if (ViewModel.IsTypeSelected(ClipboardContentType.Text)) selectedTypes.Add((ClipboardContentType.Text, "\uE8C1"));
        if (ViewModel.IsTypeSelected(ClipboardContentType.Image)) selectedTypes.Add((ClipboardContentType.Image, "\uE91B"));
        if (ViewModel.IsTypeSelected(ClipboardContentType.File)) selectedTypes.Add((ClipboardContentType.File, "\uE7C3"));
        if (ViewModel.IsTypeSelected(ClipboardContentType.Folder)) selectedTypes.Add((ClipboardContentType.Folder, "\uE8B7"));
        if (ViewModel.IsTypeSelected(ClipboardContentType.Link)) selectedTypes.Add((ClipboardContentType.Link, "\uE71B"));
        if (ViewModel.IsTypeSelected(ClipboardContentType.Audio)) selectedTypes.Add((ClipboardContentType.Audio, "\uE8D6"));
        if (ViewModel.IsTypeSelected(ClipboardContentType.Video)) selectedTypes.Add((ClipboardContentType.Video, "\uE714"));

        if (selectedTypes.Count == 0)
        {
            TypePlaceholder.Visibility = Visibility.Visible;
        }
        else
        {
            TypePlaceholder.Visibility = Visibility.Collapsed;
            var maxToShow = 5;
            var shown = 0;
            foreach (var (_, glyph) in selectedTypes)
            {
                if (shown >= maxToShow)
                {
                    var moreText = new TextBlock
                    {
                        Text = $"+{selectedTypes.Count - maxToShow}",
                        FontSize = 10,
                        Opacity = 0.6,
                        VerticalAlignment = VerticalAlignment.Center,
                        Margin = new Thickness(4, 0, 0, 0)
                    };
                    SelectedTypesPanel.Children.Add(moreText);
                    break;
                }
                // Icon chip with background for better distinction
                var chipBorder = new Border
                {
                    Background = new Microsoft.UI.Xaml.Media.SolidColorBrush(Windows.UI.Color.FromArgb(30, 128, 128, 128)),
                    CornerRadius = new CornerRadius(4),
                    Padding = new Thickness(6, 3, 6, 3),
                    Child = new FontIcon
                    {
                        Glyph = glyph,
                        FontSize = 12
                    }
                };
                SelectedTypesPanel.Children.Add(chipBorder);
                shown++;
            }
        }
    }

    private static Windows.UI.Color ParseColor(string hex)
    {
        hex = hex.TrimStart('#');
        return Windows.UI.Color.FromArgb(
            255,
            byte.Parse(hex.AsSpan(0, 2), System.Globalization.NumberStyles.HexNumber, System.Globalization.CultureInfo.InvariantCulture),
            byte.Parse(hex.AsSpan(2, 2), System.Globalization.NumberStyles.HexNumber, System.Globalization.CultureInfo.InvariantCulture),
            byte.Parse(hex.AsSpan(4, 2), System.Globalization.NumberStyles.HexNumber, System.Globalization.CultureInfo.InvariantCulture)
        );
    }

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
                    if (FindDescendant(removedRoot, "ImageBorder") is UIElement imageBorder)
                        imageBorder.Opacity = 0.6;
                    if (FindDescendant(removedRoot, "MediaBorder") is UIElement mediaBorder)
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
                    if (FindDescendant(addedRoot, "ImageBorder") is UIElement imageBorder)
                        imageBorder.Opacity = 1;
                    if (FindDescendant(addedRoot, "MediaBorder") is UIElement mediaBorder)
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
        var scrollViewer = FindScrollViewer(ClipboardListView);
        scrollViewer?.ChangeView(null, 0, null, disableAnimation: true);

        if (ClipboardListView.Items.Count > 0)
            ClipboardListView.SelectedIndex = 0;
    }

    private void OpenHelp_Click(object sender, RoutedEventArgs e) =>
        _context.OpenHelp();

    private void OpenSettings_Click(object sender, RoutedEventArgs e) =>
        _context.OpenSettings();

    private async void ShowEditDialog(object? sender, ClipboardItemViewModel itemVM)
    {
        _isDialogOpen = true;

        try
        {
            var labelBox = new TextBox
            {
                Text = itemVM.Label ?? string.Empty,
                PlaceholderText = L.Get("clipboard.editDialog.labelPlaceholder"),
                MaxLength = ClipboardItem.MaxLabelLength,
                HorizontalAlignment = HorizontalAlignment.Stretch
            };

            var colorPanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, Margin = new Thickness(0, 12, 0, 0) };
            var colorLabel = new TextBlock
            {
                Text = L.Get("clipboard.editDialog.colorLabel"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 8, 0),
                Opacity = 0.7
            };
            colorPanel.Children.Add(colorLabel);

            var colorCombo = new ComboBox { Width = 140 };
            colorCombo.Items.Add(new ComboBoxItem { Content = L.Get("clipboard.editDialog.colorNone"), Tag = CardColor.None });
            colorCombo.Items.Add(new ComboBoxItem { Content = GetColorLabelWithFallback("Red", "clipboard.editDialog.colorRed"), Tag = CardColor.Red });
            colorCombo.Items.Add(new ComboBoxItem { Content = GetColorLabelWithFallback("Green", "clipboard.editDialog.colorGreen"), Tag = CardColor.Green });
            colorCombo.Items.Add(new ComboBoxItem { Content = GetColorLabelWithFallback("Purple", "clipboard.editDialog.colorPurple"), Tag = CardColor.Purple });
            colorCombo.Items.Add(new ComboBoxItem { Content = GetColorLabelWithFallback("Yellow", "clipboard.editDialog.colorYellow"), Tag = CardColor.Yellow });
            colorCombo.Items.Add(new ComboBoxItem { Content = GetColorLabelWithFallback("Blue", "clipboard.editDialog.colorBlue"), Tag = CardColor.Blue });
            colorCombo.Items.Add(new ComboBoxItem { Content = GetColorLabelWithFallback("Orange", "clipboard.editDialog.colorOrange"), Tag = CardColor.Orange });

            colorCombo.SelectedIndex = (int)itemVM.CardColor;
            colorPanel.Children.Add(colorCombo);

            var hintText = new TextBlock
            {
                Text = L.Get("clipboard.editDialog.labelHint"),
                FontSize = 11,
                Opacity = 0.5,
                Margin = new Thickness(0, 4, 0, 0)
            };

            var contentPanel = new StackPanel { Spacing = 4 };
            contentPanel.Children.Add(labelBox);
            contentPanel.Children.Add(hintText);
            contentPanel.Children.Add(colorPanel);

            var dialog = new ContentDialog
            {
                Title = L.Get("clipboard.editDialog.title"),
                Content = contentPanel,
                PrimaryButtonText = L.Get("clipboard.editDialog.save"),
                CloseButtonText = L.Get("clipboard.editDialog.cancel"),
                DefaultButton = ContentDialogButton.Primary,
                XamlRoot = Content.XamlRoot
            };

            var result = await dialog.ShowAsync();

            if (result == ContentDialogResult.Primary && colorCombo.SelectedItem is ComboBoxItem selectedColor)
            {
                var label = string.IsNullOrWhiteSpace(labelBox.Text) ? null : labelBox.Text.Trim();
                var color = (CardColor)(selectedColor.Tag ?? CardColor.None);
                ViewModel.SaveItemLabelAndColor(itemVM, label, color);
            }
        }
        finally
        {
            _isDialogOpen = false;
        }
    }
}
