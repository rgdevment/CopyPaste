using CopyPaste.UI.Localization;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace CopyPaste.UI.Themes;

/// <summary>
/// Builds and manages the settings UI for DefaultTheme.
/// Embedded in ConfigWindow via ITheme.CreateSettingsSection().
/// </summary>
internal sealed class DefaultThemeSettingsPanel
{
    // Appearance controls
    private NumberBox? _windowWidthBox;
    private NumberBox? _marginTopBox;
    private NumberBox? _marginBottomBox;
    private NumberBox? _cardMinLinesBox;
    private NumberBox? _cardMaxLinesBox;

    // Behavior controls
    private ToggleSwitch? _pinWindowSwitch;
    private ToggleSwitch? _resetScrollSwitch;
    private ToggleSwitch? _resetFilterModeSwitch;
    private ToggleSwitch? _resetContentFilterSwitch;
    private ToggleSwitch? _resetCategoryFilterSwitch;
    private ToggleSwitch? _resetTypeFilterSwitch;

    // Grids for opacity toggling
    private Grid? _resetContentFilterGrid;
    private Grid? _resetCategoryFilterGrid;
    private Grid? _resetTypeFilterGrid;

    /// <summary>
    /// Builds the complete settings UI and populates values from <paramref name="settings"/>.
    /// </summary>
    public UIElement Build(DefaultThemeSettings settings)
    {
        var panel = new StackPanel { Spacing = 24 };
        panel.Children.Add(BuildAppearanceSection(settings));
        panel.Children.Add(BuildBehaviorSection(settings));
        return panel;
    }

    /// <summary>
    /// Reads current control values into a new DefaultThemeSettings instance.
    /// </summary>
    public DefaultThemeSettings ToSettings() => new()
    {
        WindowWidth = (int)(_windowWidthBox?.Value ?? 400),
        WindowMarginTop = (int)(_marginTopBox?.Value ?? 8),
        WindowMarginBottom = (int)(_marginBottomBox?.Value ?? 16),
        CardMinLines = (int)(_cardMinLinesBox?.Value ?? 3),
        CardMaxLines = (int)(_cardMaxLinesBox?.Value ?? 9),
        PinWindow = _pinWindowSwitch?.IsOn ?? false,
        ResetScrollOnShow = _resetScrollSwitch?.IsOn ?? true,
        ResetFilterModeOnShow = _resetFilterModeSwitch?.IsOn ?? true,
        ResetContentFilterOnShow = _resetContentFilterSwitch?.IsOn ?? true,
        ResetCategoryFilterOnShow = _resetCategoryFilterSwitch?.IsOn ?? true,
        ResetTypeFilterOnShow = _resetTypeFilterSwitch?.IsOn ?? true,
    };

    /// <summary>
    /// Resets all controls to default values.
    /// </summary>
    public void Reset()
    {
        var d = new DefaultThemeSettings();
        LoadValues(d);
    }

    private void LoadValues(DefaultThemeSettings s)
    {
        if (_windowWidthBox != null) _windowWidthBox.Value = s.WindowWidth;
        if (_marginTopBox != null) _marginTopBox.Value = s.WindowMarginTop;
        if (_marginBottomBox != null) _marginBottomBox.Value = s.WindowMarginBottom;
        if (_cardMinLinesBox != null) _cardMinLinesBox.Value = s.CardMinLines;
        if (_cardMaxLinesBox != null) _cardMaxLinesBox.Value = s.CardMaxLines;
        if (_pinWindowSwitch != null) _pinWindowSwitch.IsOn = s.PinWindow;
        if (_resetScrollSwitch != null) _resetScrollSwitch.IsOn = s.ResetScrollOnShow;
        if (_resetFilterModeSwitch != null) _resetFilterModeSwitch.IsOn = s.ResetFilterModeOnShow;
        if (_resetContentFilterSwitch != null) _resetContentFilterSwitch.IsOn = s.ResetContentFilterOnShow;
        if (_resetCategoryFilterSwitch != null) _resetCategoryFilterSwitch.IsOn = s.ResetCategoryFilterOnShow;
        if (_resetTypeFilterSwitch != null) _resetTypeFilterSwitch.IsOn = s.ResetTypeFilterOnShow;
        UpdateResetFilterSwitchesEnabled();
    }

    // ═══════════════════════════════════════════════════════════════
    // APARIENCIA section
    // ═══════════════════════════════════════════════════════════════

    private StackPanel BuildAppearanceSection(DefaultThemeSettings s)
    {
        var section = new StackPanel();

        // Heading
        var headingPanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, Margin = new Thickness(0, 0, 0, 12) };
        headingPanel.Children.Add(new FontIcon { Glyph = "\uE771", FontSize = 16, Foreground = GetAccentBrush() });
        headingPanel.Children.Add(new TextBlock
        {
            Text = L.Get("config.appearance.heading"),
            FontSize = 12,
            FontWeight = Microsoft.UI.Text.FontWeights.SemiBold,
            Foreground = GetAccentBrush(),
            CharacterSpacing = 50,
        });
        section.Children.Add(headingPanel);

