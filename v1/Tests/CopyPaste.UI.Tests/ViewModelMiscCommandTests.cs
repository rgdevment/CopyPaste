using CopyPaste.Core;
using CopyPaste.UI.Themes;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class ViewModelClearSearchCommandTests
{
    private static DefaultThemeViewModel CreateViewModel(IClipboardService? service = null)
    {
        service ??= new ClearSearchEmptyStub();
        return new DefaultThemeViewModel(service, new MyMConfig(), new DefaultThemeSettings());
    }

    [Fact]
    public void ClearSearchCommand_ClearsNonEmptySearchQuery()
    {
        var vm = CreateViewModel();
        vm.ToggleColorFilter(CardColor.Red);
        vm.SearchQuery = "clipboard";

        vm.ClearSearchCommand.Execute(null);

        Assert.Equal(string.Empty, vm.SearchQuery);
    }

    [Fact]
    public void ClearSearchCommand_WhenAlreadyEmpty_DoesNotThrow()
    {
        var vm = CreateViewModel();
        vm.SearchQuery = string.Empty;

        var exception = Record.Exception(() => vm.ClearSearchCommand.Execute(null));

        Assert.Null(exception);
        Assert.Equal(string.Empty, vm.SearchQuery);
    }

    [Fact]
    public void ClearSearchCommand_FiresPropertyChangedForSearchQuery()
    {
        var vm = CreateViewModel();
        vm.SearchQuery = "find me";
        var changedProps = new List<string?>();
        vm.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.ClearSearchCommand.Execute(null);

        Assert.Contains(nameof(vm.SearchQuery), changedProps);
    }

    [Fact]
    public void ClearSearchCommand_WithWhitespaceQuery_ClearsToEmpty()
    {
        var vm = CreateViewModel();
        vm.SearchQuery = "   ";

        vm.ClearSearchCommand.Execute(null);

        Assert.Equal(string.Empty, vm.SearchQuery);
    }
}

public sealed class ViewModelRefreshFileAvailabilityTests
{
    private static DefaultThemeViewModel CreateWithItems(List<ClipboardItem> items)
    {
        var service = new FileAvailabilityItemsStub(items);
        var config = new MyMConfig { PageSize = 50, MaxItemsBeforeCleanup = 100 };
        var vm = new DefaultThemeViewModel(service, config, new DefaultThemeSettings());
        vm.ToggleColorFilter(CardColor.Red); // triggers initial load
        return vm;
    }

    [Fact]
    public void RefreshFileAvailability_WithEmptyItems_DoesNotThrow()
    {
        var vm = CreateWithItems([]);

        var exception = Record.Exception(() => vm.RefreshFileAvailability());

        Assert.Null(exception);
    }

    [Fact]
    public void RefreshFileAvailability_WithOnlyTextItems_DoesNotThrow()
    {
        var items = new List<ClipboardItem>
        {
            new() { Content = "hello", Type = ClipboardContentType.Text },
            new() { Content = "world", Type = ClipboardContentType.Text },
        };
        var vm = CreateWithItems(items);

        var exception = Record.Exception(() => vm.RefreshFileAvailability());

        Assert.Null(exception);
    }

    [Fact]
    public void RefreshFileAvailability_OnFileTypeItem_FiresFileWarningVisibilityPropertyChanged()
    {
        var fileItem = new ClipboardItem
        {
            Content = @"C:\this\does\not\exist\file.txt",
            Type = ClipboardContentType.File
        };
        var vm = CreateWithItems([fileItem]);
        var fileItemVM = vm.Items.First(i => i.IsFileType);
        var changedProps = new List<string?>();
        fileItemVM.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.RefreshFileAvailability();

        Assert.Contains(nameof(ClipboardItemViewModel.FileWarningVisibility), changedProps);
    }

    [Fact]
    public void RefreshFileAvailability_OnFolderTypeItem_FiresPropertyChanged()
    {
        var folderItem = new ClipboardItem
        {
            Content = @"C:\this\does\not\exist\",
            Type = ClipboardContentType.Folder
        };
        var vm = CreateWithItems([folderItem]);
        var folderItemVM = vm.Items.First(i => i.IsFileType);
        var changedProps = new List<string?>();
        folderItemVM.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.RefreshFileAvailability();

        Assert.NotEmpty(changedProps);
    }

