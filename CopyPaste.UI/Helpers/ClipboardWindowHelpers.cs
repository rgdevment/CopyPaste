using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using System;
using System.Globalization;
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
}
