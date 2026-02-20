using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CopyPaste.Core;
using System.Threading.Tasks;
using CopyPaste.UI.Helpers;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using WinRT.Interop;

namespace CopyPaste.UI.Themes;

public abstract partial class ClipboardThemeViewModelBase : ObservableObject
{
    private readonly IClipboardService _service;
    private readonly MyMConfig _config;
    private readonly int _cardMaxLines;
    private readonly int _cardMinLines;
    private Window? _window;
    private DispatcherQueue? _dispatcherQueue;
    private bool _isLoading;
    private bool _hasMoreItems = true;

    protected IClipboardService Service => _service;

    public virtual bool IsWindowPinned => false;

    protected ClipboardThemeViewModelBase(IClipboardService service, MyMConfig config, int cardMaxLines, int cardMinLines)
    {
        _service = service;
        _config = config;
        _cardMaxLines = cardMaxLines;
        _cardMinLines = cardMinLines;
    }

    public ObservableCollection<ClipboardItemViewModel> Items { get; } = [];

    [ObservableProperty]
    public partial bool IsEmpty { get; set; } = true;

    [ObservableProperty]
    public partial bool IsLoadingMore { get; set; }

    [ObservableProperty]
    public partial string SearchQuery { get; set; } = string.Empty;

    [ObservableProperty]
    public partial bool HasSearchQuery { get; set; }

    [ObservableProperty]
    public partial int SelectedTabIndex { get; set; }

    [ObservableProperty]
    public partial int ActiveFilterMode { get; set; }

    private readonly HashSet<CardColor> _selectedColors = [];
    private readonly HashSet<ClipboardContentType> _selectedTypes = [];

    public bool IsContentFilterMode => ActiveFilterMode == 0;
    public bool IsCategoryFilterMode => ActiveFilterMode == 1;
    public bool IsTypeFilterMode => ActiveFilterMode == 2;

    private bool? CurrentPinnedFilter => SelectedTabIndex switch
    {
        0 => false,
        1 => true,
        _ => null
    };

    partial void OnSelectedTabIndexChanged(int value) => ReloadItems();

    partial void OnActiveFilterModeChanged(int value)
    {
        OnPropertyChanged(nameof(IsContentFilterMode));
        OnPropertyChanged(nameof(IsCategoryFilterMode));
        OnPropertyChanged(nameof(IsTypeFilterMode));
        ReloadItems();
    }

    partial void OnSearchQueryChanged(string value)
    {
        HasSearchQuery = !string.IsNullOrWhiteSpace(value);
        ReloadItems();
    }

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

    protected void ReloadItems()
    {
        Items.Clear();
        _hasMoreItems = true;
        LoadItems();
        UpdateIsEmpty();
    }

    private void LoadItems()
    {
        var query = IsContentFilterMode ? GetSearchQuery() : null;
        var types = IsTypeFilterMode && _selectedTypes.Count > 0 ? _selectedTypes : null;
        var colors = IsCategoryFilterMode && _selectedColors.Count > 0 ? _selectedColors : null;

        var items = _service.GetHistoryAdvanced(
            _config.PageSize,
            0,
            query,
            types,
            colors,
            CurrentPinnedFilter);

        foreach (var item in items)
            Items.Add(CreateViewModel(item));

        UpdateIsEmpty();
    }

    private void UpdateIsEmpty() => IsEmpty = Items.Count == 0;

    public void LoadMoreItems()
    {
        if (_isLoading || !_hasMoreItems) return;

        _isLoading = true;
        IsLoadingMore = true;

        var query = IsContentFilterMode ? GetSearchQuery() : null;
        var types = IsTypeFilterMode && _selectedTypes.Count > 0 ? _selectedTypes : null;
        var colors = IsCategoryFilterMode && _selectedColors.Count > 0 ? _selectedColors : null;

        var newItems = _service.GetHistoryAdvanced(
            _config.PageSize,
            Items.Count,
            query,
            types,
            colors,
            CurrentPinnedFilter).ToList();

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
    }

