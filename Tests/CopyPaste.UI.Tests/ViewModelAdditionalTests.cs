using System.Collections.ObjectModel;
using CopyPaste.Core;
using CopyPaste.UI.Themes;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class DefaultThemeViewModelTests
{
    private static DefaultThemeViewModel CreateViewModel(IClipboardService? service = null, DefaultThemeSettings? settings = null)
    {
        service ??= new StubClipboardService();
        settings ??= new DefaultThemeSettings();
        return new DefaultThemeViewModel(service, new MyMConfig(), settings);
    }

    [Fact]
    public void Constructor_SetsDefaults()
    {
        var vm = CreateViewModel();

        Assert.True(vm.IsEmpty);
        Assert.Equal(string.Empty, vm.SearchQuery);
        Assert.False(vm.HasSearchQuery);
        Assert.Equal(0, vm.SelectedTabIndex);
        Assert.Equal(0, vm.ActiveFilterMode);
        Assert.False(vm.IsLoadingMore);
    }

    [Fact]
    public void IsWindowPinned_Default_IsFalse()
    {
        var vm = CreateViewModel();

        Assert.False(vm.IsWindowPinned);
    }

    [Fact]
    public void IsWindowPinned_WhenPinWindowTrue_ReturnsTrue()
    {
        var settings = new DefaultThemeSettings { PinWindow = true };
        var vm = CreateViewModel(settings: settings);

        Assert.True(vm.IsWindowPinned);
    }

    [Fact]
    public void SearchQuery_Set_UpdatesHasSearchQuery()
    {
        var vm = CreateViewModel();

        vm.SearchQuery = "hello";

        Assert.True(vm.HasSearchQuery);
    }

    [Fact]
    public void SearchQuery_Clear_UpdatesHasSearchQuery()
    {
        var vm = CreateViewModel();
        vm.SearchQuery = "hello";

        vm.SearchQuery = string.Empty;

        Assert.False(vm.HasSearchQuery);
    }

    [Fact]
    public void IsContentFilterMode_DefaultIsTrue()
    {
        var vm = CreateViewModel();

        Assert.True(vm.IsContentFilterMode);
    }

    [Fact]
    public void IsCategoryFilterMode_DefaultIsFalse()
    {
        var vm = CreateViewModel();

        Assert.False(vm.IsCategoryFilterMode);
    }

    [Fact]
    public void IsTypeFilterMode_DefaultIsFalse()
    {
        var vm = CreateViewModel();

        Assert.False(vm.IsTypeFilterMode);
    }

    [Fact]
    public void ActiveFilterMode_Set1_IsCategoryTrue()
    {
        var vm = CreateViewModel();

        vm.ActiveFilterMode = 1;

        Assert.False(vm.IsContentFilterMode);
        Assert.True(vm.IsCategoryFilterMode);
        Assert.False(vm.IsTypeFilterMode);
    }

    [Fact]
    public void ActiveFilterMode_Set2_IsTypeTrue()
    {
        var vm = CreateViewModel();

        vm.ActiveFilterMode = 2;

        Assert.False(vm.IsContentFilterMode);
        Assert.False(vm.IsCategoryFilterMode);
        Assert.True(vm.IsTypeFilterMode);
    }

    [Fact]
    public void Items_InitiallyEmpty()
    {
        var vm = CreateViewModel();

        Assert.Empty(vm.Items);
    }

    #region Filter Tests (IsColorSelected / ToggleColorFilter / IsTypeSelected / ToggleTypeFilter)

    [Fact]
    public void IsColorSelected_InitialState_ReturnsFalse()
    {
        var vm = CreateViewModel();

        Assert.False(vm.IsColorSelected(CardColor.Red));
        Assert.False(vm.IsColorSelected(CardColor.Green));
    }

    [Fact]
    public void ToggleColorFilter_SelectsColor()
    {
        var vm = CreateViewModel();

        vm.ToggleColorFilter(CardColor.Red);

        Assert.True(vm.IsColorSelected(CardColor.Red));
        Assert.False(vm.IsColorSelected(CardColor.Green));
    }

    [Fact]
    public void ToggleColorFilter_Twice_DeselectsColor()
    {
        var vm = CreateViewModel();
        vm.ToggleColorFilter(CardColor.Red);

        vm.ToggleColorFilter(CardColor.Red);

        Assert.False(vm.IsColorSelected(CardColor.Red));
    }

    [Fact]
    public void ClearColorFilters_DeselectsAll()
    {
        var vm = CreateViewModel();
        vm.ToggleColorFilter(CardColor.Red);
        vm.ToggleColorFilter(CardColor.Green);

        vm.ClearColorFilters();

        Assert.False(vm.IsColorSelected(CardColor.Red));
        Assert.False(vm.IsColorSelected(CardColor.Green));
    }

    [Fact]
    public void IsTypeSelected_InitialState_ReturnsFalse()
    {
        var vm = CreateViewModel();

        Assert.False(vm.IsTypeSelected(ClipboardContentType.Text));
        Assert.False(vm.IsTypeSelected(ClipboardContentType.Image));
    }

    [Fact]
    public void ToggleTypeFilter_SelectsType()
    {
        var vm = CreateViewModel();

        vm.ToggleTypeFilter(ClipboardContentType.Text);

        Assert.True(vm.IsTypeSelected(ClipboardContentType.Text));
        Assert.False(vm.IsTypeSelected(ClipboardContentType.Image));
    }

    [Fact]
    public void ToggleTypeFilter_Twice_DeselectsType()
    {
        var vm = CreateViewModel();
        vm.ToggleTypeFilter(ClipboardContentType.Text);

        vm.ToggleTypeFilter(ClipboardContentType.Text);

        Assert.False(vm.IsTypeSelected(ClipboardContentType.Text));
    }

    [Fact]
    public void ClearTypeFilters_DeselectsAll()
    {
        var vm = CreateViewModel();
        vm.ToggleTypeFilter(ClipboardContentType.Text);
        vm.ToggleTypeFilter(ClipboardContentType.Image);

        vm.ClearTypeFilters();

        Assert.False(vm.IsTypeSelected(ClipboardContentType.Text));
        Assert.False(vm.IsTypeSelected(ClipboardContentType.Image));
    }

    #endregion

    #region OpenRepoCommand Tests

    [Fact]
    public void OpenRepoCommand_IsNotNull()
    {
        var vm = CreateViewModel();

        Assert.NotNull(vm.OpenRepoCommand);
    }

    [Fact]
    public void OpenRepoCommand_CanExecute_ReturnsTrue()
    {
        var vm = CreateViewModel();

        Assert.True(vm.OpenRepoCommand.CanExecute(null));
    }

    #endregion

    private sealed class StubClipboardService : IClipboardService
    {
        public event Action<ClipboardItem>? OnItemAdded;
        public event Action<ClipboardItem>? OnThumbnailReady;
        public event Action<ClipboardItem>? OnItemReactivated;
        public List<Guid> RemovedIds { get; } = [];
        public void AddText(string? content, ClipboardContentType type, string? appSource, byte[]? rtfBytes = null, byte[]? htmlBytes = null) { }
        public void AddImage(byte[]? dibData, string? appSource) { }
        public void AddFiles(Collection<string>? filePaths, ClipboardContentType type, string? appSource) { }
        public IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? searchQuery = null, bool? isPinned = null) => [];
        public IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query, IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned) => [];
        public void RemoveItem(Guid id) => RemovedIds.Add(id);
        public void UpdatePin(Guid id, bool isPinned) { }
        public void UpdateLabelAndColor(Guid id, string? label, CardColor color) { }
        public ClipboardItem? MarkItemUsed(Guid id) => null;
        public void NotifyPasteInitiated(Guid itemId) { }
        public int PasteIgnoreWindowMs { get; set; } = 450;
        public void FireEvents() { OnItemAdded?.Invoke(null!); OnThumbnailReady?.Invoke(null!); OnItemReactivated?.Invoke(null!); }
    }
}

