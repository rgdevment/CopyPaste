using System;
using System.IO;
using System.Text.Json;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using CopyPaste.UI.Localization;
using Microsoft.UI.Xaml;

namespace CopyPaste.UI.ViewModels;

public partial class ClipboardItemViewModel : ObservableObject
{
    private readonly Action<ClipboardItemViewModel> _deleteAction;
    private readonly Action<ClipboardItemViewModel, bool> _pasteAction;
    private readonly Action<ClipboardItemViewModel> _pinAction;
    private readonly Action<ClipboardItemViewModel>? _editAction;
    private readonly string _pasteText;
    private readonly string _pastePlainText;
    private readonly string _deleteText;
    private readonly string _fileWarningText;
    private readonly string _pinText;
    private readonly string _unpinText;
    private readonly string _editText;
    private readonly string _headerTitle;
    private readonly bool _showPinIndicator;

    private bool _isPinned;
    private bool _isExpanded;

    public ClipboardItem Model { get; }

    public string Timestamp
    {
        get
        {
            var date = Model.CreatedAt.ToLocalTime();
            var now = DateTime.Now;

            if (date.Date == now.Date)
                return L.Get("clipboard.timestamps.today").Replace("{time}", date.ToString("HH:mm", System.Globalization.CultureInfo.InvariantCulture), StringComparison.Ordinal);

            if (date.Date == now.Date.AddDays(-1))
                return L.Get("clipboard.timestamps.yesterday").Replace("{time}", date.ToString("HH:mm", System.Globalization.CultureInfo.InvariantCulture), StringComparison.Ordinal);

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

    public bool IsExpanded
    {
        get => _isExpanded;
        set
        {
            if (SetProperty(ref _isExpanded, value))
            {
                OnPropertyChanged(nameof(ContentMaxLines));
                OnPropertyChanged(nameof(ContentLineHeight));
            }
        }
    }

    public int ContentMaxLines => _isExpanded
        ? ConfigLoader.Config.CardMaxLines
        : ConfigLoader.Config.CardMinLines;
    public double ContentLineHeight { get; } = 20.0;

    public ClipboardItemViewModel(
        ClipboardItem model,
        Action<ClipboardItemViewModel> deleteAction,
        Action<ClipboardItemViewModel, bool> pasteAction,
        Action<ClipboardItemViewModel> pinAction,
        Action<ClipboardItemViewModel>? editAction = null,
        bool showPinIndicator = false)
    {
        ArgumentNullException.ThrowIfNull(model);

        Model = model;
        _deleteAction = deleteAction;
        _pasteAction = pasteAction;
        _pinAction = pinAction;
        _editAction = editAction;
        _showPinIndicator = showPinIndicator;
        _pasteText = L.Get("clipboard.contextMenu.paste");
        _pastePlainText = L.Get("clipboard.contextMenu.pastePlain");
        _deleteText = L.Get("clipboard.contextMenu.delete");
        _fileWarningText = L.Get("clipboard.fileWarning");
        _pinText = L.Get("clipboard.contextMenu.pin");
        _unpinText = L.Get("clipboard.contextMenu.unpin");
        _editText = L.Get("clipboard.contextMenu.edit");
        _headerTitle = model.Type switch
        {
            ClipboardContentType.Text => L.Get("clipboard.itemTypes.text"),
            ClipboardContentType.Image => L.Get("clipboard.itemTypes.image"),
            ClipboardContentType.File => L.Get("clipboard.itemTypes.file"),
            ClipboardContentType.Folder => L.Get("clipboard.itemTypes.folder"),
            ClipboardContentType.Link => L.Get("clipboard.itemTypes.link"),
            ClipboardContentType.Audio => L.Get("clipboard.itemTypes.audio"),
            ClipboardContentType.Video => L.Get("clipboard.itemTypes.video"),
            _ => L.Get("clipboard.itemTypes.content")
        };

        _isPinned = model.IsPinned;
    }

    /// <summary>
    /// Updates the underlying model and notifies UI of property changes.
    /// Used when thumbnail/metadata becomes available without creating a new ViewModel.
    /// </summary>
    public void RefreshFromModel(ClipboardItem updatedModel)
    {
        ArgumentNullException.ThrowIfNull(updatedModel);

        // Update the model properties
        Model.Content = updatedModel.Content;
        Model.Metadata = updatedModel.Metadata;
        Model.ModifiedAt = updatedModel.ModifiedAt;

        // Clear cached paths to force re-evaluation
        _cachedThumbnailPath = null;
        _cachedImagePath = null;

        // Notify UI of ALL properties that might have changed
        OnPropertyChanged(nameof(Content));
        OnPropertyChanged(nameof(ThumbnailPath));
        OnPropertyChanged(nameof(ImagePath));
        OnPropertyChanged(nameof(HasValidImagePath));
        OnPropertyChanged(nameof(ImageVisibility));
        OnPropertyChanged(nameof(MediaThumbnailVisibility));
        OnPropertyChanged(nameof(MediaDuration));
        OnPropertyChanged(nameof(DurationVisibility));
        OnPropertyChanged(nameof(ImageDimensions));
        OnPropertyChanged(nameof(ImageDimensionsVisibility));
        OnPropertyChanged(nameof(FileSize));
        OnPropertyChanged(nameof(FileSizeVisibility));
        OnPropertyChanged(nameof(MediaInfoVisibility));
        OnPropertyChanged(nameof(IsTextVisible));
        OnPropertyChanged(nameof(IsFileAvailable));
        OnPropertyChanged(nameof(FileWarningVisibility));

        // Fire event for image reload (used by MainWindow)
        ImagePathChanged?.Invoke(this, EventArgs.Empty);
    }

    /// <summary>
    /// Call to refresh file availability status (e.g., when file might have been deleted).
    /// </summary>
    public void RefreshFileStatus()
    {
        OnPropertyChanged(nameof(IsFileAvailable));
        OnPropertyChanged(nameof(FileWarningVisibility));
    }

    /// <summary>
    /// Fired when ImagePath changes (thumbnail becomes available).
    /// </summary>
    public event EventHandler? ImagePathChanged;

    // Cached paths to avoid repeated JSON parsing - cleared on refresh
    private string? _cachedThumbnailPath;
    private string? _cachedImagePath;

    public string Content => Model.Content ?? string.Empty;

    public string ThumbnailPath
    {
        get
        {
            _cachedThumbnailPath ??= GetThumbnailPathOrPlaceholder();
            return _cachedThumbnailPath;
        }
    }

    public string ImagePath
    {
        get
        {
            _cachedImagePath ??= GetImagePathOrThumbnail();
            return _cachedImagePath;
        }
    }

    public bool HasValidImagePath => Model.Type == ClipboardContentType.Image && !string.IsNullOrEmpty(GetImagePathOrThumbnail());

    public Visibility ImageVisibility => HasValidImagePath ? Visibility.Visible : Visibility.Collapsed;

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

    public string HeaderTitle => _headerTitle;

    public string TypeIcon => Model.Type switch
    {
        ClipboardContentType.Text => "\uE8C4",
        ClipboardContentType.Image => "\uE91B",
        ClipboardContentType.Link => "\uE71B",
        ClipboardContentType.File => "\uE8B7",
        ClipboardContentType.Folder => "\uE8D5",
        ClipboardContentType.Audio => "\uE8D6",
        ClipboardContentType.Video => "\uE714",
        _ => "\uE7ba"
    };

    public string PinIconGlyph => _isPinned ? "\uE840" : "\uE718";
    public string PinMenuText => _isPinned ? _unpinText : _pinText;
    public string PasteText => _pasteText;
    public string PastePlainText => _pastePlainText;
    public string DeleteText => _deleteText;
    public string FileWarningText => _fileWarningText;

    public Visibility PinIndicatorVisibility => _isPinned && _showPinIndicator ? Visibility.Visible : Visibility.Collapsed;

    public string? Label => Model.Label;

    public bool HasLabel => !string.IsNullOrEmpty(Model.Label);

    public Visibility LabelVisibility => HasLabel ? Visibility.Visible : Visibility.Collapsed;

    public Visibility DefaultHeaderVisibility => HasLabel ? Visibility.Collapsed : Visibility.Visible;

    public CardColor CardColor => Model.CardColor;

    public bool HasCardColor => Model.CardColor != CardColor.None;

    public Visibility CardColorVisibility => HasCardColor ? Visibility.Visible : Visibility.Collapsed;

    public string CardBorderColor => Model.CardColor switch
    {
        CardColor.Red => "#E74C3C",
        CardColor.Green => "#2ECC71",
        CardColor.Purple => "#9B59B6",
        CardColor.Yellow => "#F1C40F",
        CardColor.Blue => "#3498DB",
        CardColor.Orange => "#E67E22",
        _ => "Transparent"
    };

    public string EditText => _editText;

    public bool CanEdit => _editAction != null;

    public bool IsFileType => Model.Type is ClipboardContentType.File or ClipboardContentType.Folder or ClipboardContentType.Audio or ClipboardContentType.Video;

    public bool IsFileAvailable => !IsFileType || CheckFirstFileExists();

    public Visibility FileWarningVisibility => IsFileType && !IsFileAvailable ? Visibility.Visible : Visibility.Collapsed;

    // App Source (bottom left)
    public string? AppSource => Model.AppSource;
    public Visibility AppSourceVisibility => !string.IsNullOrEmpty(AppSource) ? Visibility.Visible : Visibility.Collapsed;

    // Image dimensions from metadata
    public string? ImageDimensions => GetImageDimensions();
    public Visibility ImageDimensionsVisibility => ImageDimensions != null ? Visibility.Visible : Visibility.Collapsed;

    // File size from metadata
    public string? FileSize => GetFileSize();
    public Visibility FileSizeVisibility => FileSize != null ? Visibility.Visible : Visibility.Collapsed;

    // Media info visibility (duration + size for video/audio/image) - only when we have data
    public Visibility MediaInfoVisibility =>
        (Model.Type is ClipboardContentType.Video or ClipboardContentType.Audio or ClipboardContentType.Image)
        && (FileSize != null || ImageDimensions != null || MediaDuration != null)
            ? Visibility.Visible : Visibility.Collapsed;

    // Paste count display
    public string PasteCountDisplay => Model.PasteCount >= 1000 ? "×1K+" : $"×{Model.PasteCount}";
    public Visibility PasteCountVisibility => Model.PasteCount > 0 ? Visibility.Visible : Visibility.Collapsed;

    private string? GetFileSize()
    {
        if (string.IsNullOrEmpty(Model.Metadata)) return null;

        try
        {
            using var doc = JsonDocument.Parse(Model.Metadata);

            // Try "file_size" first (used for files), then "size" (used for images)
            if (doc.RootElement.TryGetProperty("file_size", out var fileSizeProp))
            {
                return FormatFileSize(fileSizeProp.GetInt64());
            }

            if (doc.RootElement.TryGetProperty("size", out var sizeProp))
            {
                return FormatFileSize(sizeProp.GetInt64());
            }
        }
        catch (JsonException) { }

        return null;
    }

    private static string FormatFileSize(long bytes)
    {
        string[] sizes = ["B", "KB", "MB", "GB"];
        int order = 0;
        double size = bytes;

        while (size >= 1024 && order < sizes.Length - 1)
        {
            order++;
            size /= 1024;
        }

        return $"{size:0.#} {sizes[order]}";
    }

    private string? GetImageDimensions()
    {
        if (Model.Type != ClipboardContentType.Image) return null;
        if (string.IsNullOrEmpty(Model.Metadata)) return null;

        try
        {
            using var doc = JsonDocument.Parse(Model.Metadata);
            if (doc.RootElement.TryGetProperty("width", out var widthProp) &&
                doc.RootElement.TryGetProperty("height", out var heightProp))
            {
                return $"{widthProp.GetInt64()}×{heightProp.GetInt64()}";
            }
        }
        catch (JsonException) { }

        return null;
    }

    private bool CheckFirstFileExists()
    {
        if (string.IsNullOrEmpty(Model.Content)) return false;

        var paths = Model.Content.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries);
        if (paths.Length == 0) return false;

        return File.Exists(paths[0]) || Directory.Exists(paths[0]);
    }

    private string GetImagePathOrThumbnail()
    {
        // First try the thumbnail path (always preferred for display)
        var thumbPath = GetThumbnailPath();
        if (!string.IsNullOrEmpty(thumbPath))
            return thumbPath;

        // Fallback to Content path if it exists
        if (!string.IsNullOrEmpty(Model.Content) && File.Exists(Model.Content))
            return Model.Content;

        // Generic placeholder for images when processing failed
        if (Model.Type == ClipboardContentType.Image)
            return "ms-appx:///Assets/thumb/image.png";

        return string.Empty;
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
        IsPinned = !IsPinned;
        _pinAction(this);
    }

    [RelayCommand]
    private void Edit() => _editAction?.Invoke(this);

    public void RefreshLabelAndColor()
    {
        OnPropertyChanged(nameof(Label));
        OnPropertyChanged(nameof(HasLabel));
        OnPropertyChanged(nameof(LabelVisibility));
        OnPropertyChanged(nameof(DefaultHeaderVisibility));
        OnPropertyChanged(nameof(CardColor));
        OnPropertyChanged(nameof(HasCardColor));
        OnPropertyChanged(nameof(CardColorVisibility));
        OnPropertyChanged(nameof(CardBorderColor));
    }

    public void RefreshPasteCount()
    {
        OnPropertyChanged(nameof(PasteCountDisplay));
        OnPropertyChanged(nameof(PasteCountVisibility));
    }

    public void ToggleExpanded() => IsExpanded = !IsExpanded;

    public void Collapse() => IsExpanded = false;
}