    [Fact]
    public void RefreshFileAvailability_TextItemsAreSkipped_NoPropertyChangedFired()
    {
        var textItem = new ClipboardItem { Content = "plain text", Type = ClipboardContentType.Text };
        var vm = CreateWithItems([textItem]);
        var textItemVM = vm.Items.First();
        var changedProps = new List<string?>();
        textItemVM.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.RefreshFileAvailability();

        Assert.Empty(changedProps);
    }

    [Fact]
    public void RefreshFileAvailability_MixedItems_OnlyFileTypesRefreshed()
    {
        var fileItem = new ClipboardItem
        {
            Content = @"C:\nonexistent.txt",
            Type = ClipboardContentType.File
        };
        var textItem = new ClipboardItem { Content = "text", Type = ClipboardContentType.Text };
        var vm = CreateWithItems([fileItem, textItem]);

        var fileItemVM = vm.Items.First(i => i.IsFileType);
        var textItemVM = vm.Items.First(i => !i.IsFileType);

        var fileChangedProps = new List<string?>();
        var textChangedProps = new List<string?>();
        fileItemVM.PropertyChanged += (_, e) => fileChangedProps.Add(e.PropertyName);
        textItemVM.PropertyChanged += (_, e) => textChangedProps.Add(e.PropertyName);

        vm.RefreshFileAvailability();

        Assert.NotEmpty(fileChangedProps);
        Assert.Empty(textChangedProps);
    }
}

public sealed class ViewModelCallbackTests
{
    private static (DefaultThemeViewModel vm, ViewModelCallbackStubService service) CreateVmWithItems(
        List<ClipboardItem> items)
    {
        var service = new ViewModelCallbackStubService(items);
        var config = new MyMConfig { PageSize = 50, MaxItemsBeforeCleanup = 100 };
        var vm = new DefaultThemeViewModel(service, config, new DefaultThemeSettings());
        vm.ToggleColorFilter(CardColor.Red); // trigger load
        return (vm, service);
    }

    [Fact]
    public void IsWindowPinned_DefaultThemeViewModel_ReturnsFalse()
    {
        var service = new ViewModelCallbackStubService([]);
        var vm = new DefaultThemeViewModel(service, new MyMConfig(), new DefaultThemeSettings());

        Assert.False(vm.IsWindowPinned);
    }

    [Fact]
    public void DeleteCommand_OnLoadedItem_RemovesItemFromList()
    {
        var items = new List<ClipboardItem>
        {
            new() { Content = "item1", Type = ClipboardContentType.Text },
            new() { Content = "item2", Type = ClipboardContentType.Text }
        };
        var (vm, _) = CreateVmWithItems(items);
        var toDelete = vm.Items[0];

        toDelete.DeleteCommand.Execute(null);

        Assert.Single(vm.Items);
    }

    [Fact]
    public void DeleteCommand_OnLoadedItem_CallsServiceRemove()
    {
        var item = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var (vm, service) = CreateVmWithItems([item]);
        var toDelete = vm.Items[0];

        toDelete.DeleteCommand.Execute(null);

        Assert.Contains(item.Id, service.RemovedIds);
    }

    [Fact]
    public void DeleteCommand_LastItem_SetsIsEmptyTrue()
    {
        var item = new ClipboardItem { Content = "only", Type = ClipboardContentType.Text };
        var (vm, _) = CreateVmWithItems([item]);
        var toDelete = vm.Items[0];

        toDelete.DeleteCommand.Execute(null);

        Assert.True(vm.IsEmpty);
    }

    [Fact]
    public void TogglePinCommand_OnLoadedItem_CallsServiceUpdatePin()
    {
        var item = new ClipboardItem { Content = "pinme", Type = ClipboardContentType.Text, IsPinned = false };
        var (vm, service) = CreateVmWithItems([item]);
        var toPin = vm.Items[0];

        toPin.TogglePinCommand.Execute(null);

        Assert.Contains(item.Id, service.PinnedIds);
    }

