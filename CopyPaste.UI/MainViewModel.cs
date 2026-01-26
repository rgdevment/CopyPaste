using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using CopyPaste.UI.Helpers;
using Microsoft.UI;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
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

    [ObservableProperty]
    public partial bool IsLoadingMore { get; set; }

    [ObservableProperty]
    public partial string SearchQuery { get; set; } = string.Empty;

    [ObservableProperty]
    public partial bool HasSearchQuery { get; set; } = false;

    [ObservableProperty]
    public partial int SelectedTabIndex { get; set; }

    private bool? CurrentPinnedFilter => SelectedTabIndex switch
    {
        0 => false,  // Recientes: IsPinned = false
        1 => true,   // Anclados: IsPinned = true
        _ => null
    };

    partial void OnSelectedTabIndexChanged(int value) => ReloadItems();

    partial void OnSearchQueryChanged(string value)
    {
        HasSearchQuery = !string.IsNullOrWhiteSpace(value);
        ReloadItems();
    }

    [System.Runtime.InteropServices.LibraryImport("user32.dll")]
    [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.Bool)]
    private static partial bool SetForegroundWindow(IntPtr hWnd);

    public MainViewModel(ClipboardService service)
    {
        _service = service;
        _service.OnItemAdded += OnItemAdded;
        _service.OnThumbnailReady += OnThumbnailReady;
        LoadItems();
    }

    public void Initialize(Window window)
    {
        ArgumentNullException.ThrowIfNull(window);
        _window = window;
        _dispatcherQueue = window.DispatcherQueue;
    }

    private void ReloadItems()
    {
        Items.Clear();
        _hasMoreItems = true;
        LoadItems();
    }

    private void LoadItems()
    {
        var query = string.IsNullOrWhiteSpace(SearchQuery) ? null : SearchQuery;
        var items = _service.GetHistory(UIConfig.PageSize, 0, query, CurrentPinnedFilter);

        foreach (var item in items)
        {
            Items.Add(new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));
        }
    }

    public void LoadMoreItems()
    {
        if (_isLoading || !_hasMoreItems) return;

        _isLoading = true;
        IsLoadingMore = true;

        _dispatcherQueue?.TryEnqueue(() =>
        {
            var query = string.IsNullOrWhiteSpace(SearchQuery) ? null : SearchQuery;
            var newItems = _service.GetHistory(UIConfig.PageSize, Items.Count, query, CurrentPinnedFilter).ToList();

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
        });
    }

    private void OnItemAdded(ClipboardItem item) =>
        _dispatcherQueue?.TryEnqueue(() =>
        {
            // Only add new items if we're on "Recientes" tab and not searching
            if (SelectedTabIndex == 0 && string.IsNullOrWhiteSpace(SearchQuery))
            {
                Items.Insert(0, new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));
            }
        });

    private void OnThumbnailReady(ClipboardItem item) =>
        _dispatcherQueue?.TryEnqueue(() =>
        {
            var existingVm = Items.FirstOrDefault(vm => vm.Model.Id == item.Id);
            if (existingVm != null)
            {
                var index = Items.IndexOf(existingVm);
                Items[index] = new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem);
            }
        });

    private void OnDeleteItem(ClipboardItemViewModel itemVM)
    {
        _service.RemoveItem(itemVM.Model.Id);
        Items.Remove(itemVM);
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types")]
    private async void OnPasteItem(ClipboardItemViewModel itemVM, bool plain)
    {
        try
        {
            // Verify file availability for file-based types
            if (itemVM.IsFileType && !itemVM.IsFileAvailable)
            {
                Debug.WriteLine($"Cannot paste: file not available for item {itemVM.Model.Id}");
                return;
            }

            // Notify service that we're pasting to prevent duplicate detection
            _service.NotifyPasteInitiated(itemVM.Model.Id);

            // Set content to Windows clipboard
            var success = ClipboardHelper.SetClipboardContent(itemVM.Model, plain);
            if (!success)
            {
                Debug.WriteLine($"Failed to set clipboard content for item {itemVM.Model.Id}");
                return;
            }

            // Mark item as used (updates ModifiedAt timestamp)
            var updatedItem = _service.MarkItemUsed(itemVM.Model.Id);
            if (updatedItem != null)
            {
                // Update the model with new timestamp
                itemVM.Model.ModifiedAt = updatedItem.ModifiedAt;

                // Move item to the top of the list in UI
                MoveItemToTop(itemVM);
            }

            // Hide window and restore focus to previous window, then simulate Ctrl+V
            HideWindow();
            await FocusHelper.RestoreAndPasteAsync().ConfigureAwait(false);
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Paste operation failed: {ex.Message}");
        }
    }

    private void MoveItemToTop(ClipboardItemViewModel itemVM) =>
        _dispatcherQueue?.TryEnqueue(() =>
        {
            var currentIndex = Items.IndexOf(itemVM);
            if (currentIndex > 0)
            {
                Items.Move(currentIndex, 0);
            }
        });

    private void HideWindow()
    {
        if (_window == null) return;

        var hWnd = WindowNative.GetWindowHandle(_window);
        var appWindow = AppWindow.GetFromWindowId(Win32Interop.GetWindowIdFromWindow(hWnd));
        appWindow.Hide();
    }

    private void OnPinItem(ClipboardItemViewModel itemVM)
    {
        _service.UpdatePin(itemVM.Model.Id, itemVM.IsPinned);
        // Reload items when pin status changes since item moves between tabs
        ReloadItems();
    }

    public void OnWindowDeactivated()
    {
        if (Items.Count <= UIConfig.MaxItemsBeforeCleanup) return;

        // Reset to initial page size and clear search when window is deactivated
        SearchQuery = string.Empty;

        while (Items.Count > UIConfig.PageSize)
        {
            Items.RemoveAt(Items.Count - 1);
        }

        _hasMoreItems = true;
        GC.Collect(2, GCCollectionMode.Optimized, false);
    }

    [RelayCommand]
    private void ClearSearch() => SearchQuery = string.Empty;

    [RelayCommand]
    private void ClearAll()
    {
        for (int i = Items.Count - 1; i >= 0; i--)
        {
            if (!Items[i].IsPinned)
            {
                _service.RemoveItem(Items[i].Model.Id);
                Items.RemoveAt(i);
            }
        }
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