    private void OnItemAdded(ClipboardItem item)
    {
        if (_dispatcherQueue is null) return;

        _dispatcherQueue.TryEnqueue(() =>
        {
            if (ShouldShowInCurrentView())
            {
                Items.Insert(0, CreateViewModel(item));
                UpdateIsEmpty();
            }
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
        UpdateIsEmpty();
    }

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
                itemVM.Model.PasteCount = updatedItem.PasteCount;
                itemVM.RefreshPasteCount();
                MoveItemToTop(itemVM);
            }

            if (!IsWindowPinned)
                HideWindow();

            await FocusHelper.RestoreAndPasteAsync(
                _config.DelayBeforeFocusMs,
                _config.MaxFocusVerifyAttempts,
                _config.DelayBeforePasteMs).ConfigureAwait(false);
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
            {
                Items.Move(currentIndex, 0);
                OnScrollToTopRequested?.Invoke(this, EventArgs.Empty);
            }
        });
    }

    protected void HideWindow()
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
        if (Items.Count <= _config.MaxItemsBeforeCleanup) return;

        SearchQuery = string.Empty;

        while (Items.Count > _config.PageSize)
            Items.RemoveAt(Items.Count - 1);

        _hasMoreItems = true;

        GC.Collect(0, GCCollectionMode.Optimized, false);
    }

    [RelayCommand]
    private void ClearSearch() => SearchQuery = string.Empty;

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
        Win32WindowHelper.SetForegroundWindow(hWnd);
    }

    [RelayCommand]
    public static void Exit()
    {
        if (Application.Current is App app)
            app.BeginExit();
        else
            Application.Current.Exit();
    }

    internal event EventHandler<ClipboardItemViewModel>? OnEditRequested;

    internal event EventHandler? OnScrollToTopRequested;

    private void OnEditItem(ClipboardItemViewModel itemVM) => OnEditRequested?.Invoke(this, itemVM);

    public void SaveItemLabelAndColor(ClipboardItemViewModel itemVM, string? label, CardColor color)
    {
        ArgumentNullException.ThrowIfNull(itemVM);

        _service.UpdateLabelAndColor(itemVM.Model.Id, label, color);
        itemVM.Model.Label = label;
        itemVM.Model.CardColor = color;
        itemVM.RefreshLabelAndColor();
    }

    public void RefreshFileAvailability()
    {
        foreach (var item in Items.Where(i => i.IsFileType))
            item.RefreshFileStatus();
    }

    private string? GetSearchQuery() =>
        string.IsNullOrWhiteSpace(SearchQuery) ? null : SearchQuery;

    public bool IsColorSelected(CardColor color) => _selectedColors.Contains(color);
    public bool IsTypeSelected(ClipboardContentType type) => _selectedTypes.Contains(type);

    public void ToggleColorFilter(CardColor color)
    {
        if (!_selectedColors.Add(color))
            _selectedColors.Remove(color);
        ReloadItems();
    }

    public void ToggleTypeFilter(ClipboardContentType type)
    {
        if (!_selectedTypes.Add(type))
            _selectedTypes.Remove(type);
        ReloadItems();
    }

    public void ClearColorFilters()
    {
        if (_selectedColors.Count == 0) return;
        _selectedColors.Clear();
        ReloadItems();
    }

    public void ClearTypeFilters()
    {
        if (_selectedTypes.Count == 0) return;
        _selectedTypes.Clear();
        ReloadItems();
    }

    public void ResetFilters(bool resetMode, bool content, bool category, bool type)
    {
        var needsReload = false;

        if (resetMode)
        {
            if (ActiveFilterMode != 0)
                ActiveFilterMode = 0;

            if (!string.IsNullOrEmpty(SearchQuery))
            {
                SearchQuery = string.Empty;
                needsReload = true;
            }

            if (_selectedColors.Count > 0)
            {
                _selectedColors.Clear();
                needsReload = true;
            }

            if (_selectedTypes.Count > 0)
            {
                _selectedTypes.Clear();
                needsReload = true;
            }
        }
        else
        {
            if (content && !string.IsNullOrEmpty(SearchQuery))
            {
                SearchQuery = string.Empty;
                needsReload = true;
            }

            if (category && _selectedColors.Count > 0)
            {
                _selectedColors.Clear();
                needsReload = true;
            }

            if (type && _selectedTypes.Count > 0)
            {
                _selectedTypes.Clear();
                needsReload = true;
            }
        }

        if (needsReload)
            ReloadItems();
    }

    private bool ShouldShowInCurrentView() =>
        SelectedTabIndex is 0 && string.IsNullOrWhiteSpace(SearchQuery);

    private bool HasActiveFilter =>
        !string.IsNullOrWhiteSpace(SearchQuery) ||
        _selectedColors.Count > 0 ||
        _selectedTypes.Count > 0;

    private ClipboardItemViewModel CreateViewModel(ClipboardItem item) =>
        new(item, OnDeleteItem, OnPasteItem, OnPinItem, OnEditItem, HasActiveFilter,
            _cardMaxLines, _cardMinLines);
}
