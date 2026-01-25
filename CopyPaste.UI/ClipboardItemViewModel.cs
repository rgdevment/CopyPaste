using System;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using Microsoft.UI.Xaml;

namespace CopyPaste.UI.ViewModels;

public partial class ClipboardItemViewModel : ObservableObject
{
    // --- Private Fields (Internal State) ---
    private readonly Action<ClipboardItemViewModel> _deleteAction;
    private readonly Action<ClipboardItemViewModel, bool> _pasteAction;
    private readonly Action<ClipboardItemViewModel> _pinAction;

    // Internal backing field for the UI state
    private bool _isPinned;

    // --- Public Properties ---
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

    // Explicit Property Implementation
    // This connects the private field (_isPinned) to the public UI (IsPinned)
    public bool IsPinned
    {
        get => _isPinned;
        set
        {
            // SetProperty checks if value changed and notifies UI
            if (SetProperty(ref _isPinned, value))
            {
                // When 'IsPinned' changes, these visual properties must update too
                OnPropertyChanged(nameof(PinIconGlyph));
                OnPropertyChanged(nameof(PinMenuText));
                OnPropertyChanged(nameof(PinIndicatorVisibility));

                // Update the underlying data model
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

        // INIT STATE:
        // We take the value from the Database Model (model.IsPinned)
        // and push it into our UI variable (_isPinned) to start correctly.
        _isPinned = model.IsPinned;
    }

    public string Content => Model.Content;

    // --- Visual Helpers (Read-only) ---

    public Visibility IsImageVisible => Model.Type == ClipboardContentType.Image ? Visibility.Visible : Visibility.Collapsed;
    public Visibility IsTextVisible => Model.Type != ClipboardContentType.Image ? Visibility.Visible : Visibility.Collapsed;

    public string HeaderTitle => Model.Type switch
    {
        ClipboardContentType.Text => "TEXT",
        ClipboardContentType.Image => "IMAGE",
        ClipboardContentType.Html => "TEXT",
        ClipboardContentType.File => "FILE",
        _ => "CONTENT"
    };

    public string TypeIcon => Model.Type switch
    {
        ClipboardContentType.Text => "\uE8C4", // Document
        ClipboardContentType.Image => "\uE91B", // Photo
        ClipboardContentType.Html => "\uE774",  // Globe
        ClipboardContentType.File => "\uE8B7",  // File
        _ => "\uE7ba" // Clipboard
    };

    // --- Dynamic UI Properties ---
    // These depend on 'IsPinned'. When IsPinned changes, these update.

    public string PinIconGlyph => _isPinned ? "\uE840" : "\uE718"; // Filled vs Outline
    public string PinMenuText => _isPinned ? "Desanclar" : "Anclar";
    public Visibility PinIndicatorVisibility => _isPinned ? Visibility.Visible : Visibility.Collapsed;

    // --- Commands ---

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
