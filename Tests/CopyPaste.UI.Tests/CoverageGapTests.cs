using CopyPaste.Core;
using CopyPaste.UI.Themes;
using Microsoft.UI.Xaml;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using Xunit;

namespace CopyPaste.UI.Tests;

// ─────────────────────────────────────────────────────────────────────────────
// ClipboardItemViewModel.Timestamp — 3 branches: today / yesterday / older date
// ─────────────────────────────────────────────────────────────────────────────

public sealed class ClipboardItemViewModelTimestampTests
{
    private static ClipboardItemViewModel CreateVm(ClipboardItem model) =>
        new(model, _ => { }, (_, _) => { }, _ => { });

    [Fact]
    public void Timestamp_ForToday_IsNotEmpty()
    {
        var model = new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, CreatedAt = DateTime.UtcNow };
        var vm = CreateVm(model);

        Assert.NotEmpty(vm.Timestamp);
    }

    [Fact]
    public void Timestamp_ForToday_ContainsTodayKey()
    {
        // When L is not initialized it returns "[clipboard.timestamps.today]"
        var model = new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, CreatedAt = DateTime.UtcNow };
        var vm = CreateVm(model);

        var timestamp = vm.Timestamp;

        Assert.Contains("today", timestamp, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Timestamp_ForYesterday_IsNotEmpty()
    {
        var yesterday = DateTime.Now.AddDays(-1).ToUniversalTime();
        var model = new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, CreatedAt = yesterday };
        var vm = CreateVm(model);

        Assert.NotEmpty(vm.Timestamp);
    }

    [Fact]
    public void Timestamp_ForYesterday_ContainsYesterdayKey()
    {
        var yesterday = DateTime.Now.AddDays(-1).ToUniversalTime();
        var model = new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, CreatedAt = yesterday };
        var vm = CreateVm(model);

        var timestamp = vm.Timestamp;

        Assert.Contains("yesterday", timestamp, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Timestamp_ForOlderDate_DoesNotContainBrackets()
    {
        // A fixed date far in the past — definitely not today or yesterday.
        // The third branch returns date.ToString("dd MMM HH:mm") directly (no L.Get).
        var olderDate = new DateTime(2024, 1, 15, 10, 30, 0, DateTimeKind.Local);
        var model = new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, CreatedAt = olderDate.ToUniversalTime() };
        var vm = CreateVm(model);

        var timestamp = vm.Timestamp;

        Assert.NotEmpty(timestamp);
        Assert.DoesNotContain("[", timestamp, StringComparison.Ordinal);
    }

    [Fact]
    public void Timestamp_ForOlderDate_IsFormattedAsDdMmmHhMm()
    {
        var olderDate = new DateTime(2024, 1, 15, 10, 30, 0, DateTimeKind.Local);
        var model = new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, CreatedAt = olderDate.ToUniversalTime() };
        var vm = CreateVm(model);

        var timestamp = vm.Timestamp;

        Assert.Equal("15 Jan 10:30", timestamp);
    }

    [Fact]
    public void Timestamp_TodayVsOlder_AreDifferentBranches()
    {
        var today = DateTime.UtcNow;
        var older = new DateTime(2024, 1, 15, 10, 30, 0, DateTimeKind.Local).ToUniversalTime();

        var todayVm = CreateVm(new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, CreatedAt = today });
        var olderVm = CreateVm(new ClipboardItem { Content = "x", Type = ClipboardContentType.Text, CreatedAt = older });

        // today uses L.Get path → contains "[" when L not initialized
        // older uses direct date.ToString → never contains "["
        var todayTs = todayVm.Timestamp;
        var olderTs = olderVm.Timestamp;

        Assert.NotEqual(todayTs, olderTs);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OnWindowDeactivated — verifies SearchQuery is cleared when above threshold
// ─────────────────────────────────────────────────────────────────────────────

public sealed class ViewModelOnWindowDeactivatedSearchQueryTests
{
    private static DefaultThemeViewModel CreateVm(
        List<ClipboardItem> items, int pageSize = 2, int maxCleanup = 3)
    {
        var service = new ItemsStubService(items);
        var config = new MyMConfig { PageSize = pageSize, MaxItemsBeforeCleanup = maxCleanup };
        return new DefaultThemeViewModel(service, config, new DefaultThemeSettings());
    }

