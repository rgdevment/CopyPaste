using CopyPaste.Core;
using CopyPaste.UI.Themes;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class ViewModelResetFiltersTests
{
    private static DefaultThemeViewModel CreateViewModel(IClipboardService? service = null)
    {
        service ??= new EmptyStubService();
        return new DefaultThemeViewModel(service, new MyMConfig(), new DefaultThemeSettings());
    }

    [Fact]
    public void ResetFilters_WithResetModeTrue_ClearsSearchQuery()
    {
        var vm = CreateViewModel();
        vm.SearchQuery = "hello";

        vm.ResetFilters(resetMode: true, content: false, category: false, type: false);

        Assert.Equal(string.Empty, vm.SearchQuery);
    }

    [Fact]
    public void ResetFilters_WithResetModeTrue_ClearsColorFilter()
    {
        var vm = CreateViewModel();
        vm.ToggleColorFilter(CardColor.Red);

        vm.ResetFilters(resetMode: true, content: false, category: false, type: false);

        Assert.False(vm.IsColorSelected(CardColor.Red));
    }

    [Fact]
    public void ResetFilters_WithResetModeTrue_ClearsTypeFilter()
    {
        var vm = CreateViewModel();
        vm.ToggleTypeFilter(ClipboardContentType.Text);

        vm.ResetFilters(resetMode: true, content: false, category: false, type: false);

        Assert.False(vm.IsTypeSelected(ClipboardContentType.Text));
    }

    [Fact]
    public void ResetFilters_WithResetModeTrue_ResetsFilterModeToZero()
    {
        var vm = CreateViewModel();
        vm.ActiveFilterMode = 1;

        vm.ResetFilters(resetMode: true, content: false, category: false, type: false);

        Assert.Equal(0, vm.ActiveFilterMode);
    }

    [Fact]
    public void ResetFilters_WithResetModeFalse_ContentOnly_ClearsOnlySearchQuery()
    {
        var vm = CreateViewModel();
        vm.SearchQuery = "hello";
        vm.ToggleColorFilter(CardColor.Blue);

        vm.ResetFilters(resetMode: false, content: true, category: false, type: false);

        Assert.Equal(string.Empty, vm.SearchQuery);
        Assert.True(vm.IsColorSelected(CardColor.Blue));
    }

    [Fact]
    public void ResetFilters_WithResetModeFalse_CategoryOnly_ClearsOnlyColors()
    {
        var vm = CreateViewModel();
        vm.SearchQuery = "hello";
        vm.ToggleColorFilter(CardColor.Red);

        vm.ResetFilters(resetMode: false, content: false, category: true, type: false);

        Assert.Equal("hello", vm.SearchQuery);
        Assert.False(vm.IsColorSelected(CardColor.Red));
    }

    [Fact]
    public void ResetFilters_WithResetModeFalse_TypeOnly_ClearsOnlyTypes()
    {
        var vm = CreateViewModel();
        vm.ToggleColorFilter(CardColor.Green);
        vm.ToggleTypeFilter(ClipboardContentType.Image);

        vm.ResetFilters(resetMode: false, content: false, category: false, type: true);

        Assert.True(vm.IsColorSelected(CardColor.Green));
        Assert.False(vm.IsTypeSelected(ClipboardContentType.Image));
    }

    [Fact]
    public void ResetFilters_NothingActive_DoesNotThrow()
    {
        var vm = CreateViewModel();

        var exception = Record.Exception(() =>
            vm.ResetFilters(resetMode: true, content: true, category: true, type: true));

        Assert.Null(exception);
        Assert.Equal(string.Empty, vm.SearchQuery);
        Assert.Equal(0, vm.ActiveFilterMode);
    }

    [Fact]
    public void ResetFilters_WithResetModeFalse_AllFlagsTrue_ClearsAllFilters()
    {
        var vm = CreateViewModel();
        vm.SearchQuery = "test";
        vm.ToggleColorFilter(CardColor.Red);
        vm.ToggleTypeFilter(ClipboardContentType.File);

        vm.ResetFilters(resetMode: false, content: true, category: true, type: true);

        Assert.Equal(string.Empty, vm.SearchQuery);
        Assert.False(vm.IsColorSelected(CardColor.Red));
        Assert.False(vm.IsTypeSelected(ClipboardContentType.File));
    }

    [Fact]
    public void ResetFilters_WithResetModeTrue_ClearsMultipleColors()
    {
        var vm = CreateViewModel();
        vm.ToggleColorFilter(CardColor.Red);
        vm.ToggleColorFilter(CardColor.Blue);

        vm.ResetFilters(resetMode: true, content: false, category: false, type: false);

        Assert.False(vm.IsColorSelected(CardColor.Red));
        Assert.False(vm.IsColorSelected(CardColor.Blue));
    }
}

