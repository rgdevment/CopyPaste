using CopyPaste.Core;
using CopyPaste.UI.ViewModels;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using System;
using System.Runtime.Versioning;
using WinRT.Interop;

namespace CopyPaste.UI;

public sealed partial class MainWindow : Window
{
    public MainViewModel ViewModel { get; }
    private readonly AppWindow _appWindow;

    public MainWindow(ClipboardService service)
    {
        ViewModel = new MainViewModel(service);
        ViewModel.Initialize(this);
        this.InitializeComponent();

        IntPtr hWnd = WindowNative.GetWindowHandle(this);
        WindowId wndId = Win32Interop.GetWindowIdFromWindow(hWnd);
        _appWindow = AppWindow.GetFromWindowId(wndId);

        SetWindowIcon();
        ConfigureSidebarStyle();

        RemoveWindowBorder(hWnd);

        this.Activated += MainWindow_Activated;
        this.Closed += (s, e) => { e.Handled = true; _appWindow.Hide(); };

        ClipboardListView.Loaded += ClipboardListView_Loaded;
    }

    private void ClipboardListView_Loaded(object sender, RoutedEventArgs e)
    {
        var scrollViewer = FindScrollViewer(ClipboardListView);
        if (scrollViewer != null)
        {
            scrollViewer.ViewChanged += ScrollViewer_ViewChanged;
        }
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

    private void ScrollViewer_ViewChanged(object? sender, ScrollViewerViewChangedEventArgs e)
    {
        if (sender is not ScrollViewer scrollViewer) return;

        var verticalOffset = scrollViewer.VerticalOffset;
        var maxVerticalOffset = scrollViewer.ScrollableHeight;

        if (maxVerticalOffset > 0 && verticalOffset >= maxVerticalOffset - 100)
        {
            ViewModel.LoadMoreItems();
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
        {
            _appWindow.SetIcon(iconPath);
        }
    }

    // Win32 API Definitions
    [System.Runtime.InteropServices.LibraryImport("user32.dll")]
    private static partial nint GetWindowLongPtrW(IntPtr hWnd, int nIndex);

    [System.Runtime.InteropServices.LibraryImport("user32.dll")]
    private static partial nint SetWindowLongPtrW(IntPtr hWnd, int nIndex, nint dwNewLong);

    [System.Runtime.InteropServices.LibraryImport("user32.dll")]
    [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.Bool)]
    private static partial bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);

    [System.Runtime.InteropServices.LibraryImport("dwmapi.dll")]
    private static partial int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref uint attrValue, int attrSize);

    // Window style constants
    private const int _gWL_STYLE = -16;
    private const int _gWL_EXSTYLE = -20;

    private const nint _wS_BORDER = 0x00800000;
    private const nint _wS_DLGFRAME = 0x00400000;
    private const nint _wS_THICKFRAME = 0x00040000;

    private const nint _wS_EX_WINDOWEDGE = 0x00000100;
    private const nint _wS_EX_CLIENTEDGE = 0x00000200;
    private const nint _wS_EX_STATICEDGE = 0x00020000;

    // SetWindowPos flags
    private const uint _sWP_FRAMECHANGED = 0x0020;
    private const uint _sWP_NOMOVE = 0x0002;
    private const uint _sWP_NOSIZE = 0x0001;
    private const uint _sWP_NOZORDER = 0x0004;
    private const uint _sWP_NOACTIVATE = 0x0010;

    private const int _dWMWA_WINDOW_CORNER_PREFERENCE = 33;

    private static void RemoveWindowBorder(IntPtr hWnd)
    {
        nint style = GetWindowLongPtrW(hWnd, _gWL_STYLE);
        style &= ~_wS_BORDER;
        style &= ~_wS_DLGFRAME;
        style &= ~_wS_THICKFRAME;
        SetWindowLongPtrW(hWnd, _gWL_STYLE, style);

        nint exStyle = GetWindowLongPtrW(hWnd, _gWL_EXSTYLE);
        exStyle &= ~_wS_EX_WINDOWEDGE;
        exStyle &= ~_wS_EX_CLIENTEDGE;
        exStyle &= ~_wS_EX_STATICEDGE;
        SetWindowLongPtrW(hWnd, _gWL_EXSTYLE, exStyle);

        SetWindowPos(hWnd, IntPtr.Zero, 0, 0, 0, 0,
            _sWP_FRAMECHANGED | _sWP_NOMOVE | _sWP_NOSIZE | _sWP_NOZORDER | _sWP_NOACTIVATE);

        uint cornerPreference = 2;
        _ = DwmSetWindowAttribute(hWnd, _dWMWA_WINDOW_CORNER_PREFERENCE, ref cornerPreference, sizeof(uint));
    }

    private void MoveToRightEdge()
    {
        var displayArea = DisplayArea.GetFromWindowId(_appWindow.Id, DisplayAreaFallback.Primary);
        var workArea = displayArea.WorkArea;
        int width = 400;
        int height = workArea.Height - 16;
        int x = (workArea.X + workArea.Width) - width;
        int y = workArea.Y + 8;
        _appWindow.MoveAndResize(new Windows.Graphics.RectInt32(x, y, width, height));
    }

    private void MainWindow_Activated(object sender, WindowActivatedEventArgs args)
    {
        if (args.WindowActivationState == WindowActivationState.Deactivated)
        {
            ViewModel.OnWindowDeactivated();
            _appWindow.Hide();
        }
        else
        {
            IntPtr hWnd = WindowNative.GetWindowHandle(this);
            RemoveWindowBorder(hWnd);
        }
    }

    [SupportedOSPlatform("windows10.0.17763.0")]
    private void Card_PointerEntered(object sender, PointerRoutedEventArgs _)
    {
        if (sender is FrameworkElement root && root.FindName("ActionPanel") is UIElement panel)
        {
            panel.Opacity = 1;
        }
    }

    [SupportedOSPlatform("windows10.0.17763.0")]
    private void Card_PointerExited(object sender, PointerRoutedEventArgs _)
    {
        if (sender is FrameworkElement root && root.FindName("ActionPanel") is UIElement panel)
        {
            panel.Opacity = 0;
        }
    }

    private void ClipboardImage_Loaded(object sender, RoutedEventArgs e)
    {
        if (sender is not Image image) return;

        var imagePath = image.Tag as string;
        if (string.IsNullOrEmpty(imagePath)) return;

        try
        {
            image.Source = new Microsoft.UI.Xaml.Media.Imaging.BitmapImage(new Uri(imagePath));
        }
        catch (UriFormatException ex)
        {
            System.Diagnostics.Debug.WriteLine($"Formato de URI inválido: {ex.Message}");
            throw;
        }
        catch (System.IO.IOException ex)
        {
            System.Diagnostics.Debug.WriteLine($"Error de E/S al cargar la imagen: {ex.Message}");
            throw;
        }
        catch (UnauthorizedAccessException ex)
        {
            System.Diagnostics.Debug.WriteLine($"Acceso no autorizado al cargar la imagen: {ex.Message}");
            throw;
        }
    }
}

