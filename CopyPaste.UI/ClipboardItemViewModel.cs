using System;
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

    public string Content => Model.Content;

    public Visibility IsImageVisible => Model.Type == ClipboardContentType.Image ? Visibility.Visible : Visibility.Collapsed;
    public Visibility IsTextVisible => Model.Type != ClipboardContentType.Image ? Visibility.Visible : Visibility.Collapsed;

    public string HeaderTitle => Model.Type switch
    {
        ClipboardContentType.Text => "TEXT",
        ClipboardContentType.RichText => "RICH TEXT",
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
        ClipboardContentType.RichText => "\uE8C4",
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
