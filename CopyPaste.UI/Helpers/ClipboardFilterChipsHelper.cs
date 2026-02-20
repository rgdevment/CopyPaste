using CopyPaste.Core;
using CopyPaste.UI.Themes;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using System.Collections.Generic;
using Windows.UI;

namespace CopyPaste.UI.Helpers;

internal static class ClipboardFilterChipsHelper
{
    internal static void SyncFilterChipsState(ClipboardThemeViewModelBase vm, FrameworkElement root)
    {
        SetChecked(root, "ColorCheckRed", vm.IsColorSelected(CardColor.Red));
        SetChecked(root, "ColorCheckGreen", vm.IsColorSelected(CardColor.Green));
        SetChecked(root, "ColorCheckPurple", vm.IsColorSelected(CardColor.Purple));
        SetChecked(root, "ColorCheckYellow", vm.IsColorSelected(CardColor.Yellow));
        SetChecked(root, "ColorCheckBlue", vm.IsColorSelected(CardColor.Blue));
        SetChecked(root, "ColorCheckOrange", vm.IsColorSelected(CardColor.Orange));

        SetChecked(root, "TypeCheckText", vm.IsTypeSelected(ClipboardContentType.Text));
        SetChecked(root, "TypeCheckImage", vm.IsTypeSelected(ClipboardContentType.Image));
        SetChecked(root, "TypeCheckFile", vm.IsTypeSelected(ClipboardContentType.File));
        SetChecked(root, "TypeCheckFolder", vm.IsTypeSelected(ClipboardContentType.Folder));
        SetChecked(root, "TypeCheckLink", vm.IsTypeSelected(ClipboardContentType.Link));
        SetChecked(root, "TypeCheckAudio", vm.IsTypeSelected(ClipboardContentType.Audio));
        SetChecked(root, "TypeCheckVideo", vm.IsTypeSelected(ClipboardContentType.Video));

        SetChecked(root, "FilterModeContent", vm.ActiveFilterMode == 0);
        SetChecked(root, "FilterModeCategory", vm.ActiveFilterMode == 1);
        SetChecked(root, "FilterModeType", vm.ActiveFilterMode == 2);

        UpdateSelectedColorsDisplay(vm, root);
        UpdateSelectedTypesDisplay(vm, root);
    }

    internal static void UpdateSelectedColorsDisplay(ClipboardThemeViewModelBase vm, FrameworkElement root)
    {
        if (root.FindName("SelectedColorsPanel") is not Panel panel) return;
        if (root.FindName("ColorPlaceholder") is not UIElement placeholder) return;

        while (panel.Children.Count > 1)
            panel.Children.RemoveAt(1);

        var selectedColors = new List<(CardColor color, string hex)>();
        if (vm.IsColorSelected(CardColor.Red)) selectedColors.Add((CardColor.Red, "#E74C3C"));
        if (vm.IsColorSelected(CardColor.Green)) selectedColors.Add((CardColor.Green, "#2ECC71"));
        if (vm.IsColorSelected(CardColor.Purple)) selectedColors.Add((CardColor.Purple, "#9B59B6"));
        if (vm.IsColorSelected(CardColor.Yellow)) selectedColors.Add((CardColor.Yellow, "#F1C40F"));
        if (vm.IsColorSelected(CardColor.Blue)) selectedColors.Add((CardColor.Blue, "#3498DB"));
        if (vm.IsColorSelected(CardColor.Orange)) selectedColors.Add((CardColor.Orange, "#E67E22"));

        if (selectedColors.Count == 0)
        {
            placeholder.Visibility = Visibility.Visible;
        }
        else
        {
            placeholder.Visibility = Visibility.Collapsed;
            foreach (var (_, hex) in selectedColors)
            {
                var chip = new Ellipse
                {
                    Width = 16,
                    Height = 16,
                    Fill = new SolidColorBrush(ClipboardWindowHelpers.ParseColor(hex)),
                    Stroke = new SolidColorBrush(Color.FromArgb(40, 0, 0, 0)),
                    StrokeThickness = 1,
                    Margin = new Thickness(0, 0, 2, 0)
                };
                panel.Children.Add(chip);
            }
        }
    }

    internal static void UpdateSelectedTypesDisplay(ClipboardThemeViewModelBase vm, FrameworkElement root)
    {
        if (root.FindName("SelectedTypesPanel") is not Panel panel) return;
        if (root.FindName("TypePlaceholder") is not UIElement placeholder) return;

        while (panel.Children.Count > 1)
            panel.Children.RemoveAt(1);

        var selectedTypes = new List<(ClipboardContentType type, string glyph)>();
        if (vm.IsTypeSelected(ClipboardContentType.Text)) selectedTypes.Add((ClipboardContentType.Text, "\uE8C1"));
        if (vm.IsTypeSelected(ClipboardContentType.Image)) selectedTypes.Add((ClipboardContentType.Image, "\uE91B"));
        if (vm.IsTypeSelected(ClipboardContentType.File)) selectedTypes.Add((ClipboardContentType.File, "\uE7C3"));
        if (vm.IsTypeSelected(ClipboardContentType.Folder)) selectedTypes.Add((ClipboardContentType.Folder, "\uE8B7"));
        if (vm.IsTypeSelected(ClipboardContentType.Link)) selectedTypes.Add((ClipboardContentType.Link, "\uE71B"));
        if (vm.IsTypeSelected(ClipboardContentType.Audio)) selectedTypes.Add((ClipboardContentType.Audio, "\uE8D6"));
        if (vm.IsTypeSelected(ClipboardContentType.Video)) selectedTypes.Add((ClipboardContentType.Video, "\uE714"));

        if (selectedTypes.Count == 0)
        {
            placeholder.Visibility = Visibility.Visible;
        }
        else
        {
            placeholder.Visibility = Visibility.Collapsed;
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
                    panel.Children.Add(moreText);
                    break;
                }
                var chipBorder = new Border
                {
                    Background = new SolidColorBrush(Color.FromArgb(30, 128, 128, 128)),
                    CornerRadius = new CornerRadius(4),
                    Padding = new Thickness(6, 3, 6, 3),
                    Child = new FontIcon { Glyph = glyph, FontSize = 12 }
                };
                panel.Children.Add(chipBorder);
                shown++;
            }
        }
    }

    private static void SetChecked(FrameworkElement root, string name, bool value)
    {
        if (root.FindName(name) is CheckBox cb) cb.IsChecked = value;
        else if (root.FindName(name) is RadioMenuFlyoutItem item) item.IsChecked = value;
    }
}
