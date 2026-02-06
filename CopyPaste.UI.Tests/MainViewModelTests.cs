using CopyPaste.Core;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class MainViewModelBasicTests
{
    private static ViewModels.MainViewModel CreateViewModel(IClipboardService? service = null)
    {
        service ??= new ClipboardService(new StubRepository());
        return new ViewModels.MainViewModel(service, new MyMConfig());
    }

    [Fact]
    public void Constructor_WithValidService_CreatesInstance()
    {
        var viewModel = CreateViewModel();

        Assert.NotNull(viewModel);
        Assert.NotNull(viewModel.Items);
    }

    [Fact]
    public void Items_InitialState_IsEmpty()
    {
        var viewModel = CreateViewModel();

        Assert.Empty(viewModel.Items);
    }

    [Fact]
    public void IsEmpty_InitialState_IsTrue()
    {
        var viewModel = CreateViewModel();

        Assert.True(viewModel.IsEmpty);
    }

    [Fact]
    public void SearchQuery_InitialState_IsEmpty()
    {
        var viewModel = CreateViewModel();

        Assert.Equal(string.Empty, viewModel.SearchQuery);
    }

    [Fact]
    public void HasSearchQuery_InitialState_IsFalse()
    {
        var viewModel = CreateViewModel();

        Assert.False(viewModel.HasSearchQuery);
    }

    [Fact]
    public void SelectedTabIndex_InitialState_IsZero()
    {
        var viewModel = CreateViewModel();

        Assert.Equal(0, viewModel.SelectedTabIndex);
    }

    [Fact]
    public void SearchQuery_WhenSet_UpdatesHasSearchQuery()
    {
        var viewModel = CreateViewModel();

        viewModel.SearchQuery = "test";

        Assert.True(viewModel.HasSearchQuery);
    }

    [Fact]
    public void SearchQuery_WhenSetToEmpty_UpdatesHasSearchQueryToFalse()
    {
        var viewModel = CreateViewModel();

        viewModel.SearchQuery = "test";
        viewModel.SearchQuery = "";

        Assert.False(viewModel.HasSearchQuery);
    }

    [Fact]
    public void SelectedTabIndex_CanBeChanged()
    {
        var viewModel = CreateViewModel();

        viewModel.SelectedTabIndex = 1;

        Assert.Equal(1, viewModel.SelectedTabIndex);
    }

    [Fact]
    public void Cleanup_CanBeCalled()
    {
        var viewModel = CreateViewModel();

        viewModel.Cleanup();

        Assert.True(true);
    }

    [Fact]
    public void ActiveFilterMode_InitialState_IsZero()
    {
        var viewModel = CreateViewModel();

        Assert.Equal(0, viewModel.ActiveFilterMode);
    }

    [Fact]
    public void IsContentFilterMode_WhenActiveFilterModeIsZero_ReturnsTrue()
    {
        var viewModel = CreateViewModel();

        viewModel.ActiveFilterMode = 0;

        Assert.True(viewModel.IsContentFilterMode);
        Assert.False(viewModel.IsCategoryFilterMode);
        Assert.False(viewModel.IsTypeFilterMode);
    }

    [Fact]
    public void IsCategoryFilterMode_WhenActiveFilterModeIsOne_ReturnsTrue()
    {
        var viewModel = CreateViewModel();

        viewModel.ActiveFilterMode = 1;

        Assert.False(viewModel.IsContentFilterMode);
        Assert.True(viewModel.IsCategoryFilterMode);
        Assert.False(viewModel.IsTypeFilterMode);
    }

    [Fact]
    public void IsTypeFilterMode_WhenActiveFilterModeIsTwo_ReturnsTrue()
    {
        var viewModel = CreateViewModel();

        viewModel.ActiveFilterMode = 2;

        Assert.False(viewModel.IsContentFilterMode);
        Assert.False(viewModel.IsCategoryFilterMode);
        Assert.True(viewModel.IsTypeFilterMode);
    }

    [Fact]
    public void ToggleColorFilter_AddsAndRemovesColor()
    {
        var viewModel = CreateViewModel();

        viewModel.ToggleColorFilter(CardColor.Red);
        Assert.True(viewModel.IsColorSelected(CardColor.Red));

        viewModel.ToggleColorFilter(CardColor.Red);
        Assert.False(viewModel.IsColorSelected(CardColor.Red));
    }

    [Fact]
    public void ToggleTypeFilter_AddsAndRemovesType()
    {
        var viewModel = CreateViewModel();

        viewModel.ToggleTypeFilter(ClipboardContentType.Text);
        Assert.True(viewModel.IsTypeSelected(ClipboardContentType.Text));

        viewModel.ToggleTypeFilter(ClipboardContentType.Text);
        Assert.False(viewModel.IsTypeSelected(ClipboardContentType.Text));
    }

    [Fact]
    public void ClearColorFilters_RemovesAllColors()
    {
        var viewModel = CreateViewModel();

        viewModel.ToggleColorFilter(CardColor.Red);
        viewModel.ToggleColorFilter(CardColor.Blue);
        Assert.True(viewModel.IsColorSelected(CardColor.Red));
        Assert.True(viewModel.IsColorSelected(CardColor.Blue));

        viewModel.ClearColorFilters();

        Assert.False(viewModel.IsColorSelected(CardColor.Red));
        Assert.False(viewModel.IsColorSelected(CardColor.Blue));
    }

    [Fact]
    public void ClearTypeFilters_RemovesAllTypes()
    {
        var viewModel = CreateViewModel();

        viewModel.ToggleTypeFilter(ClipboardContentType.Text);
        viewModel.ToggleTypeFilter(ClipboardContentType.Image);
        Assert.True(viewModel.IsTypeSelected(ClipboardContentType.Text));
        Assert.True(viewModel.IsTypeSelected(ClipboardContentType.Image));

        viewModel.ClearTypeFilters();

        Assert.False(viewModel.IsTypeSelected(ClipboardContentType.Text));
        Assert.False(viewModel.IsTypeSelected(ClipboardContentType.Image));
    }

    [Fact]
    public void MultipleColorFilters_CanBeAddedAndChecked()
    {
        var viewModel = CreateViewModel();

        viewModel.ToggleColorFilter(CardColor.Red);
        viewModel.ToggleColorFilter(CardColor.Blue);
        viewModel.ToggleColorFilter(CardColor.Green);

        Assert.True(viewModel.IsColorSelected(CardColor.Red));
        Assert.True(viewModel.IsColorSelected(CardColor.Blue));
        Assert.True(viewModel.IsColorSelected(CardColor.Green));
        Assert.False(viewModel.IsColorSelected(CardColor.Yellow));
        Assert.False(viewModel.IsColorSelected(CardColor.Purple));
    }

    [Fact]
    public void MultipleTypeFilters_CanBeAddedAndChecked()
    {
        var viewModel = CreateViewModel();

        viewModel.ToggleTypeFilter(ClipboardContentType.Text);
        viewModel.ToggleTypeFilter(ClipboardContentType.Image);
        viewModel.ToggleTypeFilter(ClipboardContentType.Link);

        Assert.True(viewModel.IsTypeSelected(ClipboardContentType.Text));
        Assert.True(viewModel.IsTypeSelected(ClipboardContentType.Image));
        Assert.True(viewModel.IsTypeSelected(ClipboardContentType.Link));
        Assert.False(viewModel.IsTypeSelected(ClipboardContentType.File));
        Assert.False(viewModel.IsTypeSelected(ClipboardContentType.Folder));
    }

    [Theory]
    [InlineData(0, true, false, false)]
    [InlineData(1, false, true, false)]
    [InlineData(2, false, false, true)]
    public void FilterMode_PropertiesReflectActiveMode(int mode, bool isContent, bool isCategory, bool isType)
    {
        var viewModel = CreateViewModel();

        viewModel.ActiveFilterMode = mode;

        Assert.Equal(isContent, viewModel.IsContentFilterMode);
        Assert.Equal(isCategory, viewModel.IsCategoryFilterMode);
        Assert.Equal(isType, viewModel.IsTypeFilterMode);
    }

    [Fact]
    public void ToggleColorFilter_WithNoneColor_StillToggles()
    {
        // Note: CardColor.None is not filtered out, so it will be added
        var viewModel = CreateViewModel();

        viewModel.ToggleColorFilter(CardColor.None);
        Assert.True(viewModel.IsColorSelected(CardColor.None));

        viewModel.ToggleColorFilter(CardColor.None);
        Assert.False(viewModel.IsColorSelected(CardColor.None));
    }

    private sealed class StubRepository : IClipboardRepository
    {
        public void Save(ClipboardItem item) { }
        public void Update(ClipboardItem item) { }
        public void Delete(Guid id) { }
        public ClipboardItem? GetById(Guid id) => null;
        public ClipboardItem? GetLatest() => null;
        public ClipboardItem? FindByContentAndType(string content, ClipboardContentType type) => null;
        public ClipboardItem? FindByContentHash(string contentHash) => null;
        public IEnumerable<ClipboardItem> GetAll() => [];
        public IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0) => [];
        public int ClearOldItems(int days, bool excludePinned = true) => 0;
        public IEnumerable<ClipboardItem> SearchAdvanced(string? query, IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned, int limit, int skip) => [];
    }
}