public sealed class ViewModelLoadMoreItemsTests
{
    private static (DefaultThemeViewModel vm, ItemsStubService service) CreateVmWithItems(
        List<ClipboardItem> items, int pageSize = 2)
    {
        var service = new ItemsStubService(items);
        var config = new MyMConfig { PageSize = pageSize, MaxItemsBeforeCleanup = 100 };
        var vm = new DefaultThemeViewModel(service, config, new DefaultThemeSettings());
        return (vm, service);
    }

    [Fact]
    public void LoadMoreItems_AppendsNextPage()
    {
        var items = Enumerable.Range(0, 5)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVmWithItems(items);
        vm.ToggleColorFilter(CardColor.Red); // loads first page: 2 items

        vm.LoadMoreItems(); // loads second page: 2 more

        Assert.Equal(4, vm.Items.Count);
    }

    [Fact]
    public void LoadMoreItems_WhenEmptyResult_SetsNoMoreItems()
    {
        var items = Enumerable.Range(0, 2)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVmWithItems(items);
        vm.ToggleColorFilter(CardColor.Red); // loads items 0,1

        vm.LoadMoreItems(); // skip=2, returns empty → no more items
        vm.LoadMoreItems(); // should be no-op now

        Assert.Equal(2, vm.Items.Count);
    }

    [Fact]
    public void LoadMoreItems_MultiplePagesUntilExhausted_CountIsCorrect()
    {
        var items = Enumerable.Range(0, 4)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVmWithItems(items);
        vm.ToggleColorFilter(CardColor.Red); // 2 items

        vm.LoadMoreItems(); // +2, total 4
        vm.LoadMoreItems(); // returns empty, no more items
        vm.LoadMoreItems(); // no-op

        Assert.Equal(4, vm.Items.Count);
    }

    [Fact]
    public void LoadMoreItems_IsLoadingMore_FalseAfterCompletion()
    {
        var items = Enumerable.Range(0, 3)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVmWithItems(items);
        vm.ToggleColorFilter(CardColor.Red);

        vm.LoadMoreItems();

        Assert.False(vm.IsLoadingMore);
    }
}

public sealed class ViewModelWindowDeactivatedTests
{
    private static (DefaultThemeViewModel vm, ItemsStubService service) CreateVmWithSmallConfig(
        List<ClipboardItem> items, int pageSize = 2, int maxCleanup = 3)
    {
        var service = new ItemsStubService(items);
        var config = new MyMConfig { PageSize = pageSize, MaxItemsBeforeCleanup = maxCleanup };
        var vm = new DefaultThemeViewModel(service, config, new DefaultThemeSettings());
        return (vm, service);
    }

    [Fact]
    public void OnWindowDeactivated_BelowThreshold_DoesNotTrimItems()
    {
        var items = Enumerable.Range(0, 5)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVmWithSmallConfig(items, pageSize: 2, maxCleanup: 3);
        vm.ToggleColorFilter(CardColor.Red); // loads 2 items (< maxCleanup=3)

        vm.OnWindowDeactivated(); // 2 <= 3, no trim

        Assert.Equal(2, vm.Items.Count);
    }

    [Fact]
    public void OnWindowDeactivated_AboveThreshold_TrimsToPageSize()
    {
        var items = Enumerable.Range(0, 5)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVmWithSmallConfig(items, pageSize: 2, maxCleanup: 3);
        vm.ToggleColorFilter(CardColor.Red); // 2 items
        vm.LoadMoreItems();                  // +2 → 4 items (> maxCleanup=3)

        vm.OnWindowDeactivated();

        Assert.Equal(2, vm.Items.Count);
    }

    [Fact]
    public void OnWindowDeactivated_AboveThreshold_AllowsLoadMoreAfterwards()
    {
        var items = Enumerable.Range(0, 5)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVmWithSmallConfig(items, pageSize: 2, maxCleanup: 3);
        vm.ToggleColorFilter(CardColor.Red); // 2 items
        vm.LoadMoreItems();                  // 4 items
        vm.OnWindowDeactivated();            // trims to 2, resets _hasMoreItems = true

        vm.LoadMoreItems(); // should load 2 more from skip=2

        Assert.Equal(4, vm.Items.Count);
    }

