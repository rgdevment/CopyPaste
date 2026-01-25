using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks; // Required for Task
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using Microsoft.UI.Xaml;

namespace CopyPaste.UI.ViewModels;

public partial class MainViewModel : ObservableObject
{
    private readonly ClipboardService _service;
    private Window? _window;

    public ObservableCollection<ClipboardItemViewModel> Items { get; } = [];

    public MainViewModel(ClipboardService service)
    {
        _service = service;
        LoadHistory();
    }

    public void Initialize(Window window) => _window = window;

    private void LoadHistory()
    {
        Items.Clear();
        foreach (var item in _service.GetHistory(20))
        {
            Items.Add(new ClipboardItemViewModel(item, OnDeleteItem, OnPasteItem, OnPinItem));
        }
    }

    private void OnDeleteItem(ClipboardItemViewModel itemVM) => Items.Remove(itemVM);
    private void OnPasteItem(ClipboardItemViewModel itemVM, bool plain) { /* Paste logic */ }
    private void OnPinItem(ClipboardItemViewModel itemVM) { /* Pin logic */ }

    [RelayCommand]
    private void ClearAll()
    {
        for (int i = Items.Count - 1; i >= 0; i--)
        {
            if (!Items[i].IsPinned) Items.RemoveAt(i);
        }
    }

    // FIXED: Changed 'void' to 'Task' to fix MVVMTK0039
    [RelayCommand]
    private static async Task OpenRepo()
    {
        var uri = new Uri("https://github.com/rgdevment/CopyPaste");
        await Windows.System.Launcher.LaunchUriAsync(uri);
    }

    [RelayCommand] public void ShowWindow() => _window?.Activate();
    [RelayCommand] public static void Exit() => Application.Current.Exit();
}