        // Card
        var card = new Border
        {
            Background = (Brush)Application.Current.Resources["CardBackgroundFillColorDefaultBrush"],
            CornerRadius = new CornerRadius(8),
            Padding = new Thickness(16),
        };
        var cardStack = new StackPanel { Spacing = 16 };

        _windowWidthBox = new NumberBox { Minimum = 400, Maximum = 600, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact, SmallChange = 10, Value = s.WindowWidth };
        cardStack.Children.Add(BuildNumberRow(L.Get("config.appearance.panelWidth"), L.Get("config.appearance.panelWidthDesc"), _windowWidthBox));

        cardStack.Children.Add(BuildDivider());

        _marginTopBox = new NumberBox { Minimum = 0, Maximum = 100, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact, SmallChange = 4, Value = s.WindowMarginTop };
        cardStack.Children.Add(BuildNumberRow(L.Get("config.appearance.marginTop"), L.Get("config.appearance.marginTopDesc"), _marginTopBox));

        cardStack.Children.Add(BuildDivider());

        _marginBottomBox = new NumberBox { Minimum = 0, Maximum = 100, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact, SmallChange = 4, Value = s.WindowMarginBottom };
        cardStack.Children.Add(BuildNumberRow(L.Get("config.appearance.marginBottom"), L.Get("config.appearance.marginBottomDesc"), _marginBottomBox));

        cardStack.Children.Add(BuildDivider());

        _cardMinLinesBox = new NumberBox { Minimum = 1, Maximum = 10, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact, SmallChange = 1, Value = s.CardMinLines };
        cardStack.Children.Add(BuildNumberRow(L.Get("config.appearance.cardMinLines"), L.Get("config.appearance.cardMinLinesDesc"), _cardMinLinesBox));

        cardStack.Children.Add(BuildDivider());

        _cardMaxLinesBox = new NumberBox { Minimum = 3, Maximum = 20, SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact, SmallChange = 1, Value = s.CardMaxLines };
        cardStack.Children.Add(BuildNumberRow(L.Get("config.appearance.cardMaxLines"), L.Get("config.appearance.cardMaxLinesDesc"), _cardMaxLinesBox));