    [Fact]
    public void OnWindowDeactivated_ExactlyAtThreshold_DoesNotTrim()
    {
        var items = Enumerable.Range(0, 5)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVmWithSmallConfig(items, pageSize: 2, maxCleanup: 2);
        vm.ToggleColorFilter(CardColor.Red); // loads exactly 2 items = maxCleanup=2

        vm.OnWindowDeactivated(); // 2 <= 2, no trim

        Assert.Equal(2, vm.Items.Count);
    }
}

public sealed class ViewModelSaveItemAndColorTests
{
    private static (DefaultThemeViewModel vm, ItemsStubService service) CreateVm()
    {
        var service = new ItemsStubService([]);
        var vm = new DefaultThemeViewModel(service, new MyMConfig(), new DefaultThemeSettings());
        return (vm, service);
    }

    [Fact]
    public void SaveItemLabelAndColor_UpdatesModelLabel()
    {
        var (vm, _) = CreateVm();
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var itemVM = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        vm.SaveItemLabelAndColor(itemVM, "myLabel", CardColor.Blue);

        Assert.Equal("myLabel", model.Label);
    }

    [Fact]
    public void SaveItemLabelAndColor_UpdatesModelColor()
    {
        var (vm, _) = CreateVm();
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var itemVM = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        vm.SaveItemLabelAndColor(itemVM, null, CardColor.Green);

        Assert.Equal(CardColor.Green, model.CardColor);
    }

    [Fact]
    public void SaveItemLabelAndColor_CallsServiceUpdate()
    {
        var (vm, service) = CreateVm();
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var itemVM = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        vm.SaveItemLabelAndColor(itemVM, "label", CardColor.Red);

        Assert.Single(service.LabelColorUpdates);
        var (id, label, color) = service.LabelColorUpdates[0];
        Assert.Equal(model.Id, id);
        Assert.Equal("label", label);
        Assert.Equal(CardColor.Red, color);
    }

    [Fact]
    public void SaveItemLabelAndColor_WithNullItemVM_ThrowsArgumentNullException()
    {
        var (vm, _) = CreateVm();

        Assert.Throws<ArgumentNullException>(() =>
            vm.SaveItemLabelAndColor(null!, "label", CardColor.None));
    }

    [Fact]
    public void SaveItemLabelAndColor_WithNullLabel_SetsLabelToNull()
    {
        var (vm, _) = CreateVm();
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text, Label = "existing" };
        var itemVM = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        vm.SaveItemLabelAndColor(itemVM, null, CardColor.None);

        Assert.Null(model.Label);
    }

    [Fact]
    public void SaveItemLabelAndColor_FiresPropertyChangedForLabel()
    {
        var (vm, _) = CreateVm();
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var itemVM = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });
        var changedProps = new List<string?>();
        itemVM.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.SaveItemLabelAndColor(itemVM, "newLabel", CardColor.Purple);

        Assert.Contains(nameof(ClipboardItemViewModel.Label), changedProps);
    }
}

public sealed class DefaultThemeViewModelClearAllTests
{
    private static (DefaultThemeViewModel vm, ItemsStubService service) CreateVmWithItems(
        List<ClipboardItem> items, int pageSize = 10)
    {
        var service = new ItemsStubService(items);
        var config = new MyMConfig { PageSize = pageSize, MaxItemsBeforeCleanup = 100 };
        var vm = new DefaultThemeViewModel(service, config, new DefaultThemeSettings());
        return (vm, service);
    }

    [Fact]
    public void ClearAll_RemovesUnpinnedItems_KeepsPinned()
    {
        var items = new List<ClipboardItem>
        {
            new() { Content = "pinned",    Type = ClipboardContentType.Text, IsPinned = true  },
            new() { Content = "unpinned1", Type = ClipboardContentType.Text, IsPinned = false },
            new() { Content = "unpinned2", Type = ClipboardContentType.Text, IsPinned = false },
        };
        var (vm, _) = CreateVmWithItems(items);
        vm.ToggleColorFilter(CardColor.Red); // loads all 3 items

        vm.ClearAllCommand.Execute(null);

        var remaining = Assert.Single(vm.Items);
        Assert.True(remaining.IsPinned);
    }

