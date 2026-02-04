using CopyPaste.UI.Localization;
using Microsoft.UI.Xaml;

namespace CopyPaste.UI;

internal sealed partial class HelpWindow : Window
{
    public HelpWindow()
    {
        InitializeComponent();
        SetWindowProperties();
        ApplyLocalizedStrings();
    }

    private void SetWindowProperties()
    {
        Title = L.Get("help.window.title");

        // Center window on screen
        var hWnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
        var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hWnd);
        var appWindow = Microsoft.UI.Windowing.AppWindow.GetFromWindowId(windowId);

        if (appWindow != null)
        {
            // Set window size
            appWindow.Resize(new Windows.Graphics.SizeInt32(700, 600));

            // Center on screen
            var displayArea = Microsoft.UI.Windowing.DisplayArea.GetFromWindowId(windowId, Microsoft.UI.Windowing.DisplayAreaFallback.Primary);
            if (displayArea != null)
            {
                var workArea = displayArea.WorkArea;
                var x = (workArea.Width - 700) / 2 + workArea.X;
                var y = (workArea.Height - 600) / 2 + workArea.Y;
                appWindow.Move(new Windows.Graphics.PointInt32(x, y));
            }
        }
    }

    private void ApplyLocalizedStrings()
    {
        // Header
        HelpTitle.Text = L.Get("help.title");
        HelpSubtitle.Text = L.Get("help.subtitle");

        // Sections
        SectionGeneral.Text = L.Get("help.section.general");
        SectionSearch.Text = L.Get("help.section.search");
        SectionFilters.Text = L.Get("help.section.filters");
        SectionNavigation.Text = L.Get("help.section.navigation");
        SectionActions.Text = L.Get("help.section.actions");

        // General shortcuts
        ShortcutOpenClose.Text = L.Get("help.shortcuts.openClose.key");
        DescOpenClose.Text = L.Get("help.shortcuts.openClose.desc");
        ShortcutEscape.Text = L.Get("help.shortcuts.escape.key");
        DescEscape.Text = L.Get("help.shortcuts.escape.desc");
        ShortcutTab1.Text = L.Get("help.shortcuts.tab1.key");
        DescTab1.Text = L.Get("help.shortcuts.tab1.desc");
        ShortcutTab2.Text = L.Get("help.shortcuts.tab2.key");
        DescTab2.Text = L.Get("help.shortcuts.tab2.desc");

        // Search shortcuts
        ShortcutDown.Text = L.Get("help.shortcuts.down.key");
        DescDown.Text = L.Get("help.shortcuts.down.desc");
        ShortcutShiftTab.Text = L.Get("help.shortcuts.shiftTab.key");
        DescShiftTab.Text = L.Get("help.shortcuts.shiftTab.desc");

        // Filter shortcuts
        ShortcutFilterContent.Text = L.Get("help.shortcuts.filterContent.key");
        DescFilterContent.Text = L.Get("help.shortcuts.filterContent.desc");
        ShortcutFilterCategory.Text = L.Get("help.shortcuts.filterCategory.key");
        DescFilterCategory.Text = L.Get("help.shortcuts.filterCategory.desc");
        ShortcutFilterType.Text = L.Get("help.shortcuts.filterType.key");
        DescFilterType.Text = L.Get("help.shortcuts.filterType.desc");

        // Navigation shortcuts
        ShortcutArrows.Text = L.Get("help.shortcuts.arrows.key");
        DescArrows.Text = L.Get("help.shortcuts.arrows.desc");
        ShortcutRight.Text = L.Get("help.shortcuts.right.key");
        DescRight.Text = L.Get("help.shortcuts.right.desc");

        // Action shortcuts
        ShortcutEnter.Text = L.Get("help.shortcuts.enter.key");
        DescEnter.Text = L.Get("help.shortcuts.enter.desc");
        ShortcutDelete.Text = L.Get("help.shortcuts.delete.key");
        DescDelete.Text = L.Get("help.shortcuts.delete.desc");
        ShortcutPin.Text = L.Get("help.shortcuts.pin.key");
        DescPin.Text = L.Get("help.shortcuts.pin.desc");
        ShortcutEdit.Text = L.Get("help.shortcuts.edit.key");
        DescEdit.Text = L.Get("help.shortcuts.edit.desc");

        // Footer
        CloseButton.Content = L.Get("help.closeButton");
    }

    private void CloseButton_Click(object sender, RoutedEventArgs e) => Close();
}
