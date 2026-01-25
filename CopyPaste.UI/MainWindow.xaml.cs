using CopyPaste.Core;
using CopyPaste.UI.ViewModels;
using Microsoft.UI;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Input;
using System;
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

        ConfigureSidebarStyle();

        // Initial border fix
        PaintBorderGray(hWnd);

        this.Activated += MainWindow_Activated;
        this.Closed += (s, e) => { e.Handled = true; _appWindow.Hide(); };
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

    // DWM API Definition
#pragma warning disable IDE0060
    [System.Runtime.InteropServices.LibraryImport("dwmapi.dll")]
    public static partial int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref int attrValue, int attrSize);

    private const int _dWMWA_BORDER_COLOR = 34;

    private const int _colorDarkGray = 0x00353535;

    private static void PaintBorderGray(IntPtr hWnd)
    {
        int attributeValue = _colorDarkGray;
        _ = DwmSetWindowAttribute(hWnd, _dWMWA_BORDER_COLOR, ref attributeValue, sizeof(int));
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
            _appWindow.Hide();
        }
        else
        {
            IntPtr hWnd = WindowNative.GetWindowHandle(this);
            PaintBorderGray(hWnd);
        }
    }

    private void Card_PointerEntered(object sender, PointerRoutedEventArgs _)
    {
        if (sender is FrameworkElement root && root.FindName("ActionPanel") is UIElement panel)
        {
            panel.Opacity = 1;
        }
    }

    private void Card_PointerExited(object sender, PointerRoutedEventArgs _)
    {
        if (sender is FrameworkElement root && root.FindName("ActionPanel") is UIElement panel)
        {
            panel.Opacity = 0;
        }
    }
}
