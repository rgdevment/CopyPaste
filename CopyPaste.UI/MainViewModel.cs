using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using CopyPaste.UI.Helpers;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using WinRT.Interop;

namespace CopyPaste.UI.ViewModels;

public partial class MainViewModel(ClipboardService service) : ObservableObject
{
    private readonly ClipboardService _service = service;
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
        0 => false,
        1 => true,
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

    public void Initialize(Window window)
    {
        ArgumentNullException.ThrowIfNull(window);
        _window = window;
        _dispatcherQueue = window.DispatcherQueue;

        _service.OnItemAdded += OnItemAdded;
        _service.OnThumbnailReady += OnThumbnailReady;
        _service.OnItemReactivated += OnItemReactivated;

        LoadItems();
    }

    public void Cleanup()
    {
        _service.OnItemAdded -= OnItemAdded;
        _service.OnThumbnailReady -= OnThumbnailReady;
        _service.OnItemReactivated -= OnItemReactivated;
    }

    private void ReloadItems()
    {
        Items.Clear();
        _hasMoreItems = true;
        LoadItems();
    }

    private void LoadItems()
    {
        var query = GetSearchQuery();
        var items = _service.GetHistory(UIConfig.PageSize, 0, query, CurrentPinnedFilter);

        foreach (var item in items)
            Items.Add(CreateViewModel(item));
    }

    public void LoadMoreItems()
    {
        if (_isLoading || !_hasMoreItems || _dispatcherQueue is null) return;

        _isLoading = true;
        IsLoadingMore = true;

        _dispatcherQueue.TryEnqueue(() =>
        {
            var query = GetSearchQuery();
            var newItems = _service.GetHistory(UIConfig.PageSize, Items.Count, query, CurrentPinnedFilter).ToList();

            if (newItems.Count is 0)
            {
                _hasMoreItems = false;
            }
            else
            {
                foreach (var item in newItems)
                    Items.Add(CreateViewModel(item));
            }

            _isLoading = false;
            IsLoadingMore = false;
        });
    }

    private void OnItemAdded(ClipboardItem item)
    {
        if (_dispatcherQueue is null) return;

        _dispatcherQueue.TryEnqueue(() =>
        {
            if (ShouldShowInCurrentView())
                Items.Insert(0, CreateViewModel(item));
        });
    }

    private void OnItemReactivated(ClipboardItem item)
    {
        if (_dispatcherQueue is null) return;

        _dispatcherQueue.TryEnqueue(() =>
        {
            var existingVm = Items.FirstOrDefault(vm => vm.Model.Id == item.Id);

            if (existingVm is not null)
            {
                existingVm.Model.ModifiedAt = item.ModifiedAt;
                MoveItemToTop(existingVm);
            }
            else if (ShouldShowInCurrentView())
            {
                Items.Insert(0, CreateViewModel(item));
            }
        });
    }

    private void OnThumbnailReady(ClipboardItem item)
    {
        if (_dispatcherQueue is null) return;

        _dispatcherQueue.TryEnqueue(() =>
        {
            var existingVm = Items.FirstOrDefault(vm => vm.Model.Id == item.Id);
            existingVm?.RefreshFromModel(item);
        });
    }

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
            if (itemVM.IsFileType && !itemVM.IsFileAvailable)
            {
                AppLogger.Warn($"Cannot paste: file not available for item {itemVM.Model.Id}");
                return;
            }

            _service.NotifyPasteInitiated(itemVM.Model.Id);

            if (!ClipboardHelper.SetClipboardContent(itemVM.Model, plain))
            {
                AppLogger.Warn($"Failed to set clipboard content for item {itemVM.Model.Id}");
                return;
            }

            if (_service.MarkItemUsed(itemVM.Model.Id) is { } updatedItem)
            {
                itemVM.Model.ModifiedAt = updatedItem.ModifiedAt;
                MoveItemToTop(itemVM);
            }

            HideWindow();
            await FocusHelper.RestoreAndPasteAsync().ConfigureAwait(false);
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Paste operation failed");
        }
    }

    private void MoveItemToTop(ClipboardItemViewModel itemVM)
    {
        if (_dispatcherQueue is null) return;

        _dispatcherQueue.TryEnqueue(() =>
        {
            var currentIndex = Items.IndexOf(itemVM);
            if (currentIndex > 0)
                Items.Move(currentIndex, 0);
        });
    }

    private void HideWindow()
    {
        if (_window is null) return;

        var hWnd = WindowNative.GetWindowHandle(_window);
        var appWindow = AppWindow.GetFromWindowId(Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hWnd));
        appWindow.Hide();
    }

    private void OnPinItem(ClipboardItemViewModel itemVM)
    {
        _service.UpdatePin(itemVM.Model.Id, itemVM.IsPinned);
        ReloadItems();
    }

    public void OnWindowDeactivated()
    {
        if (Items.Count <= UIConfig.MaxItemsBeforeCleanup) return;

        SearchQuery = string.Empty;

        while (Items.Count > UIConfig.PageSize)
            Items.RemoveAt(Items.Count - 1);

        _hasMoreItems = true;

        GC.Collect(0, GCCollectionMode.Optimized, false);
    }

    [RelayCommand]
    private void ClearSearch() => SearchQuery = string.Empty;

    [RelayCommand]
    private void ClearAll()
    {
        for (int i = Items.Count - 1; i >= 0; i--)
        {
            if (Items[i].IsPinned) continue;

            _service.RemoveItem(Items[i].Model.Id);
            Items.RemoveAt(i);
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
        if (_window is null) return;

        _window.Activate();
        var hWnd = WindowNative.GetWindowHandle(_window);
        SetForegroundWindow(hWnd);
    }

    [RelayCommand]
    public static void Exit()
    {
        if (Application.Current is App app)
            app.BeginExit();
        else
            Application.Current.Exit();
    }

    private string? GetSearchQuery() =>
        string.IsNullOrWhiteSpace(SearchQuery) ? null : SearchQuery;

    private bool ShouldShowInCurrentView() =>
        SelectedTabIndex is 0 && string.IsNullOrWhiteSpace(SearchQuery);

    private ClipboardItemViewModel CreateViewModel(ClipboardItem item) =>
        new(item, OnDeleteItem, OnPasteItem, OnPinItem);
}
