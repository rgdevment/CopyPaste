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

    public ObservableCollection<ClipboardItemViewModel> Items { get; } = [];

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
        foreach (var item in _service.GetHistory(20))
        {
            Items.Add(new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));
        }
    }

    private void OnItemAdded(ClipboardItem item) =>
        _dispatcherQueue?.TryEnqueue(() =>
        {
            Items.Insert(0, new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));

            // Limit visible items
            while (Items.Count > 20)
            {
                Items.RemoveAt(Items.Count - 1);
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

    private void OnDeleteItem(ClipboardItemViewModel itemVM) => Items.Remove(itemVM);
    private void OnPasteItem(ClipboardItemViewModel itemVM, bool plain) { }
    private void OnPinItem(ClipboardItemViewModel itemVM) { }

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
