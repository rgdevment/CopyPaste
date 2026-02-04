using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using CopyPaste.Core;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;

namespace CopyPaste.UI.Helpers;

/// <summary>
/// Helper class for Windows clipboard operations.
/// Handles setting text, files, images, and other content types to the system clipboard.
/// </summary>
[System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types")]
internal static class ClipboardHelper
{
    /// <summary>
    /// Sets the clipboard content based on the item type.
    /// </summary>
    /// <param name="item">The clipboard item to paste.</param>
    /// <param name="plainText">If true, paste as plain text (removes formatting).</param>
    /// <returns>True if the operation was successful, false otherwise.</returns>
    public static bool SetClipboardContent(ClipboardItem item, bool plainText = false)
    {
        ArgumentNullException.ThrowIfNull(item);

        try
        {
            return item.Type switch
            {
                ClipboardContentType.Text => SetText(item.Content, item.Metadata, plainText),
                ClipboardContentType.Link => SetText(item.Content, metadata: null, plainText: true),
                ClipboardContentType.Image => SetImage(item),
                ClipboardContentType.File => SetFiles(item.Content),
                ClipboardContentType.Folder => SetFiles(item.Content),
                ClipboardContentType.Audio => SetFiles(item.Content),
                ClipboardContentType.Video => SetFiles(item.Content),
                _ => SetText(item.Content, metadata: null, plainText: true)
            };
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to set clipboard content");
            return false;
        }
    }

    /// <summary>
    /// Sets text content to the clipboard.
    /// </summary>
    private static bool SetText(string content, string? metadata, bool plainText)
    {
        if (string.IsNullOrEmpty(content))
            return false;

        try
        {
            var dataPackage = new DataPackage();
            dataPackage.SetText(content);

            // If not plain text and we have RTF metadata, include it
            if (!plainText && !string.IsNullOrEmpty(metadata))
            {
                try
                {
                    using var doc = JsonDocument.Parse(metadata);
                    if (doc.RootElement.TryGetProperty("rtf", out var rtfProp))
                    {
                        var rtfBase64 = rtfProp.GetString();
                        if (!string.IsNullOrEmpty(rtfBase64))
                        {
                            var rtfBytes = Convert.FromBase64String(rtfBase64);
                            var rtfText = System.Text.Encoding.UTF8.GetString(rtfBytes);
                            dataPackage.SetRtf(rtfText);
                        }
                    }
                }
                catch (JsonException)
                {
                    // Metadata parsing failed, proceed with plain text only
                }
            }

            Clipboard.SetContent(dataPackage);
            Clipboard.Flush();
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to set text to clipboard");
            return false;
        }
    }

    /// <summary>
    /// Sets image content to the clipboard.
    /// </summary>
    private static bool SetImage(ClipboardItem item)
    {
        // For images, Content contains the path to the saved image file
        if (string.IsNullOrEmpty(item.Content) || !File.Exists(item.Content))
        {
            AppLogger.Warn($"Image file not found: {item.Content}");
            return false;
        }

        try
        {
            var dataPackage = new DataPackage();

            // Set as file reference
            var storageFile = StorageFile.GetFileFromPathAsync(item.Content).AsTask().Result;

            // Use explicit List<IStorageItem> for WinRT interop compatibility
            var storageItems = new List<IStorageItem> { storageFile };
            dataPackage.SetStorageItems(storageItems);

            // Also set as bitmap for applications that prefer bitmap format
            var streamRef = Windows.Storage.Streams.RandomAccessStreamReference.CreateFromFile(storageFile);
            dataPackage.SetBitmap(streamRef);

            Clipboard.SetContent(dataPackage);
            Clipboard.Flush();
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to set image to clipboard");
            return false;
        }
    }

    /// <summary>
    /// Sets file/folder paths to the clipboard.
    /// </summary>
    private static bool SetFiles(string content)
    {
        if (string.IsNullOrEmpty(content))
            return false;

        try
        {
            var paths = content.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries);
            var validPaths = new List<IStorageItem>();

            foreach (var path in paths)
            {
                var trimmedPath = path.Trim();
                if (string.IsNullOrEmpty(trimmedPath))
                    continue;

                try
                {
                    if (File.Exists(trimmedPath))
                    {
                        var file = StorageFile.GetFileFromPathAsync(trimmedPath).AsTask().Result;
                        validPaths.Add(file);
                    }
                    else if (Directory.Exists(trimmedPath))
                    {
                        var folder = StorageFolder.GetFolderFromPathAsync(trimmedPath).AsTask().Result;
                        validPaths.Add(folder);
                    }
                    else
                    {
                        AppLogger.Warn($"Path not found: {trimmedPath}");
                    }
                }
                catch (Exception ex)
                {
                    AppLogger.Exception(ex, $"Failed to access path '{trimmedPath}'");
                }
            }

            if (validPaths.Count == 0)
            {
                AppLogger.Warn("No valid files or folders found to copy");
                return false;
            }

            var dataPackage = new DataPackage
            {
                RequestedOperation = DataPackageOperation.Copy
            };
            dataPackage.SetStorageItems(validPaths, readOnly: false);

            Clipboard.SetContent(dataPackage);
            Clipboard.Flush();
            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to set files to clipboard");
            return false;
        }
    }
}
