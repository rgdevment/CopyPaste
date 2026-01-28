using CopyPaste.Core;
using CopyPaste.UI.Helpers;
using CopyPaste.UI.Localization;
using CopyPaste.UI.ViewModels;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using System;
using System.Linq;
using Windows.UI;
using WinRT.Interop;

namespace CopyPaste.UI;

internal sealed partial class MainWindow : Window
{
    public MainViewModel ViewModel { get; }
    private readonly AppWindow _appWindow;
    private readonly IntPtr _hWnd;
    private readonly MyMConfig _config = ConfigLoader.Config; // Cache config once at startup
    private const int _hotkeyId = 1;

    public MainWindow(ClipboardService service)
    {
        ViewModel = new MainViewModel(service);
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
        RegisterGlobalHotkey();
        HotkeyHelper.RegisterMessageHandler(_hWnd, OnHotkeyPressed);
    }

    private void ApplyLocalizedStrings()
    {
        TrayMenuExit.Text = L.Get("tray.exit");
        ToolTipService.SetToolTip(RecentTab, L.Get("ui.section.recent"));
        ToolTipService.SetToolTip(PinnedTab, L.Get("ui.section.pinned"));
        ToolTipService.SetToolTip(SettingsButton, L.Get("ui.sidebar.settings"));
        ToolTipService.SetToolTip(ReportButton, L.Get("ui.sidebar.report"));
        SearchBox.PlaceholderText = L.Get("ui.search.placeholder");
        EmptyStateText.Text = L.Get("ui.emptyState");
        SectionTitle.Text = L.Get("ui.section.recent");
    }

    private void RegisterEventHandlers()
    {
        Activated += MainWindow_Activated;
        Closed += MainWindow_Closed;
        _appWindow.Changed += AppWindow_Changed;
        ClipboardListView.Loaded += ClipboardListView_Loaded;
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
        // Verificar que los elementos XAML estén inicializados
        if (SectionTitle == null || SectionIcon == null)
            return;

        var orangeColor = Color.FromArgb(255, 210, 120, 50);
        var orangeBrush = new SolidColorBrush(orangeColor);
        var blueColor = Color.FromArgb(255, 91, 155, 213);
        var blueBrush = new SolidColorBrush(blueColor);
        var inactiveColor = Color.FromArgb(255, 120, 120, 120);
        var inactiveBrush = new SolidColorBrush(inactiveColor);

        if (tabName == nameof(PinnedTab))
        {
            SectionTitle.Text = L.Get("ui.section.pinned");
            SectionIcon.Glyph = "\uE718";
            SectionTitle.Foreground = orangeBrush;
            SectionIcon.Foreground = orangeBrush;

            // Actualizar iconos del sidebar
            if (PinnedTabIcon != null) PinnedTabIcon.Foreground = orangeBrush;
            if (RecentTabIcon != null) RecentTabIcon.Foreground = inactiveBrush;
        }
        else
        {
            SectionTitle.Text = L.Get("ui.section.recent");
            SectionIcon.Glyph = "\uE823";
            SectionTitle.Foreground = blueBrush;
            SectionIcon.Foreground = blueBrush;

            // Actualizar iconos del sidebar
            if (RecentTabIcon != null) RecentTabIcon.Foreground = blueBrush;
            if (PinnedTabIcon != null) PinnedTabIcon.Foreground = inactiveBrush;
        }
    }

    private void MainWindow_Closed(object? _, WindowEventArgs args)
    {
        Win32WindowHelper.UnregisterHotKey(_hWnd, _hotkeyId);
        HotkeyHelper.UnregisterMessageHandler(_hWnd);

        // Cleanup ViewModel event subscriptions
        ViewModel.Cleanup();

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

    private void OnHotkeyPressed()
    {
        if (_appWindow.IsVisible)
        {
            _appWindow.Hide();
        }
        else
        {
            // FocusHelper.CapturePreviousWindow() is called in HotkeyHelper
            // BEFORE this method, ensuring we capture the correct window
            ViewModel.ShowWindow();
        }
    }

    private void RegisterGlobalHotkey()
    {
        uint modifiers = 0;

        if (_config.UseCtrlKey) modifiers |= Win32WindowHelper.MOD_CONTROL;
        if (_config.UseWinKey) modifiers |= Win32WindowHelper.MOD_WIN;
        if (_config.UseAltKey) modifiers |= Win32WindowHelper.MOD_ALT;
        if (_config.UseShiftKey) modifiers |= Win32WindowHelper.MOD_SHIFT;

        bool registered = Win32WindowHelper.RegisterHotKey(_hWnd, _hotkeyId, modifiers, _config.VirtualKey);

        // Fallback: if Win key fails, try with Ctrl instead
        if (!registered && _config.UseWinKey)
        {
            modifiers &= ~Win32WindowHelper.MOD_WIN;
            modifiers |= Win32WindowHelper.MOD_CONTROL;
            Win32WindowHelper.RegisterHotKey(_hWnd, _hotkeyId, modifiers, _config.VirtualKey);
        }
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

    private void MainWindow_Activated(object? _, WindowActivatedEventArgs args)
    {
        if (args.WindowActivationState == WindowActivationState.Deactivated)
        {
            ViewModel.OnWindowDeactivated();
            _appWindow.Hide();
        }
        else
        {
            Win32WindowHelper.RemoveWindowBorder(_hWnd);
            // Refresh file availability status for visible items
            RefreshFileAvailability();
        }
    }

    private void RefreshFileAvailability()
    {
        // Refresh file status for visible file-type items when window is activated
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
        int width = _config.WindowWidth;
        int height = workArea.Height - _config.WindowMarginBottom;
        int x = workArea.X + workArea.Width - width;
        int y = workArea.Y + _config.WindowMarginTop;
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
        container.PointerEntered += Container_PointerEntered;
        container.PointerExited += Container_PointerExited;
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

    private void Container_DoubleTapped(object sender, Microsoft.UI.Xaml.Input.DoubleTappedRoutedEventArgs e)
    {
        if (sender is ListViewItem item && item.Content is ViewModels.ClipboardItemViewModel vm)
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

    private static void LoadImageSource(Image image, string? imagePath)
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
                DecodePixelHeight = ConfigLoader.Config.ThumbnailUIDecodeHeight // Static access needed here
            };
        }
        catch { /* Silently fail */ }
    }

    private void SearchBox_TextChanged(object? sender, TextChangedEventArgs _)
    {
        if (SearchBox is TextBox textBox)
            ViewModel.SearchQuery = textBox.Text ?? string.Empty;
    }

    private void OpenSettings_Click(object sender, RoutedEventArgs e)
    {
        var configWindow = new ConfigWindow();
        configWindow.Activate();
    }
}