    [Fact]
    public void ClearAll_CallsServiceRemoveForEachUnpinned()
    {
        var unpinned1 = new ClipboardItem { Content = "a", Type = ClipboardContentType.Text, IsPinned = false };
        var unpinned2 = new ClipboardItem { Content = "b", Type = ClipboardContentType.Text, IsPinned = false };
        var pinned    = new ClipboardItem { Content = "c", Type = ClipboardContentType.Text, IsPinned = true  };
        var (vm, service) = CreateVmWithItems([unpinned1, unpinned2, pinned]);
        vm.ToggleColorFilter(CardColor.Red);

        vm.ClearAllCommand.Execute(null);

        Assert.Equal(2, service.RemovedIds.Count);
        Assert.Contains(unpinned1.Id, service.RemovedIds);
        Assert.Contains(unpinned2.Id, service.RemovedIds);
        Assert.DoesNotContain(pinned.Id, service.RemovedIds);
    }

    [Fact]
    public void ClearAll_WithAllPinned_RemovesNothing()
    {
        var items = new List<ClipboardItem>
        {
            new() { Content = "p1", Type = ClipboardContentType.Text, IsPinned = true },
            new() { Content = "p2", Type = ClipboardContentType.Text, IsPinned = true },
        };
        var (vm, service) = CreateVmWithItems(items);
        vm.ToggleColorFilter(CardColor.Red);

        vm.ClearAllCommand.Execute(null);

        Assert.Equal(2, vm.Items.Count);
        Assert.Empty(service.RemovedIds);
    }

    [Fact]
    public void ClearAll_WithAllUnpinned_RemovesAll()
    {
        var items = new List<ClipboardItem>
        {
            new() { Content = "u1", Type = ClipboardContentType.Text, IsPinned = false },
            new() { Content = "u2", Type = ClipboardContentType.Text, IsPinned = false },
            new() { Content = "u3", Type = ClipboardContentType.Text, IsPinned = false },
        };
        var (vm, service) = CreateVmWithItems(items);
        vm.ToggleColorFilter(CardColor.Red);

        vm.ClearAllCommand.Execute(null);

        Assert.Empty(vm.Items);
        Assert.Equal(3, service.RemovedIds.Count);
    }

    [Fact]
    public void ClearAll_EmptyItems_DoesNotThrow()
    {
        var (vm, service) = CreateVmWithItems([]);

        var exception = Record.Exception(() => vm.ClearAllCommand.Execute(null));

        Assert.Null(exception);
        Assert.Empty(service.RemovedIds);
    }
}

// Shared stubs used by all test classes above.

internal sealed class EmptyStubService : IClipboardService
{
#pragma warning disable CS0067
    public event Action<ClipboardItem>? OnItemAdded;
    public event Action<ClipboardItem>? OnThumbnailReady;
    public event Action<ClipboardItem>? OnItemReactivated;
#pragma warning restore CS0067
    public int PasteIgnoreWindowMs { get; set; } = 450;
    public void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null, byte[]? htmlBytes = null) { }
    public void AddImage(byte[]? dibData, string? source) { }
    public void AddFiles(Collection<string>? files, ClipboardContentType type, string? source) { }
    public IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? query = null, bool? isPinned = null) => [];
    public IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query, IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned) => [];
    public void RemoveItem(Guid id) { }
    public void UpdatePin(Guid id, bool isPinned) { }
    public void UpdateLabelAndColor(Guid id, string? label, CardColor color) { }
    public ClipboardItem? MarkItemUsed(Guid id) => null;
    public void NotifyPasteInitiated(Guid itemId) { }
}

internal sealed class ItemsStubService : IClipboardService
{
    private readonly List<ClipboardItem> _items;

    public List<(Guid id, string? label, CardColor color)> LabelColorUpdates { get; } = [];
    public List<Guid> RemovedIds { get; } = [];

    public ItemsStubService(List<ClipboardItem> items) => _items = items;

#pragma warning disable CS0067
    public event Action<ClipboardItem>? OnItemAdded;
    public event Action<ClipboardItem>? OnThumbnailReady;
    public event Action<ClipboardItem>? OnItemReactivated;
#pragma warning restore CS0067
    public int PasteIgnoreWindowMs { get; set; } = 450;

    public IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query, IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned)
        => _items.Skip(skip).Take(limit);

    public void UpdateLabelAndColor(Guid id, string? label, CardColor color)
        => LabelColorUpdates.Add((id, label, color));

    public void RemoveItem(Guid id) => RemovedIds.Add(id);

    public void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null, byte[]? htmlBytes = null) { }
    public void AddImage(byte[]? dibData, string? source) { }
    public void AddFiles(Collection<string>? files, ClipboardContentType type, string? source) { }
    public IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? query = null, bool? isPinned = null) => [];
    public void UpdatePin(Guid id, bool isPinned) { }
    public ClipboardItem? MarkItemUsed(Guid id) => null;
    public void NotifyPasteInitiated(Guid itemId) { }
}
