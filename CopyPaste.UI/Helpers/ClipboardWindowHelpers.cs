using CopyPaste.Core;
using CopyPaste.UI.Localization;
using CopyPaste.UI.Themes;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using System;
using System.Globalization;
using System.Threading.Tasks;
using Windows.UI;

namespace CopyPaste.UI.Helpers;

internal static class ClipboardWindowHelpers
{
    internal static ScrollViewer? FindScrollViewer(DependencyObject parent)
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

    internal static DependencyObject? FindDescendant(DependencyObject parent, string name)
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

    internal static T? FindChild<T>(DependencyObject parent, string name) where T : FrameworkElement
    {
        for (int i = 0; i < VisualTreeHelper.GetChildrenCount(parent); i++)
        {
            var child = VisualTreeHelper.GetChild(parent, i);
            if (child is T typedChild && typedChild.Name == name)
                return typedChild;
            var result = FindChild<T>(child, name);
            if (result != null)
                return result;
        }
        return null;
    }

    internal static Color ParseColor(string hex)
    {
        hex = hex.TrimStart('#');
        return Color.FromArgb(
            255,
            byte.Parse(hex.AsSpan(0, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture),
            byte.Parse(hex.AsSpan(2, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture),
            byte.Parse(hex.AsSpan(4, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture)
        );
    }

    internal static void LoadImageSource(Image image, string? imagePath, int thumbnailDecodeHeight)
    {
        if (string.IsNullOrEmpty(imagePath)) return;

        if (!imagePath.StartsWith("ms-appx://", StringComparison.OrdinalIgnoreCase) &&
            !System.IO.File.Exists(imagePath))
        {
            if (imagePath.Contains("_t.", StringComparison.Ordinal))
            {
                try
                {
                    image.Source = new BitmapImage
                    {
                        UriSource = new Uri("ms-appx:///Assets/thumb/image.png")
                    };
                }
                catch { /* Silently fail */ }
            }
            return;
        }

        if (image.Source is BitmapImage currentBitmap)
        {
            var currentPath = currentBitmap.UriSource?.LocalPath;
            if (currentPath != null && imagePath.EndsWith(System.IO.Path.GetFileName(currentPath), StringComparison.OrdinalIgnoreCase))
                return;
        }

        try
        {
            image.Source = new BitmapImage
            {
                UriSource = new Uri(imagePath),
                CreateOptions = BitmapCreateOptions.None,
                DecodePixelHeight = thumbnailDecodeHeight
            };
        }
        catch { /* Silently fail */ }
    }

    internal static void SetWindowIcon(AppWindow appWindow)
    {
        var iconPath = System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "CopyPasteLogoSimple.ico");
        if (System.IO.File.Exists(iconPath))
            appWindow.SetIcon(iconPath);
    }

    internal static async Task<(string? label, CardColor color)?> ShowEditDialogAsync(
        XamlRoot xamlRoot,
        ClipboardItemViewModel itemVM,
        Func<string, string, string>? colorLabelResolver = null)
    {
        colorLabelResolver ??= (_, key) => L.Get(key);

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
        colorCombo.Items.Add(new ComboBoxItem { Content = colorLabelResolver("Red", "clipboard.editDialog.colorRed"), Tag = CardColor.Red });
        colorCombo.Items.Add(new ComboBoxItem { Content = colorLabelResolver("Green", "clipboard.editDialog.colorGreen"), Tag = CardColor.Green });
        colorCombo.Items.Add(new ComboBoxItem { Content = colorLabelResolver("Purple", "clipboard.editDialog.colorPurple"), Tag = CardColor.Purple });
        colorCombo.Items.Add(new ComboBoxItem { Content = colorLabelResolver("Yellow", "clipboard.editDialog.colorYellow"), Tag = CardColor.Yellow });
        colorCombo.Items.Add(new ComboBoxItem { Content = colorLabelResolver("Blue", "clipboard.editDialog.colorBlue"), Tag = CardColor.Blue });
        colorCombo.Items.Add(new ComboBoxItem { Content = colorLabelResolver("Orange", "clipboard.editDialog.colorOrange"), Tag = CardColor.Orange });

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
            XamlRoot = xamlRoot
        };

        var result = await dialog.ShowAsync();
        if (result == ContentDialogResult.Primary && colorCombo.SelectedItem is ComboBoxItem selectedColor)
        {
            var label = string.IsNullOrWhiteSpace(labelBox.Text) ? null : labelBox.Text.Trim();
            var color = (CardColor)(selectedColor.Tag ?? CardColor.None);
            return (label, color);
        }
        return null;
    }
}
