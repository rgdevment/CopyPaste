using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;

namespace CopyPaste.UI.ViewModels;

public partial class MainViewModel : ObservableObject
{
    private readonly ClipboardService _service;
    private Window? _window;
    private DispatcherQueue? _dispatcherQueue;
    private const int _pageSize = 20;
    private const int _maxItemsBeforeCleanup = 100;
    private bool _isLoading;
    private bool _hasMoreItems = true;

    public ObservableCollection<ClipboardItemViewModel> Items { get; } = [];

    [ObservableProperty]
    public partial bool IsLoadingMore { get; set; }

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
        foreach (var item in _service.GetHistory(_pageSize))
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
            var newItems = _service.GetHistory(_pageSize, Items.Count).ToList();

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
            Items.Insert(0, new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));
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

    private void OnDeleteItem(ClipboardItemViewModel itemVM) => Items.Remove(itemVM);
    private void OnPasteItem(ClipboardItemViewModel itemVM, bool plain) { }
    private void OnPinItem(ClipboardItemViewModel itemVM) { }

    public void OnWindowDeactivated()
    {
        if (Items.Count <= _maxItemsBeforeCleanup) return;

        while (Items.Count > _pageSize)
        {
            Items.RemoveAt(Items.Count - 1);
        }

        _hasMoreItems = true;
        GC.Collect(2, GCCollectionMode.Optimized, false);
    }

    [RelayCommand]
    private void ClearAll()
    {
        for (int i = Items.Count - 1; i >= 0; i--)
        {
            if (!Items[i].IsPinned) Items.RemoveAt(i);
        }
    }

    [RelayCommand]
    private static async Task OpenRepo()
    {
        var uri = new Uri("https://github.com/rgdevment/CopyPaste/issues");
        await Windows.System.Launcher.LaunchUriAsync(uri);
    }

    [RelayCommand] public void ShowWindow() => _window?.Activate();
    [RelayCommand] public static void Exit() => Application.Current.Exit();
}
