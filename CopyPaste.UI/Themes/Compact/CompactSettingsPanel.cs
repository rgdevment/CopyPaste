using CopyPaste.UI.Localization;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace CopyPaste.UI.Themes;

internal sealed class CompactSettingsPanel
{
    private NumberBox? _popupWidthBox;
    private NumberBox? _popupHeightBox;
    private NumberBox? _cardMinLinesBox;
    private NumberBox? _cardMaxLinesBox;
    private ToggleSwitch? _pinWindowSwitch;
    private ToggleSwitch? _scrollToTopOnPasteSwitch;
    private ToggleSwitch? _hideOnDeactivateSwitch;
    private ToggleSwitch? _resetScrollSwitch;
    private ToggleSwitch? _resetSearchSwitch;

    public UIElement Build(CompactSettings settings)
    {
        var panel = new StackPanel { Spacing = 24 };
        panel.Children.Add(BuildAppearanceSection(settings));
        panel.Children.Add(BuildBehaviorSection(settings));
        return panel;
    }

    public CompactSettings ToSettings() => new()
    {
        PopupWidth = (int)(_popupWidthBox?.Value ?? 368),
        PopupHeight = (int)(_popupHeightBox?.Value ?? 480),
        CardMinLines = (int)(_cardMinLinesBox?.Value ?? 2),
        CardMaxLines = (int)(_cardMaxLinesBox?.Value ?? 5),
        PinWindow = _pinWindowSwitch?.IsOn ?? false,
        ScrollToTopOnPaste = _scrollToTopOnPasteSwitch?.IsOn ?? true,
        HideOnDeactivate = _hideOnDeactivateSwitch?.IsOn ?? true,
        ResetScrollOnShow = _resetScrollSwitch?.IsOn ?? true,
        ResetSearchOnShow = _resetSearchSwitch?.IsOn ?? true,
    };

    public void Reset()
    {
        var d = new CompactSettings();
        LoadValues(d);
    }

    private void LoadValues(CompactSettings s)
    {
        if (_popupWidthBox != null) _popupWidthBox.Value = s.PopupWidth;
        if (_popupHeightBox != null) _popupHeightBox.Value = s.PopupHeight;
        if (_cardMinLinesBox != null) _cardMinLinesBox.Value = s.CardMinLines;
        if (_cardMaxLinesBox != null) _cardMaxLinesBox.Value = s.CardMaxLines;
        if (_pinWindowSwitch != null) _pinWindowSwitch.IsOn = s.PinWindow;
        if (_scrollToTopOnPasteSwitch != null) _scrollToTopOnPasteSwitch.IsOn = s.ScrollToTopOnPaste;
        if (_hideOnDeactivateSwitch != null) _hideOnDeactivateSwitch.IsOn = s.HideOnDeactivate;
        if (_resetScrollSwitch != null) _resetScrollSwitch.IsOn = s.ResetScrollOnShow;
        if (_resetSearchSwitch != null) _resetSearchSwitch.IsOn = s.ResetSearchOnShow;
    }

    private StackPanel BuildAppearanceSection(CompactSettings s)
    {
        var section = new StackPanel();

        var headingPanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, Margin = new Thickness(0, 0, 0, 12) };
        headingPanel.Children.Add(new FontIcon { Glyph = "\uE771", FontSize = 16, Foreground = GetAccentBrush() });
        headingPanel.Children.Add(new TextBlock
        {
            Text = L.Get("config.appearance.heading"),
            FontSize = 12,
            FontWeight = Microsoft.UI.Text.FontWeights.SemiBold,
            Opacity = 0.8,
            VerticalAlignment = VerticalAlignment.Center
        });
        section.Children.Add(headingPanel);

        var grid = new Grid { Margin = new Thickness(0, 0, 0, 0) };
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        grid.RowDefinitions.Add(new RowDefinition());
        grid.RowDefinitions.Add(new RowDefinition());
        grid.RowDefinitions.Add(new RowDefinition());
        grid.RowDefinitions.Add(new RowDefinition());

        _popupWidthBox = CreateNumberBox(s.PopupWidth, 250, 600, 10);
        AddSettingRow(grid, 0, L.Get("config.compact.popupWidth", "Popup width (px)"), _popupWidthBox);

        _popupHeightBox = CreateNumberBox(s.PopupHeight, 300, 800, 10);
        AddSettingRow(grid, 1, L.Get("config.compact.popupHeight", "Popup height (px)"), _popupHeightBox);

