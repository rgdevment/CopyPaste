using CopyPaste.Core;
using CopyPaste.UI.Themes;
using System;
using System.Collections.Generic;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class ClipboardItemViewModelLifecycleTests
{
    private static ClipboardItemViewModel CreateViewModel(
        ClipboardItem? model = null,
        int cardMaxLines = 12,
        int cardMinLines = 3)
    {
        model ??= new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        return new ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { },
            cardMaxLines: cardMaxLines,
            cardMinLines: cardMinLines);
    }

    #region RefreshPasteCount

    [Fact]
    public void RefreshPasteCount_FiresPasteCountDisplayPropertyChanged()
    {
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text, PasteCount = 5 };
        var vm = CreateViewModel(model);
        var changedProps = new List<string?>();
        vm.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.RefreshPasteCount();

        Assert.Contains(nameof(ClipboardItemViewModel.PasteCountDisplay), changedProps);
    }

    [Fact]
    public void RefreshPasteCount_FiresPasteCountVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text, PasteCount = 3 };
        var vm = CreateViewModel(model);
        var changedProps = new List<string?>();
        vm.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.RefreshPasteCount();

        Assert.Contains(nameof(ClipboardItemViewModel.PasteCountVisibility), changedProps);
    }

    [Fact]
    public void RefreshPasteCount_AfterModelUpdate_ReflectsNewCount()
    {
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text, PasteCount = 0 };
        var vm = CreateViewModel(model);
        Assert.Equal("×0", vm.PasteCountDisplay);

        model.PasteCount = 42;
        vm.RefreshPasteCount();

        Assert.Equal("×42", vm.PasteCountDisplay);
    }

    [Fact]
    public void RefreshPasteCount_WhenCountExceeds999_DisplaysK()
    {
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text, PasteCount = 0 };
        var vm = CreateViewModel(model);

        model.PasteCount = 1000;
        vm.RefreshPasteCount();

        Assert.Equal("×1K+", vm.PasteCountDisplay);
    }

    #endregion

    #region RefreshFileStatus

    [Fact]
    public void RefreshFileStatus_FiresIsFileAvailablePropertyChanged()
    {
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var vm = CreateViewModel(model);
        var changedProps = new List<string?>();
        vm.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.RefreshFileStatus();

        Assert.Contains(nameof(ClipboardItemViewModel.IsFileAvailable), changedProps);
    }

    [Fact]
    public void RefreshFileStatus_FiresFileWarningVisibilityPropertyChanged()
    {
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var vm = CreateViewModel(model);
        var changedProps = new List<string?>();
        vm.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.RefreshFileStatus();

        Assert.Contains(nameof(ClipboardItemViewModel.FileWarningVisibility), changedProps);
    }

    [Fact]
    public void RefreshFileStatus_DoesNotThrow()
    {
        var vm = CreateViewModel();

        var exception = Record.Exception(() => vm.RefreshFileStatus());

        Assert.Null(exception);
    }

    #endregion

    #region ToggleExpanded

    [Fact]
    public void ToggleExpanded_SetsIsExpandedToTrue_WhenStartingFalse()
    {
        var vm = CreateViewModel();
        Assert.False(vm.IsExpanded);

        vm.ToggleExpanded();

        Assert.True(vm.IsExpanded);
    }

    [Fact]
    public void ToggleExpanded_SetsIsExpandedToFalse_WhenStartingTrue()
    {
        var vm = CreateViewModel();
        vm.ToggleExpanded(); // true

        vm.ToggleExpanded(); // back to false

        Assert.False(vm.IsExpanded);
    }

    [Fact]
    public void ToggleExpanded_FiresContentMaxLinesPropertyChanged()
    {
        var vm = CreateViewModel();
        var changedProps = new List<string?>();
        vm.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.ToggleExpanded();

        Assert.Contains(nameof(ClipboardItemViewModel.ContentMaxLines), changedProps);
    }

    [Fact]
    public void ToggleExpanded_FiresContentLineHeightPropertyChanged()
    {
        var vm = CreateViewModel();
        var changedProps = new List<string?>();
        vm.PropertyChanged += (_, e) => changedProps.Add(e.PropertyName);

        vm.ToggleExpanded();

        Assert.Contains(nameof(ClipboardItemViewModel.ContentLineHeight), changedProps);
    }

    #endregion

    #region Collapse

    [Fact]
    public void Collapse_SetsIsExpandedToFalse_AfterExpand()
    {
        var vm = CreateViewModel();
        vm.ToggleExpanded(); // expand first
        Assert.True(vm.IsExpanded);

        vm.Collapse();

        Assert.False(vm.IsExpanded);
    }

    [Fact]
    public void Collapse_WhenAlreadyCollapsed_DoesNotThrow()
    {
        var vm = CreateViewModel();
        Assert.False(vm.IsExpanded);

        var exception = Record.Exception(() => vm.Collapse());

        Assert.Null(exception);
        Assert.False(vm.IsExpanded);
    }

    [Fact]
    public void Collapse_CalledTwice_DoesNotThrow()
    {
        var vm = CreateViewModel();
        vm.ToggleExpanded();

        var exception = Record.Exception(() =>
        {
            vm.Collapse();
            vm.Collapse();
        });

        Assert.Null(exception);
        Assert.False(vm.IsExpanded);
    }

    #endregion

    #region ContentMaxLines

    [Fact]
    public void ContentMaxLines_WhenCollapsed_ReturnsCardMinLines()
    {
        var vm = CreateViewModel(cardMaxLines: 12, cardMinLines: 3);

        Assert.Equal(3, vm.ContentMaxLines);
        Assert.False(vm.IsExpanded);
    }

    [Fact]
    public void ContentMaxLines_WhenExpanded_ReturnsCardMaxLines()
    {
        var vm = CreateViewModel(cardMaxLines: 12, cardMinLines: 3);
        vm.ToggleExpanded();

        Assert.Equal(12, vm.ContentMaxLines);
    }

    [Fact]
    public void ContentMaxLines_TogglesCorrectlyBetweenExpandAndCollapse()
    {
        var vm = CreateViewModel(cardMaxLines: 10, cardMinLines: 4);

        Assert.Equal(4, vm.ContentMaxLines);

        vm.ToggleExpanded();
        Assert.Equal(10, vm.ContentMaxLines);

        vm.Collapse();
        Assert.Equal(4, vm.ContentMaxLines);
    }

    [Fact]
    public void ContentMaxLines_WithCustomLines_RespectsSettings()
    {
        var vm = CreateViewModel(cardMaxLines: 20, cardMinLines: 2);

        Assert.Equal(2, vm.ContentMaxLines);

        vm.ToggleExpanded();
        Assert.Equal(20, vm.ContentMaxLines);
    }

    #endregion
}