    [Fact]
    public void OnWindowDeactivated_AboveThreshold_ClearsSearchQuery()
    {
        // pageSize=10, maxCleanup=4: setting SearchQuery triggers a reload that
        // loads 10 items (10 > maxCleanup=4) → OnWindowDeactivated proceeds and clears it.
        var items = Enumerable.Range(0, 10)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var service = new ItemsStubService(items);
        var config = new MyMConfig { PageSize = 10, MaxItemsBeforeCleanup = 4 };
        var vm = new DefaultThemeViewModel(service, config, new DefaultThemeSettings());
        vm.SearchQuery = "some query"; // triggers ReloadItems → 10 items loaded (10 > maxCleanup=4)

        vm.OnWindowDeactivated();

        Assert.Equal(string.Empty, vm.SearchQuery);
    }

    [Fact]
    public void OnWindowDeactivated_BelowThreshold_DoesNotClearSearchQuery()
    {
        var items = Enumerable.Range(0, 5)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var vm = CreateVm(items, pageSize: 2, maxCleanup: 10);
        vm.ToggleColorFilter(CardColor.Red); // loads 2 items (2 <= 10 → below threshold)
        vm.SearchQuery = "should stay";

        vm.OnWindowDeactivated();

        Assert.Equal("should stay", vm.SearchQuery);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SelectedTabIndex filter behavior — verifies isPinned passed to GetHistoryAdvanced
// ─────────────────────────────────────────────────────────────────────────────

public sealed class ViewModelTabFilterBehaviorTests
{
    private static (DefaultThemeViewModel vm, TabFilterCapturingStub service) CreateVm(
        List<ClipboardItem>? items = null, int pageSize = 10)
    {
        items ??= [];
        var service = new TabFilterCapturingStub(items);
        var config = new MyMConfig { PageSize = pageSize, MaxItemsBeforeCleanup = 100 };
        var vm = new DefaultThemeViewModel(service, config, new DefaultThemeSettings());
        return (vm, service);
    }

    [Fact]
    public void SelectedTabIndex_Default_PassesPinnedFalseToService()
    {
        var (vm, service) = CreateVm();
        vm.ToggleColorFilter(CardColor.Red); // triggers initial load with SelectedTabIndex=0

        Assert.Equal(false, service.LastIsPinnedFilter);
    }

    [Fact]
    public void SelectedTabIndex_SetTo1_PassesPinnedTrueToService()
    {
        var (vm, service) = CreateVm();
        vm.ToggleColorFilter(CardColor.Red);

        vm.SelectedTabIndex = 1; // triggers ReloadItems → isPinned = true

        Assert.Equal(true, service.LastIsPinnedFilter);
    }

    [Fact]
    public void SelectedTabIndex_SetTo2_PassesNullPinnedFilterToService()
    {
        var (vm, service) = CreateVm();
        vm.ToggleColorFilter(CardColor.Red);

        vm.SelectedTabIndex = 2; // triggers ReloadItems → isPinned = null

        Assert.Null(service.LastIsPinnedFilter);
    }

    [Fact]
    public void SelectedTabIndex_BackTo0_PassesPinnedFalseAgain()
    {
        var (vm, service) = CreateVm();
        vm.ToggleColorFilter(CardColor.Red);
        vm.SelectedTabIndex = 1;

        vm.SelectedTabIndex = 0;

        Assert.Equal(false, service.LastIsPinnedFilter);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RefreshFromModel — all property change notifications
// ─────────────────────────────────────────────────────────────────────────────

public sealed class ClipboardItemViewModelRefreshFromModelNotificationTests
{
    private static ClipboardItemViewModel CreateVm(ClipboardItem model) =>
        new(model, _ => { }, (_, _) => { }, _ => { });

    private static List<string?> CollectChanges(ClipboardItemViewModel vm, Action act)
    {
        var props = new List<string?>();
        vm.PropertyChanged += (_, e) => props.Add(e.PropertyName);
        act();
        return props;
    }

    [Fact]
    public void RefreshFromModel_FiresThumbnailPathPropertyChanged()
    {
        var model = new ClipboardItem { Content = "vid.mp4", Type = ClipboardContentType.Video };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "new.mp4" }));

        Assert.Contains(nameof(ClipboardItemViewModel.ThumbnailPath), props);
    }

    [Fact]
    public void RefreshFromModel_FiresImagePathPropertyChanged()
    {
        var model = new ClipboardItem { Content = "img.png", Type = ClipboardContentType.Image };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "img2.png" }));

        Assert.Contains(nameof(ClipboardItemViewModel.ImagePath), props);
    }

    [Fact]
    public void RefreshFromModel_FiresHasValidImagePathPropertyChanged()
    {
        var model = new ClipboardItem { Content = "img.png", Type = ClipboardContentType.Image };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "img2.png" }));

        Assert.Contains(nameof(ClipboardItemViewModel.HasValidImagePath), props);
    }

    [Fact]
    public void RefreshFromModel_FiresImageVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "img.png", Type = ClipboardContentType.Image };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "img2.png" }));

        Assert.Contains(nameof(ClipboardItemViewModel.ImageVisibility), props);
    }

    [Fact]
    public void RefreshFromModel_FiresMediaThumbnailVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "vid.mp4", Type = ClipboardContentType.Video };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "vid2.mp4" }));

        Assert.Contains(nameof(ClipboardItemViewModel.MediaThumbnailVisibility), props);
    }

    [Fact]
    public void RefreshFromModel_FiresMediaDurationPropertyChanged()
    {
        var model = new ClipboardItem { Content = "vid.mp4", Type = ClipboardContentType.Video };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "vid2.mp4" }));

        Assert.Contains(nameof(ClipboardItemViewModel.MediaDuration), props);
    }

    [Fact]
    public void RefreshFromModel_FiresDurationVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "vid.mp4", Type = ClipboardContentType.Video };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "vid2.mp4" }));

        Assert.Contains(nameof(ClipboardItemViewModel.DurationVisibility), props);
    }

    [Fact]
    public void RefreshFromModel_FiresImageDimensionsPropertyChanged()
    {
        var model = new ClipboardItem { Content = "img.png", Type = ClipboardContentType.Image };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "img2.png", Metadata = """{"width":800,"height":600}""" }));

        Assert.Contains(nameof(ClipboardItemViewModel.ImageDimensions), props);
    }

    [Fact]
    public void RefreshFromModel_FiresImageDimensionsVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "img.png", Type = ClipboardContentType.Image };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "img2.png" }));

        Assert.Contains(nameof(ClipboardItemViewModel.ImageDimensionsVisibility), props);
    }

    [Fact]
    public void RefreshFromModel_FiresFileSizePropertyChanged()
    {
        var model = new ClipboardItem { Content = "file.txt", Type = ClipboardContentType.File };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "file2.txt" }));

        Assert.Contains(nameof(ClipboardItemViewModel.FileSize), props);
    }

    [Fact]
    public void RefreshFromModel_FiresFileSizeVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "file.txt", Type = ClipboardContentType.File };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "file2.txt" }));

        Assert.Contains(nameof(ClipboardItemViewModel.FileSizeVisibility), props);
    }

    [Fact]
    public void RefreshFromModel_FiresMediaInfoVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "vid.mp4", Type = ClipboardContentType.Video };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "vid2.mp4" }));

        Assert.Contains(nameof(ClipboardItemViewModel.MediaInfoVisibility), props);
    }

    [Fact]
    public void RefreshFromModel_FiresIsTextVisiblePropertyChanged()
    {
        var model = new ClipboardItem { Content = "hello", Type = ClipboardContentType.Text };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "world" }));

        Assert.Contains(nameof(ClipboardItemViewModel.IsTextVisible), props);
    }

    [Fact]
    public void RefreshFromModel_FiresIsFileAvailablePropertyChanged()
    {
        var model = new ClipboardItem { Content = "file.txt", Type = ClipboardContentType.File };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "file2.txt" }));

        Assert.Contains(nameof(ClipboardItemViewModel.IsFileAvailable), props);
    }

    [Fact]
    public void RefreshFromModel_FiresFileWarningVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "file.txt", Type = ClipboardContentType.File };
        var vm = CreateVm(model);
        var props = CollectChanges(vm, () => vm.RefreshFromModel(new ClipboardItem { Content = "file2.txt" }));

        Assert.Contains(nameof(ClipboardItemViewModel.FileWarningVisibility), props);
    }

    [Fact]
    public void RefreshFromModel_ClearsCachedThumbnailPath_AfterRefresh()
    {
        var thumbFile = System.IO.Path.GetTempFileName();
        try
        {
            var metadata = $$"""{"thumb_path": "{{thumbFile.Replace("\\", "\\\\", StringComparison.Ordinal)}}"}""";
            var model = new ClipboardItem { Content = "vid.mp4", Type = ClipboardContentType.Video, Metadata = metadata };
            var vm = CreateVm(model);
            var firstPath = vm.ThumbnailPath; // populate cache → tempFile
            Assert.Equal(thumbFile, firstPath);

            vm.RefreshFromModel(new ClipboardItem { Content = "vid2.mp4" }); // clears cache + no thumb_path

            var secondPath = vm.ThumbnailPath; // re-evaluates without cached value
            Assert.NotEqual(thumbFile, secondPath);
        }
        finally
        {
            System.IO.File.Delete(thumbFile);
        }
    }

    [Fact]
    public void RefreshFromModel_ClearsCachedImagePath_AfterRefresh()
    {
        var existingFile = System.IO.Path.GetTempFileName();
        try
        {
            var model = new ClipboardItem { Content = existingFile, Type = ClipboardContentType.Image };
            var vm = CreateVm(model);
            var firstPath = vm.ImagePath; // populate cache → existingFile
            Assert.Equal(existingFile, firstPath);

            vm.RefreshFromModel(new ClipboardItem { Content = @"C:\nonexistent\img.png" }); // clears cache

            var secondPath = vm.ImagePath; // re-evaluates — different content
            Assert.NotEqual(existingFile, secondPath);
        }
        finally
        {
            System.IO.File.Delete(existingFile);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CompactViewModel — comprehensive tests matching DefaultThemeViewModel coverage
// ─────────────────────────────────────────────────────────────────────────────

public sealed class CompactViewModelComprehensiveTests
{
    private static (CompactViewModel vm, ComprehensiveItemsStub service) CreateVm(
        List<ClipboardItem>? items = null, int pageSize = 10, int maxCleanup = 100)
    {
        items ??= [];
        var service = new ComprehensiveItemsStub(items);
        var config = new MyMConfig { PageSize = pageSize, MaxItemsBeforeCleanup = maxCleanup };
        var vm = new CompactViewModel(service, config, new CompactSettings());
        return (vm, service);
    }

    [Fact]
    public void SearchQuery_WhenSet_UpdatesHasSearchQuery()
    {
        var (vm, _) = CreateVm();
        vm.SearchQuery = "hello";
        Assert.True(vm.HasSearchQuery);
    }

    [Fact]
    public void SearchQuery_WhenCleared_UpdatesHasSearchQueryFalse()
    {
        var (vm, _) = CreateVm();
        vm.SearchQuery = "hello";
        vm.SearchQuery = string.Empty;
        Assert.False(vm.HasSearchQuery);
    }

    [Fact]
    public void SearchQuery_WhenWhitespaceOnly_HasSearchQueryIsFalse()
    {
        var (vm, _) = CreateVm();
        vm.SearchQuery = "   ";
        Assert.False(vm.HasSearchQuery);
    }

    [Fact]
    public void ActiveFilterMode_SetTo1_IsCategoryFilterModeTrue()
    {
        var (vm, _) = CreateVm();
        vm.ActiveFilterMode = 1;
        Assert.False(vm.IsContentFilterMode);
        Assert.True(vm.IsCategoryFilterMode);
        Assert.False(vm.IsTypeFilterMode);
    }

    [Fact]
    public void ActiveFilterMode_SetTo2_IsTypeFilterModeTrue()
    {
        var (vm, _) = CreateVm();
        vm.ActiveFilterMode = 2;
        Assert.False(vm.IsContentFilterMode);
        Assert.False(vm.IsCategoryFilterMode);
        Assert.True(vm.IsTypeFilterMode);
    }

    [Fact]
    public void ActiveFilterMode_SetBack0_IsContentFilterModeTrue()
    {
        var (vm, _) = CreateVm();
        vm.ActiveFilterMode = 2;
        vm.ActiveFilterMode = 0;
        Assert.True(vm.IsContentFilterMode);
        Assert.False(vm.IsCategoryFilterMode);
        Assert.False(vm.IsTypeFilterMode);
    }

    [Fact]
    public void ToggleColorFilter_SelectsColor()
    {
        var (vm, _) = CreateVm();
        vm.ToggleColorFilter(CardColor.Red);
        Assert.True(vm.IsColorSelected(CardColor.Red));
    }

    [Fact]
    public void ToggleColorFilter_Twice_DeselectsColor()
    {
        var (vm, _) = CreateVm();
        vm.ToggleColorFilter(CardColor.Blue);
        vm.ToggleColorFilter(CardColor.Blue);
        Assert.False(vm.IsColorSelected(CardColor.Blue));
    }

    [Fact]
    public void ClearColorFilters_DeselectsAllColors()
    {
        var (vm, _) = CreateVm();
        vm.ToggleColorFilter(CardColor.Red);
        vm.ToggleColorFilter(CardColor.Green);
        vm.ClearColorFilters();
        Assert.False(vm.IsColorSelected(CardColor.Red));
        Assert.False(vm.IsColorSelected(CardColor.Green));
    }

    [Fact]
    public void ClearColorFilters_WhenAlreadyEmpty_DoesNotThrow()
    {
        var (vm, _) = CreateVm();
        var ex = Record.Exception(() => vm.ClearColorFilters());
        Assert.Null(ex);
    }

    [Fact]
    public void ToggleTypeFilter_SelectsType()
    {
        var (vm, _) = CreateVm();
        vm.ToggleTypeFilter(ClipboardContentType.Image);
        Assert.True(vm.IsTypeSelected(ClipboardContentType.Image));
    }

    [Fact]
    public void ToggleTypeFilter_Twice_DeselectsType()
    {
        var (vm, _) = CreateVm();
        vm.ToggleTypeFilter(ClipboardContentType.Text);
        vm.ToggleTypeFilter(ClipboardContentType.Text);
        Assert.False(vm.IsTypeSelected(ClipboardContentType.Text));
    }

    [Fact]
    public void ClearTypeFilters_DeselectsAllTypes()
    {
        var (vm, _) = CreateVm();
        vm.ToggleTypeFilter(ClipboardContentType.Text);
        vm.ToggleTypeFilter(ClipboardContentType.Audio);
        vm.ClearTypeFilters();
        Assert.False(vm.IsTypeSelected(ClipboardContentType.Text));
        Assert.False(vm.IsTypeSelected(ClipboardContentType.Audio));
    }

    [Fact]
    public void ClearTypeFilters_WhenAlreadyEmpty_DoesNotThrow()
    {
        var (vm, _) = CreateVm();
        var ex = Record.Exception(() => vm.ClearTypeFilters());
        Assert.Null(ex);
    }

    [Fact]
    public void ResetFilters_WithResetModeTrue_ClearsAllFilters()
    {
        var (vm, _) = CreateVm();
        vm.SearchQuery = "test";
        vm.ToggleColorFilter(CardColor.Red);
        vm.ToggleTypeFilter(ClipboardContentType.Text);
        vm.ActiveFilterMode = 1;

        vm.ResetFilters(resetMode: true, content: false, category: false, type: false);

        Assert.Equal(string.Empty, vm.SearchQuery);
        Assert.Equal(0, vm.ActiveFilterMode);
        Assert.False(vm.IsColorSelected(CardColor.Red));
        Assert.False(vm.IsTypeSelected(ClipboardContentType.Text));
    }

    [Fact]
    public void ResetFilters_WithResetModeFalse_ContentOnly_ClearsOnlySearchQuery()
    {
        var (vm, _) = CreateVm();
        vm.SearchQuery = "hello";
        vm.ToggleColorFilter(CardColor.Blue);

        vm.ResetFilters(resetMode: false, content: true, category: false, type: false);

        Assert.Equal(string.Empty, vm.SearchQuery);
        Assert.True(vm.IsColorSelected(CardColor.Blue));
    }

    [Fact]
    public void LoadMoreItems_AppendsNextPage()
    {
        var items = Enumerable.Range(0, 5)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVm(items, pageSize: 2);
        vm.ToggleColorFilter(CardColor.Red); // loads first 2 items

        vm.LoadMoreItems(); // loads next 2

        Assert.Equal(4, vm.Items.Count);
    }

    [Fact]
    public void LoadMoreItems_WhenNoMoreItems_DoesNotAddMore()
    {
        var items = Enumerable.Range(0, 2)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVm(items, pageSize: 2);
        vm.ToggleColorFilter(CardColor.Red); // loads all 2 items

        vm.LoadMoreItems(); // returns empty → sets _hasMoreItems=false
        vm.LoadMoreItems(); // no-op

        Assert.Equal(2, vm.Items.Count);
    }

    [Fact]
    public void LoadMoreItems_IsLoadingMore_FalseAfterCompletion()
    {
        var items = Enumerable.Range(0, 3)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVm(items, pageSize: 2);
        vm.ToggleColorFilter(CardColor.Red);

        vm.LoadMoreItems();

        Assert.False(vm.IsLoadingMore);
    }

    [Fact]
    public void OnWindowDeactivated_AboveThreshold_TrimsToPageSize()
    {
        var items = Enumerable.Range(0, 5)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var (vm, _) = CreateVm(items, pageSize: 2, maxCleanup: 3);
        vm.ToggleColorFilter(CardColor.Red); // 2 items
        vm.LoadMoreItems();                  // +2 → 4 items (> maxCleanup=3)

        vm.OnWindowDeactivated();

        Assert.Equal(2, vm.Items.Count);
    }

    [Fact]
    public void OnWindowDeactivated_AboveThreshold_ClearsSearchQuery()
    {
        // pageSize=10, maxCleanup=4: setting SearchQuery triggers a reload that
        // loads 10 items (10 > maxCleanup=4) → OnWindowDeactivated proceeds and clears it.
        var items = Enumerable.Range(0, 10)
            .Select(i => new ClipboardItem { Content = $"item{i}", Type = ClipboardContentType.Text })
            .ToList();
        var service = new ComprehensiveItemsStub(items);
        var config = new MyMConfig { PageSize = 10, MaxItemsBeforeCleanup = 4 };
        var vm = new CompactViewModel(service, config, new CompactSettings());
        vm.SearchQuery = "some query"; // triggers ReloadItems → 10 items loaded (10 > maxCleanup=4)

        vm.OnWindowDeactivated();

        Assert.Equal(string.Empty, vm.SearchQuery);
    }

    [Fact]
    public void ClearSearchCommand_ClearsSearchQuery()
    {
        var (vm, _) = CreateVm();
        vm.SearchQuery = "find me";
        vm.ClearSearchCommand.Execute(null);
        Assert.Equal(string.Empty, vm.SearchQuery);
    }

    [Fact]
    public void SaveItemLabelAndColor_UpdatesModelLabelAndColor()
    {
        var (vm, _) = CreateVm();
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var itemVM = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        vm.SaveItemLabelAndColor(itemVM, "tag", CardColor.Purple);

        Assert.Equal("tag", model.Label);
        Assert.Equal(CardColor.Purple, model.CardColor);
    }

    [Fact]
    public void IsEmpty_WhenItemsLoaded_IsFalse()
    {
        var items = new List<ClipboardItem>
        {
            new() { Content = "item1", Type = ClipboardContentType.Text }
        };
        var (vm, _) = CreateVm(items);
        vm.ToggleColorFilter(CardColor.Red);
        Assert.False(vm.IsEmpty);
    }

    [Fact]
    public void IsEmpty_WhenNoItems_IsTrue()
    {
        var (vm, _) = CreateVm();
        vm.ToggleColorFilter(CardColor.Red);
        Assert.True(vm.IsEmpty);
    }

    [Fact]
    public void RefreshFileAvailability_DoesNotThrow()
    {
        var (vm, _) = CreateVm();
        var ex = Record.Exception(() => vm.RefreshFileAvailability());
        Assert.Null(ex);
    }

    [Fact]
    public void OpenRepoCommand_IsNotNull()
    {
        var (vm, _) = CreateVm();
        Assert.NotNull(vm.OpenRepoCommand);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stubs for new test classes
// ─────────────────────────────────────────────────────────────────────────────

internal sealed class TabFilterCapturingStub : IClipboardService
{
    private readonly List<ClipboardItem> _items;
    public bool? LastIsPinnedFilter { get; private set; }

    public TabFilterCapturingStub(List<ClipboardItem> items) => _items = items;

#pragma warning disable CS0067
    public event Action<ClipboardItem>? OnItemAdded;
    public event Action<ClipboardItem>? OnThumbnailReady;
    public event Action<ClipboardItem>? OnItemReactivated;
#pragma warning restore CS0067
    public int PasteIgnoreWindowMs { get; set; } = 450;

    public IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query,
        IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned)
    {
        LastIsPinnedFilter = isPinned;
        return _items.Skip(skip).Take(limit);
    }

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

internal sealed class ComprehensiveItemsStub : IClipboardService
{
    private readonly List<ClipboardItem> _items;

#pragma warning disable CS0067
    public event Action<ClipboardItem>? OnItemAdded;
    public event Action<ClipboardItem>? OnThumbnailReady;
    public event Action<ClipboardItem>? OnItemReactivated;
#pragma warning restore CS0067
    public int PasteIgnoreWindowMs { get; set; } = 450;

    public ComprehensiveItemsStub(List<ClipboardItem> items) => _items = items;

    public IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query,
        IReadOnlyCollection<ClipboardContentType>? types, IReadOnlyCollection<CardColor>? colors, bool? isPinned)
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
