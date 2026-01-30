using CopyPaste.Core;
using Xunit;

namespace CopyPaste.UI.Tests;

/// <summary>
/// Basic tests for MainViewModel (limited due to WinUI3/DispatcherQueue dependencies).
/// </summary>
public class MainViewModelBasicTests
{
    [Fact]
    public void Constructor_WithValidService_CreatesInstance()
    {
        var service = new ClipboardService(new StubRepository());

        var viewModel = new ViewModels.MainViewModel(service);

        Assert.NotNull(viewModel);
        Assert.NotNull(viewModel.Items);
    }

    [Fact]
    public void Items_InitialState_IsEmpty()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        Assert.Empty(viewModel.Items);
    }

    [Fact]
    public void IsEmpty_InitialState_IsTrue()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        Assert.True(viewModel.IsEmpty);
    }

    [Fact]
    public void SearchQuery_InitialState_IsEmpty()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        Assert.Equal(string.Empty, viewModel.SearchQuery);
    }

    [Fact]
    public void HasSearchQuery_InitialState_IsFalse()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        Assert.False(viewModel.HasSearchQuery);
    }

    [Fact]
    public void SelectedTabIndex_InitialState_IsZero()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        Assert.Equal(0, viewModel.SelectedTabIndex);
    }

    [Fact]
    public void SearchQuery_WhenSet_UpdatesHasSearchQuery()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        viewModel.SearchQuery = "test";

        Assert.True(viewModel.HasSearchQuery);
    }

    [Fact]
    public void SearchQuery_WhenSetToEmpty_UpdatesHasSearchQueryToFalse()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        viewModel.SearchQuery = "test";
        viewModel.SearchQuery = "";

        Assert.False(viewModel.HasSearchQuery);
    }

    [Fact]
    public void SelectedTabIndex_CanBeChanged()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        viewModel.SelectedTabIndex = 1;

        Assert.Equal(1, viewModel.SelectedTabIndex);
    }

    [Fact]
    public void Cleanup_CanBeCalled()
    {
        var service = new ClipboardService(new StubRepository());
        var viewModel = new ViewModels.MainViewModel(service);

        viewModel.Cleanup();

        // Should not throw
        Assert.True(true);
    }

    private sealed class StubRepository : IClipboardRepository
    {
        public void Save(ClipboardItem item) { }
        public void Update(ClipboardItem item) { }
        public void Delete(Guid id) { }
        public ClipboardItem? GetById(Guid id) => null;
        public ClipboardItem? GetLatest() => null;
        public ClipboardItem? FindByContentAndType(string content, ClipboardContentType type) => null;
        public IEnumerable<ClipboardItem> GetAll() => [];
        public IEnumerable<ClipboardItem> Search(string query, int limit = 50, int skip = 0) => [];
        public int ClearOldItems(int days, bool excludePinned = true) => 0;
    }
}