        card.Child = cardStack;
        section.Children.Add(card);
        return section;
    }

    // ═══════════════════════════════════════════════════════════════
    // COMPORTAMIENTO section
    // ═══════════════════════════════════════════════════════════════

    private StackPanel BuildBehaviorSection(DefaultThemeSettings s)
    {
        var section = new StackPanel();

        // Heading
        var headingPanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, Margin = new Thickness(0, 0, 0, 12) };
        headingPanel.Children.Add(new FontIcon { Glyph = "\uE81C", FontSize = 16, Foreground = GetAccentBrush() });
        headingPanel.Children.Add(new TextBlock
        {
            Text = L.Get("config.behavior.heading"),
            FontSize = 12,
            FontWeight = Microsoft.UI.Text.FontWeights.SemiBold,
            Foreground = GetAccentBrush(),
            CharacterSpacing = 50,
        });
        section.Children.Add(headingPanel);

        // Card
        var card = new Border
        {
            Background = (Brush)Application.Current.Resources["CardBackgroundFillColorDefaultBrush"],
            CornerRadius = new CornerRadius(8),
            Padding = new Thickness(16),
        };
        var cardStack = new StackPanel { Spacing = 16 };

        _pinWindowSwitch = new ToggleSwitch
        {
            IsOn = s.PinWindow,
            OnContent = L.Get("config.behavior.yes"),
            OffContent = L.Get("config.behavior.no"),
        };
        cardStack.Children.Add(BuildToggleRow(
            L.Get("config.behavior.pinWindow"),
            L.Get("config.behavior.pinWindowDesc"),
            _pinWindowSwitch));

        cardStack.Children.Add(BuildDivider());

        _resetScrollSwitch = new ToggleSwitch
        {
            IsOn = s.ResetScrollOnShow,
            OnContent = L.Get("config.appearance.resetScrollOnShowYes"),
            OffContent = L.Get("config.appearance.resetScrollOnShowNo"),
        };
        cardStack.Children.Add(BuildToggleRow(
            L.Get("config.appearance.resetScrollOnShow"),
            L.Get("config.appearance.resetScrollOnShowDesc"),
            _resetScrollSwitch));

        cardStack.Children.Add(BuildDivider());

        _resetFilterModeSwitch = new ToggleSwitch
        {
            IsOn = s.ResetFilterModeOnShow,
            OnContent = L.Get("config.behavior.yes"),
            OffContent = L.Get("config.behavior.no"),
        };
        _resetFilterModeSwitch.Toggled += (_, _) => UpdateResetFilterSwitchesEnabled();
        cardStack.Children.Add(BuildToggleRow(
            L.Get("config.behavior.resetFilterMode"),
            L.Get("config.behavior.resetFilterModeDesc"),
            _resetFilterModeSwitch));

        cardStack.Children.Add(BuildDivider());

        _resetContentFilterSwitch = new ToggleSwitch
        {
            IsOn = s.ResetContentFilterOnShow,
            OnContent = L.Get("config.behavior.yes"),
            OffContent = L.Get("config.behavior.no"),
        };
        _resetContentFilterGrid = BuildToggleRow(
            L.Get("config.behavior.resetContentFilter"),
            L.Get("config.behavior.resetContentFilterDesc"),
            _resetContentFilterSwitch);
        cardStack.Children.Add(_resetContentFilterGrid);

        cardStack.Children.Add(BuildDivider());

        _resetCategoryFilterSwitch = new ToggleSwitch
        {
            IsOn = s.ResetCategoryFilterOnShow,
            OnContent = L.Get("config.behavior.yes"),
            OffContent = L.Get("config.behavior.no"),
        };
        _resetCategoryFilterGrid = BuildToggleRow(
            L.Get("config.behavior.resetCategoryFilter"),
            L.Get("config.behavior.resetCategoryFilterDesc"),
            _resetCategoryFilterSwitch);
        cardStack.Children.Add(_resetCategoryFilterGrid);

        cardStack.Children.Add(BuildDivider());

        _resetTypeFilterSwitch = new ToggleSwitch
        {
            IsOn = s.ResetTypeFilterOnShow,
            OnContent = L.Get("config.behavior.yes"),
            OffContent = L.Get("config.behavior.no"),
        };
        _resetTypeFilterGrid = BuildToggleRow(
            L.Get("config.behavior.resetTypeFilter"),
            L.Get("config.behavior.resetTypeFilterDesc"),
            _resetTypeFilterSwitch);
        cardStack.Children.Add(_resetTypeFilterGrid);

        card.Child = cardStack;
        section.Children.Add(card);

        UpdateResetFilterSwitchesEnabled();
        return section;
    }

    // ═══════════════════════════════════════════════════════════════
    // UI Helpers
    // ═══════════════════════════════════════════════════════════════

    private static Grid BuildNumberRow(string label, string description, NumberBox numberBox)
    {
        var grid = new Grid();
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(120) });

        var labelStack = new StackPanel();
        labelStack.Children.Add(new TextBlock { Text = label, FontSize = 14, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });
        labelStack.Children.Add(new TextBlock { Text = description, FontSize = 11, Opacity = 0.6, Margin = new Thickness(0, 2, 0, 0) });
        Grid.SetColumn(labelStack, 0);
        grid.Children.Add(labelStack);

        Grid.SetColumn(numberBox, 1);
        grid.Children.Add(numberBox);

        return grid;
    }

    private static Grid BuildToggleRow(string label, string description, ToggleSwitch toggle)
    {
        var grid = new Grid();
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });

        var labelStack = new StackPanel();
        labelStack.Children.Add(new TextBlock { Text = label, FontSize = 14, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });
        labelStack.Children.Add(new TextBlock { Text = description, FontSize = 11, Opacity = 0.6, Margin = new Thickness(0, 2, 0, 0) });
        Grid.SetColumn(labelStack, 0);
        grid.Children.Add(labelStack);

        Grid.SetColumn(toggle, 1);
        grid.Children.Add(toggle);

        return grid;
    }

    private static Border BuildDivider() => new()
    {
        Height = 1,
        Background = (Brush)Application.Current.Resources["DividerStrokeColorDefaultBrush"],
        Opacity = 0.3,
    };

    private static Brush GetAccentBrush() =>
        (Brush)Application.Current.Resources["AccentTextFillColorPrimaryBrush"];

    private void UpdateResetFilterSwitchesEnabled()
    {
        if (_resetFilterModeSwitch == null) return;

        var enabled = !_resetFilterModeSwitch.IsOn;
        if (_resetContentFilterSwitch != null) _resetContentFilterSwitch.IsEnabled = enabled;
        if (_resetCategoryFilterSwitch != null) _resetCategoryFilterSwitch.IsEnabled = enabled;
        if (_resetTypeFilterSwitch != null) _resetTypeFilterSwitch.IsEnabled = enabled;
        if (_resetContentFilterGrid != null) _resetContentFilterGrid.Opacity = enabled ? 1.0 : 0.5;
        if (_resetCategoryFilterGrid != null) _resetCategoryFilterGrid.Opacity = enabled ? 1.0 : 0.5;
        if (_resetTypeFilterGrid != null) _resetTypeFilterGrid.Opacity = enabled ? 1.0 : 0.5;
    }
}