    [Fact]
    public void EditCommand_OnLoadedItem_FiresOnEditRequestedEvent()
    {
        var item = new ClipboardItem { Content = "edit me", Type = ClipboardContentType.Text };
        var (vm, _) = CreateVmWithItems([item]);
        var toEdit = vm.Items[0];
        ClipboardItemViewModel? receivedVM = null;
        vm.OnEditRequested += (_, e) => receivedVM = e;

        toEdit.EditCommand.Execute(null);

        Assert.NotNull(receivedVM);
        Assert.Equal(item.Id, receivedVM.Model.Id);
    }

    [Fact]
    public void SelectedTabIndex_SetTo2_LoadsAllItemsWithoutFilter()
    {
        var items = new List<ClipboardItem>
        {
            new() { Content = "pinned", Type = ClipboardContentType.Text, IsPinned = true },
            new() { Content = "unpinned", Type = ClipboardContentType.Text, IsPinned = false }
        };
        var (vm, _) = CreateVmWithItems(items);

        // SelectedTabIndex=2 triggers CurrentPinnedFilter = null (all items)
        var exception = Record.Exception(() => vm.SelectedTabIndex = 2);

        Assert.Null(exception);
    }
}

internal sealed class ViewModelCallbackStubService : IClipboardService
{
    private readonly List<ClipboardItem> _items;
    public List<Guid> RemovedIds { get; } = [];
    public List<Guid> PinnedIds { get; } = [];

    public ViewModelCallbackStubService(List<ClipboardItem> items) => _items = items;

#pragma warning disable CS0067
    public event Action<ClipboardItem>? OnItemAdded;
    public event Action<ClipboardItem>? OnThumbnailReady;
    public event Action<ClipboardItem>? OnItemReactivated;
#pragma warning restore CS0067
    public int PasteIgnoreWindowMs { get; set; } = 450;

    public IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query,
        IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned)
        => _items.Skip(skip).Take(limit);

    public void RemoveItem(Guid id) => RemovedIds.Add(id);
    public void UpdatePin(Guid id, bool isPinned) => PinnedIds.Add(id);
    public void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null, byte[]? htmlBytes = null) { }
    public void AddImage(byte[]? dibData, string? source) { }
    public void AddFiles(System.Collections.ObjectModel.Collection<string>? files, ClipboardContentType type, string? source) { }
    public IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? query = null, bool? isPinned = null) => [];
    public void UpdateLabelAndColor(Guid id, string? label, CardColor color) { }
    public ClipboardItem? MarkItemUsed(Guid id) => null;
    public void NotifyPasteInitiated(Guid itemId) { }
}

internal sealed class ClearSearchEmptyStub : IClipboardService
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

internal sealed class FileAvailabilityItemsStub : IClipboardService
{
    private readonly List<ClipboardItem> _items;

    public FileAvailabilityItemsStub(List<ClipboardItem> items) => _items = items;

#pragma warning disable CS0067
    public event Action<ClipboardItem>? OnItemAdded;
    public event Action<ClipboardItem>? OnThumbnailReady;
    public event Action<ClipboardItem>? OnItemReactivated;
#pragma warning restore CS0067
    public int PasteIgnoreWindowMs { get; set; } = 450;

    public IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query, IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned)
        => _items.Skip(skip).Take(limit);

    public void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null, byte[]? htmlBytes = null) { }
    public void AddImage(byte[]? dibData, string? source) { }
    public void AddFiles(Collection<string>? files, ClipboardContentType type, string? source) { }
    public IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? query = null, bool? isPinned = null) => [];
    public void RemoveItem(Guid id) { }
    public void UpdatePin(Guid id, bool isPinned) { }
    public void UpdateLabelAndColor(Guid id, string? label, CardColor color) { }
    public ClipboardItem? MarkItemUsed(Guid id) => null;
    public void NotifyPasteInitiated(Guid itemId) { }
}
