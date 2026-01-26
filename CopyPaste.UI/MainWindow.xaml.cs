using CopyPaste.Core;
using CopyPaste.UI.Helpers;
using CopyPaste.UI.ViewModels;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using System;
using System.Diagnostics;
using System.Runtime.Versioning;
using WinRT.Interop;

namespace CopyPaste.UI;

public sealed partial class MainWindow : Window
{
    public MainViewModel ViewModel { get; }
    private readonly AppWindow _appWindow;
    private readonly IntPtr _hWnd;
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
    }

    private void InitializeWindow()
    {
        SetWindowIcon();
        ConfigureSidebarStyle();
        Win32WindowHelper.RemoveWindowBorder(_hWnd);
        RegisterGlobalHotkey();
        HotkeyHelper.RegisterMessageHandler(_hWnd, OnHotkeyPressed);
    }

    private void RegisterEventHandlers()
    {
        Activated += MainWindow_Activated;
        Closed += MainWindow_Closed;
        _appWindow.Changed += AppWindow_Changed;
        ClipboardListView.Loaded += ClipboardListView_Loaded;
    }

    private void MainWindow_Closed(object? _, WindowEventArgs args)
    {
        Win32WindowHelper.UnregisterHotKey(_hWnd, _hotkeyId);
        HotkeyHelper.UnregisterMessageHandler(_hWnd);

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
            _appWindow.Hide();
        else
            ViewModel.ShowWindow();
    }

    private void RegisterGlobalHotkey()
    {
        uint modifiers = UIHotkey.UseWinKey ? Win32WindowHelper.MOD_WIN : Win32WindowHelper.MOD_CONTROL;

        if (UIHotkey.UseAltKey)
            modifiers |= Win32WindowHelper.MOD_ALT;

        bool registered = Win32WindowHelper.RegisterHotKey(_hWnd, _hotkeyId, modifiers, UIHotkey.VirtualKey);

        if (!registered && UIHotkey.UseWinKey)
        {
            modifiers = Win32WindowHelper.MOD_CONTROL;
            if (UIHotkey.UseAltKey)
                modifiers |= Win32WindowHelper.MOD_ALT;
            Win32WindowHelper.RegisterHotKey(_hWnd, _hotkeyId, modifiers, UIHotkey.VirtualKey);
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

        if (sv.ScrollableHeight > 0 && sv.VerticalOffset >= sv.ScrollableHeight - UIConfig.ScrollLoadThreshold)
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
        int width = UIConfig.WindowWidth;
        int height = workArea.Height - UIConfig.WindowMarginBottom;
        int x = workArea.X + workArea.Width - width;
        int y = workArea.Y + UIConfig.WindowMarginTop;
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
        if (sender is FrameworkElement fe &&
            VisualTreeHelper.GetChild(fe, 0) is FrameworkElement root &&
            root.FindName("ActionPanel") is UIElement panel)
            panel.Opacity = 1;
    }

    private static void Container_PointerExited(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (sender is FrameworkElement fe &&
            VisualTreeHelper.GetChild(fe, 0) is FrameworkElement root &&
            root.FindName("ActionPanel") is UIElement panel)
            panel.Opacity = 0;
    }

    private void ClipboardListView_ContainerContentChanging(ListViewBase _, ContainerContentChangingEventArgs args)
    {
        if (args.InRecycleQueue) return;

        var container = args.ItemContainer;
        container.PointerEntered -= Container_PointerEntered;
        container.PointerExited -= Container_PointerExited;
        container.PointerEntered += Container_PointerEntered;
        container.PointerExited += Container_PointerExited;

        args.RegisterUpdateCallback(LoadClipboardImage);
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
            if (imagePath.Contains("_t.png", StringComparison.Ordinal))
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

        if (image.Source is Microsoft.UI.Xaml.Media.Imaging.BitmapImage currentBitmap &&
            currentBitmap.UriSource?.LocalPath == new Uri(imagePath).LocalPath)
            return;

        try
        {
            image.Source = new Microsoft.UI.Xaml.Media.Imaging.BitmapImage
            {
                UriSource = new Uri(imagePath),
                CreateOptions = Microsoft.UI.Xaml.Media.Imaging.BitmapCreateOptions.None,
                DecodePixelHeight = ThumbnailConfig.UIDecodeHeight
            };
        }
        catch { /* Silently fail */ }
    }

    private void SearchBox_TextChanged(object? sender, TextChangedEventArgs _)
    {
        Debug.WriteLine("Search box text changed. Sender: {0}", sender);
        if (SearchBox is TextBox textBox)
            ViewModel.SearchQuery = textBox.Text ?? string.Empty;
    }
}