        _cardMinLinesBox = CreateNumberBox(s.CardMinLines, 1, 10, 1);
        AddSettingRow(grid, 2, L.Get("config.compact.cardMinLines", "Card min lines"), _cardMinLinesBox);

        _cardMaxLinesBox = CreateNumberBox(s.CardMaxLines, 1, 20, 1);
        AddSettingRow(grid, 3, L.Get("config.compact.cardMaxLines", "Card max lines"), _cardMaxLinesBox);

        section.Children.Add(grid);
        return section;
    }

    private StackPanel BuildBehaviorSection(CompactSettings s)
    {
        var section = new StackPanel();

        var headingPanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, Margin = new Thickness(0, 0, 0, 12) };
        headingPanel.Children.Add(new FontIcon { Glyph = "\uE713", FontSize = 16, Foreground = GetAccentBrush() });
        headingPanel.Children.Add(new TextBlock
        {
            Text = L.Get("config.behavior.heading"),
            FontSize = 12,
            FontWeight = Microsoft.UI.Text.FontWeights.SemiBold,
            Opacity = 0.8,
            VerticalAlignment = VerticalAlignment.Center
        });
        section.Children.Add(headingPanel);

        var grid = new Grid { Margin = new Thickness(0, 0, 0, 0) };
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        grid.RowDefinitions.Add(new RowDefinition());
        grid.RowDefinitions.Add(new RowDefinition());
        grid.RowDefinitions.Add(new RowDefinition());
        grid.RowDefinitions.Add(new RowDefinition());
        grid.RowDefinitions.Add(new RowDefinition());

        _pinWindowSwitch = new ToggleSwitch { IsOn = s.PinWindow, Margin = new Thickness(0, 4, 0, 4) };
        _pinWindowSwitch.Toggled += (_, _) => UpdatePinWindowDependentSwitches();
        AddSettingRow(grid, 0, L.Get("config.behavior.pinWindow", "Keep window visible"), _pinWindowSwitch);

        _scrollToTopOnPasteSwitch = new ToggleSwitch { IsOn = s.ScrollToTopOnPaste, IsEnabled = s.PinWindow, Margin = new Thickness(0, 4, 0, 4) };
        AddSettingRow(grid, 1, L.Get("config.behavior.scrollToTopOnPaste", "Scroll to top after paste"), _scrollToTopOnPasteSwitch);

        _hideOnDeactivateSwitch = new ToggleSwitch { IsOn = s.HideOnDeactivate, Margin = new Thickness(0, 4, 0, 4) };
        AddSettingRow(grid, 2, L.Get("config.compact.hideOnDeactivate", "Hide on deactivate"), _hideOnDeactivateSwitch);

        _resetScrollSwitch = new ToggleSwitch { IsOn = s.ResetScrollOnShow, Margin = new Thickness(0, 4, 0, 4) };
        AddSettingRow(grid, 3, L.Get("config.compact.resetScroll", "Reset scroll on show"), _resetScrollSwitch);

        _resetSearchSwitch = new ToggleSwitch { IsOn = s.ResetSearchOnShow, Margin = new Thickness(0, 4, 0, 4) };
        AddSettingRow(grid, 4, L.Get("config.compact.resetSearch", "Reset search on show"), _resetSearchSwitch);

        section.Children.Add(grid);
        return section;
    }

    private static void AddSettingRow(Grid grid, int row, string label, FrameworkElement control)
    {
        var text = new TextBlock
        {
            Text = label,
            FontSize = 12,
            VerticalAlignment = VerticalAlignment.Center,
            Opacity = 0.9,
            Margin = new Thickness(0, 6, 12, 6)
        };
        Grid.SetRow(text, row);
        Grid.SetColumn(text, 0);
        grid.Children.Add(text);

        Grid.SetRow(control, row);
        Grid.SetColumn(control, 1);
        grid.Children.Add(control);
    }

    private static NumberBox CreateNumberBox(int value, int min, int max, int step) => new()
    {
        Value = value,
        Minimum = min,
        Maximum = max,
        SmallChange = step,
        SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact,
        Width = 130,
        Margin = new Thickness(0, 4, 0, 4)
    };

    private static SolidColorBrush GetAccentBrush() =>
        new(Windows.UI.Color.FromArgb(255, 0, 120, 215));

    private void UpdatePinWindowDependentSwitches()
    {
        if (_scrollToTopOnPasteSwitch != null)
            _scrollToTopOnPasteSwitch.IsEnabled = _pinWindowSwitch?.IsOn ?? false;
    }
}