public sealed class CompactViewModelTests
{
    private static CompactViewModel CreateViewModel(IClipboardService? service = null, CompactSettings? settings = null)
    {
        service ??= new StubClipboardService();
        settings ??= new CompactSettings();
        return new CompactViewModel(service, new MyMConfig(), settings);
    }

    [Fact]
    public void Constructor_SetsDefaults()
    {
        var vm = CreateViewModel();

        Assert.True(vm.IsEmpty);
        Assert.Equal(string.Empty, vm.SearchQuery);
        Assert.False(vm.HasSearchQuery);
        Assert.Equal(0, vm.SelectedTabIndex);
        Assert.Equal(0, vm.ActiveFilterMode);
    }

    [Fact]
    public void IsWindowPinned_Default_IsFalse()
    {
        var vm = CreateViewModel();

        Assert.False(vm.IsWindowPinned);
    }

    [Fact]
    public void IsWindowPinned_WhenPinWindowTrue_ReturnsTrue()
    {
        var settings = new CompactSettings { PinWindow = true };
        var vm = CreateViewModel(settings: settings);

        Assert.True(vm.IsWindowPinned);
    }

    [Fact]
    public void Items_InitiallyEmpty()
    {
        var vm = CreateViewModel();

        Assert.Empty(vm.Items);
    }

    private sealed class StubClipboardService : IClipboardService
    {
        public event Action<ClipboardItem>? OnItemAdded;
        public event Action<ClipboardItem>? OnThumbnailReady;
        public event Action<ClipboardItem>? OnItemReactivated;
        public List<Guid> RemovedIds { get; } = [];
        public void AddText(string? content, ClipboardContentType type, string? appSource, byte[]? rtfBytes = null, byte[]? htmlBytes = null) { }
        public void AddImage(byte[]? dibData, string? appSource) { }
        public void AddFiles(Collection<string>? filePaths, ClipboardContentType type, string? appSource) { }
        public IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? searchQuery = null, bool? isPinned = null) => [];
        public IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query, IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned) => [];
        public void RemoveItem(Guid id) => RemovedIds.Add(id);
        public void UpdatePin(Guid id, bool isPinned) { }
        public void UpdateLabelAndColor(Guid id, string? label, CardColor color) { }
        public ClipboardItem? MarkItemUsed(Guid id) => null;
        public void NotifyPasteInitiated(Guid itemId) { }
        public int PasteIgnoreWindowMs { get; set; } = 450;
        public void FireEvents() { OnItemAdded?.Invoke(null!); OnThumbnailReady?.Invoke(null!); OnItemReactivated?.Invoke(null!); }
    }
}
