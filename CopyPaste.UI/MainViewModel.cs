using System;
using System.Collections.ObjectModel;
using System.Linq;
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

    [ObservableProperty]
    public partial bool IsLoadingMore { get; set; }

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
        if (Items.Count <= UIConfig.MaxItemsBeforeCleanup) return;

        while (Items.Count > UIConfig.PageSize)
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
    [RelayCommand] public static void Exit()
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
