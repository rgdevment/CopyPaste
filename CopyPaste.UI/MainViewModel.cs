using System;
using System.Collections.ObjectModel;
using System.Globalization;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using WinRT.Interop;

namespace CopyPaste.UI.ViewModels;

public partial class MainViewModel : ObservableObject
{
    private readonly ClipboardService _service;
    private Window? _window;
    private DispatcherQueue? _dispatcherQueue;
    private bool _isLoading;
    private bool _hasMoreItems = true;

    public ObservableCollection<ClipboardItemViewModel> Items { get; } = [];
    public ObservableCollection<ClipboardItemViewModel> FilteredItems { get; } = [];

    [ObservableProperty]
    public partial bool IsLoadingMore { get; set; }

    [ObservableProperty]
    public partial string SearchQuery { get; set; } = string.Empty;

    [ObservableProperty]
    public partial bool HasSearchQuery { get; set; } = false;

    partial void OnSearchQueryChanged(string value)
    {
        HasSearchQuery = !string.IsNullOrWhiteSpace(value);
        ApplyFilter();
    }

    [System.Runtime.InteropServices.LibraryImport("user32.dll")]
    [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.Bool)]
    private static partial bool SetForegroundWindow(IntPtr hWnd);

    public MainViewModel(ClipboardService service)
    {
        _service = service;
        _service.OnItemAdded += OnItemAdded;
        _service.OnThumbnailReady += OnThumbnailReady;
        LoadHistory();
    }

    public void Initialize(Window window)
    {
        ArgumentNullException.ThrowIfNull(window);
        _window = window;
        _dispatcherQueue = window.DispatcherQueue;
    }

    private void LoadHistory()
    {
        Items.Clear();
        _hasMoreItems = true;
        foreach (var item in _service.GetHistory(UIConfig.PageSize))
        {
            Items.Add(new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));
        }
        ApplyFilter();
    }

    public void LoadMoreItems()
    {
        if (_isLoading || !_hasMoreItems) return;

        _isLoading = true;
        IsLoadingMore = true;

        _dispatcherQueue?.TryEnqueue(() =>
        {
            var newItems = _service.GetHistory(UIConfig.PageSize, Items.Count).ToList();

            if (newItems.Count == 0)
            {
                _hasMoreItems = false;
            }
            else
            {
                foreach (var item in newItems)
                {
                    Items.Add(new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));
                }
            }

            _isLoading = false;
            IsLoadingMore = false;
            ApplyFilter();
        });
    }

    private void OnItemAdded(ClipboardItem item) =>
        _dispatcherQueue?.TryEnqueue(() =>
        {
            Items.Insert(0, new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));
            ApplyFilter();
        });

    private void OnThumbnailReady(ClipboardItem item) =>
        _dispatcherQueue?.TryEnqueue(() =>
        {
            var existingVm = Items.FirstOrDefault(vm => vm.Model.Id == item.Id);
            if (existingVm != null)
            {
                var index = Items.IndexOf(existingVm);
                Items[index] = new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem);
                ApplyFilter();
            }
        });

    private void OnDeleteItem(ClipboardItemViewModel itemVM)
    {
        // Delete from database and clean up app-generated files
        _service.RemoveItem(itemVM.Model.Id);
        // Remove from UI
        Items.Remove(itemVM);
        FilteredItems.Remove(itemVM);
    }
    private void OnPasteItem(ClipboardItemViewModel itemVM, bool plain) { }

    private void OnPinItem(ClipboardItemViewModel itemVM) =>
        _service.UpdatePin(itemVM.Model.Id, itemVM.IsPinned);

    public void OnWindowDeactivated()
    {
        if (Items.Count <= UIConfig.MaxItemsBeforeCleanup) return;

        while (Items.Count > UIConfig.PageSize)
        {
            Items.RemoveAt(Items.Count - 1);
        }

        _hasMoreItems = true;
        GC.Collect(2, GCCollectionMode.Optimized, false);
    }

    private void ApplyFilter()
    {
        FilteredItems.Clear();

        if (string.IsNullOrWhiteSpace(SearchQuery))
        {
            foreach (var item in Items)
            {
                FilteredItems.Add(item);
            }
            System.Diagnostics.Debug.WriteLine($"[Search] Query empty - showing all {FilteredItems.Count} items");
            return;
        }

        var filtered = Items.Where(MatchesSearchQuery).ToList();
        
        System.Diagnostics.Debug.WriteLine($"[Search] Query: '{SearchQuery}' - Found {filtered.Count} matches out of {Items.Count} items");

        foreach (var item in filtered)
        {
            FilteredItems.Add(item);
        }
    }

    private static string RemoveAccents(string text)
    {
        if (string.IsNullOrEmpty(text))
            return text;

        var normalizedString = text.Normalize(NormalizationForm.FormD);
        var stringBuilder = new StringBuilder();

        foreach (var c in normalizedString)
        {
            var unicodeCategory = CharUnicodeInfo.GetUnicodeCategory(c);
            if (unicodeCategory != UnicodeCategory.NonSpacingMark)
            {
                stringBuilder.Append(c);
            }
        }

        return stringBuilder.ToString().Normalize(NormalizationForm.FormC);
    }

    private bool MatchesSearchQuery(ClipboardItemViewModel item)
    {
        var query = RemoveAccents(SearchQuery.Trim()).ToUpperInvariant();

        if (string.IsNullOrWhiteSpace(query))
            return true;

        var content = RemoveAccents(item.Model.Content ?? string.Empty).ToUpperInvariant();
        var appSource = RemoveAccents(item.AppSource ?? string.Empty).ToUpperInvariant();

        if (query.Contains("IMAGE", StringComparison.Ordinal) || query.Contains("IMAGEN", StringComparison.Ordinal))
            return item.Model.Type == ClipboardContentType.Image;

        if (query.Contains("VIDEO", StringComparison.Ordinal))
            return item.Model.Type == ClipboardContentType.Video;

        if (query.Contains("AUDIO", StringComparison.Ordinal) || query.Contains("MUSICA", StringComparison.Ordinal) || query.Contains("CANCION", StringComparison.Ordinal))
            return item.Model.Type == ClipboardContentType.Audio;

        if (query.Contains("FILE", StringComparison.Ordinal) || query.Contains("ARCHIVO", StringComparison.Ordinal))
            return item.Model.Type == ClipboardContentType.File;

        if (query.Contains("FOLDER", StringComparison.Ordinal) || query.Contains("CARPETA", StringComparison.Ordinal) || query.Contains("DIRECTORIO", StringComparison.Ordinal))
            return item.Model.Type == ClipboardContentType.Folder;

        if (query.Contains("LINK", StringComparison.Ordinal) || query.Contains("ENLACE", StringComparison.Ordinal) || query.Contains("URL", StringComparison.Ordinal))
            return item.Model.Type == ClipboardContentType.Link;

        if (query.Contains("TEXT", StringComparison.Ordinal) || query.Contains("TEXTO", StringComparison.Ordinal))
            return item.Model.Type == ClipboardContentType.Text;

        var fileName = ExtractFileName(content);

        if ((query.Length > 1 && query.StartsWith('*')) || (query.Length > 0 && query.StartsWith('.')))
        {
            var extension = query.TrimStart('*', '.').ToUpperInvariant();
            return content.Contains($".{extension}", StringComparison.Ordinal) || 
                   fileName.EndsWith($".{extension}", StringComparison.Ordinal) ||
                   appSource.Contains($".{extension}", StringComparison.Ordinal);
        }

        if (query.Length > 2 && query.StartsWith('*') && query.EndsWith('*'))
        {
            var searchTerm = query.AsSpan(1, query.Length - 2).ToString();
            return content.Contains(searchTerm, StringComparison.Ordinal) ||
                   appSource.Contains(searchTerm, StringComparison.Ordinal) ||
                   fileName.Contains(searchTerm, StringComparison.Ordinal);
        }

        if (query.Length > 1 && query.EndsWith('*'))
        {
            var searchTerm = query.AsSpan(0, query.Length - 1).ToString();
            return ContentContainsWord(content, searchTerm, isPrefix: true) ||
                   appSource.StartsWith(searchTerm, StringComparison.Ordinal) || 
                   fileName.StartsWith(searchTerm, StringComparison.Ordinal);
        }

        if (query.Length > 1 && query.StartsWith('*'))
        {
            var searchTerm = query.AsSpan(1).ToString();
            return ContentContainsWord(content, searchTerm, isSuffix: true) ||
                   appSource.EndsWith(searchTerm, StringComparison.Ordinal) || 
                   fileName.EndsWith(searchTerm, StringComparison.Ordinal);
        }

        return ContentContainsWord(content, query) || 
               appSource.Contains(query, StringComparison.Ordinal) || 
               fileName.Contains(query, StringComparison.Ordinal);
    }

    private static readonly char[] _fileNameSeparators = ['\\', '/', '\n', '\r'];

    private static string ExtractFileName(string path)
    {
        if (string.IsNullOrEmpty(path))
            return string.Empty;

        var fileName = path.Split(_fileNameSeparators, StringSplitOptions.RemoveEmptyEntries).LastOrDefault() ?? string.Empty;
        return fileName.ToUpperInvariant();
    }

    private static bool ContentContainsWord(string content, string searchTerm, bool isPrefix = false, bool isSuffix = false)
    {
        if (string.IsNullOrEmpty(content) || string.IsNullOrEmpty(searchTerm))
            return false;

        var separators = new[] { ' ', '\n', '\r', '\t', '\\', '/', '.', ',', ';', ':', '|', '-', '_' };

        if (isPrefix)
        {
            var words = content.Split(separators, StringSplitOptions.RemoveEmptyEntries);
            return words.Any(w => w.StartsWith(searchTerm, StringComparison.Ordinal));
        }

        if (isSuffix)
        {
            var words = content.Split(separators, StringSplitOptions.RemoveEmptyEntries);
            return words.Any(w => w.EndsWith(searchTerm, StringComparison.Ordinal));
        }

        var tokens = content.Split(separators, StringSplitOptions.RemoveEmptyEntries);
        if (tokens.Any(w => w.Equals(searchTerm, StringComparison.Ordinal)))
            return true;

        return content.Contains(searchTerm, StringComparison.Ordinal);
    }

    [RelayCommand]
    private void ClearSearch() => SearchQuery = string.Empty;

    [RelayCommand]
    private void ClearAll()
    {
        for (int i = Items.Count - 1; i >= 0; i--)
        {
            if (!Items[i].IsPinned)
                Items.RemoveAt(i);
        }
        ApplyFilter();
    }

    [RelayCommand]
    private static async Task OpenRepo()
    {
        var uri = new Uri("https://github.com/rgdevment/CopyPaste/issues");
        await Windows.System.Launcher.LaunchUriAsync(uri);
    }

    [RelayCommand]
    public void ShowWindow()
    {
        _window?.Activate();
        if (_window != null)
        {
            var hWnd = WindowNative.GetWindowHandle(_window);
            SetForegroundWindow(hWnd);
        }
    }

    [RelayCommand]
    public static void Exit()
    {
        if (Application.Current is App app)
        {
            app.BeginExit();
        }
        else
        {
            Application.Current.Exit();
        }
    }
}
