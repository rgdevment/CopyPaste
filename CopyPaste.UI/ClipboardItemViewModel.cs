using System;
using System.IO;
using System.Text.Json;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using Microsoft.UI.Xaml;

namespace CopyPaste.UI.ViewModels;

public partial class ClipboardItemViewModel : ObservableObject
{
    private readonly Action<ClipboardItemViewModel> _deleteAction;
    private readonly Action<ClipboardItemViewModel, bool> _pasteAction;
    private readonly Action<ClipboardItemViewModel> _pinAction;

    private bool _isPinned;

    public ClipboardItem Model { get; }

    public string Timestamp
    {
        get
        {
            var date = Model.CreatedAt.ToLocalTime();
            var now = DateTime.Now;

            if (date.Date == now.Date)
                return $"Hoy {date:HH:mm}";

            if (date.Date == now.Date.AddDays(-1))
                return $"Ayer {date:HH:mm}";

            return date.ToString("dd MMM HH:mm", System.Globalization.CultureInfo.InvariantCulture);
        }
    }

    public bool IsPinned
    {
        get => _isPinned;
        set
        {
            if (SetProperty(ref _isPinned, value))
            {
                OnPropertyChanged(nameof(PinIconGlyph));
                OnPropertyChanged(nameof(PinMenuText));
                OnPropertyChanged(nameof(PinIndicatorVisibility));

                Model.IsPinned = _isPinned;
            }
        }
    }

    public ClipboardItemViewModel(
        ClipboardItem model,
        Action<ClipboardItemViewModel> deleteAction,
        Action<ClipboardItemViewModel, bool> pasteAction,
        Action<ClipboardItemViewModel> pinAction)
    {
        ArgumentNullException.ThrowIfNull(model);

        Model = model;
        _deleteAction = deleteAction;
        _pasteAction = pasteAction;
        _pinAction = pinAction;

        _isPinned = model.IsPinned;
    }

    public string Content => Model.Content ?? string.Empty;

    public string ThumbnailPath => GetThumbnailPathOrPlaceholder();

    public Visibility ImageVisibility =>
        Model.Type == ClipboardContentType.Image && !string.IsNullOrEmpty(Model.Content)
            ? Visibility.Visible : Visibility.Collapsed;

    public Visibility MediaThumbnailVisibility =>
        Model.Type is ClipboardContentType.Video or ClipboardContentType.Audio
            ? Visibility.Visible : Visibility.Collapsed;

    public Visibility IsTextVisible =>
        Model.Type is not ClipboardContentType.Image
        && Model.Type is not ClipboardContentType.Video
        && Model.Type is not ClipboardContentType.Audio
            ? Visibility.Visible : Visibility.Collapsed;

    public string? MediaDuration => GetMediaDuration();

    public Visibility DurationVisibility => MediaDuration != null ? Visibility.Visible : Visibility.Collapsed;

    public string HeaderTitle => Model.Type switch
    {
        ClipboardContentType.Text => "TEXT",
        ClipboardContentType.Image => "IMAGE",
        ClipboardContentType.File => "FILE",
        ClipboardContentType.Link => "LINK",
        ClipboardContentType.Audio => "AUDIO",
        ClipboardContentType.Video => "VIDEO",
        _ => "CONTENT"
    };

    public string TypeIcon => Model.Type switch
    {
        ClipboardContentType.Text => "\uE8C4",
        ClipboardContentType.Image => "\uE91B",
        ClipboardContentType.Link => "\uE71B",
        ClipboardContentType.File => "\uE8B7",
        ClipboardContentType.Audio => "\uE8D6",
        ClipboardContentType.Video => "\uE714",
        _ => "\uE7ba"
    };

    public string PinIconGlyph => _isPinned ? "\uE840" : "\uE718";
    public string PinMenuText => _isPinned ? "Desanclar" : "Anclar";
    public Visibility PinIndicatorVisibility => _isPinned ? Visibility.Visible : Visibility.Collapsed;

    public bool IsFileType => Model.Type is ClipboardContentType.File or ClipboardContentType.Audio or ClipboardContentType.Video;

    public bool IsFileAvailable => !IsFileType || CheckFirstFileExists();

    public Visibility FileWarningVisibility => IsFileType && !IsFileAvailable ? Visibility.Visible : Visibility.Collapsed;

    private bool CheckFirstFileExists()
    {
        if (string.IsNullOrEmpty(Model.Content)) return false;

        var paths = Model.Content.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries);
        if (paths.Length == 0) return false;

        return File.Exists(paths[0]);
    }

    private string GetThumbnailPathOrPlaceholder()
    {
        var thumbPath = GetThumbnailPath();
        if (!string.IsNullOrEmpty(thumbPath))
            return thumbPath;

        // Return static placeholder from Assets based on type
        return Model.Type switch
        {
            ClipboardContentType.Video => "ms-appx:///Assets/thumb/video.png",
            ClipboardContentType.Audio => "ms-appx:///Assets/thumb/audio.png",
            _ => "ms-appx:///Assets/thumb/video.png"
        };
    }

    private string? GetThumbnailPath()
    {
        if (string.IsNullOrEmpty(Model.Metadata)) return null;

        try
        {
            using var doc = JsonDocument.Parse(Model.Metadata);
            if (doc.RootElement.TryGetProperty("thumb_path", out var pathProp))
            {
                var path = pathProp.GetString();
                if (!string.IsNullOrEmpty(path) && File.Exists(path))
                    return path;
            }
        }
        catch (JsonException) { }

        return null;
    }

    private string? GetMediaDuration()
    {
        if (string.IsNullOrEmpty(Model.Metadata)) return null;

        try
        {
            using var doc = JsonDocument.Parse(Model.Metadata);
            if (doc.RootElement.TryGetProperty("duration", out var durationProp))
            {
                var seconds = durationProp.GetInt64();
                var ts = TimeSpan.FromSeconds(seconds);
                return ts.TotalHours >= 1
                    ? ts.ToString(@"h\:mm\:ss", System.Globalization.CultureInfo.InvariantCulture)
                    : ts.ToString(@"m\:ss", System.Globalization.CultureInfo.InvariantCulture);
            }
        }
        catch (JsonException) { }

        return null;
    }


    [RelayCommand]
    private void Delete() => _deleteAction(this);

    [RelayCommand]
    private void Paste() => _pasteAction(this, false);

    [RelayCommand]
    private void PastePlain() => _pasteAction(this, true);

    [RelayCommand]
    private void TogglePin()
    {
        // This triggers the 'set' method in the IsPinned property defined above
        IsPinned = !IsPinned;

        // Execute the callback to save in DB
        _pinAction(this);
    }
}
