using CopyPaste.Core;
using CopyPaste.UI.Themes;
using Microsoft.UI.Xaml;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class ClipboardItemViewModelAdditionalTests
{
    private static ClipboardItemViewModel CreateVm(ClipboardItem model, bool showPinIndicator = false, int maxLines = 12, int minLines = 3)
    {
        return new ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { },
            _ => { },
            showPinIndicator,
            maxLines,
            minLines
        );
    }

    #region TypeIcon Tests

    [Theory]
    [InlineData(ClipboardContentType.Text, "\uE8C4")]
    [InlineData(ClipboardContentType.Image, "\uE91B")]
    [InlineData(ClipboardContentType.Link, "\uE71B")]
    [InlineData(ClipboardContentType.File, "\uE8B7")]
    [InlineData(ClipboardContentType.Folder, "\uE8D5")]
    [InlineData(ClipboardContentType.Audio, "\uE8D6")]
    [InlineData(ClipboardContentType.Video, "\uE714")]
    [InlineData(ClipboardContentType.Unknown, "\uE7ba")]
    public void TypeIcon_ReturnsCorrectGlyph(ClipboardContentType type, string expectedGlyph)
    {
        var model = new ClipboardItem { Content = "test", Type = type };
        var vm = CreateVm(model);

        Assert.Equal(expectedGlyph, vm.TypeIcon);
    }

    #endregion

    #region CardBorderColor Tests

    [Theory]
    [InlineData(CardColor.Red, "#E74C3C")]
    [InlineData(CardColor.Green, "#2ECC71")]
    [InlineData(CardColor.Purple, "#9B59B6")]
    [InlineData(CardColor.Yellow, "#F1C40F")]
    [InlineData(CardColor.Blue, "#3498DB")]
    [InlineData(CardColor.Orange, "#E67E22")]
    [InlineData(CardColor.None, "Transparent")]
    public void CardBorderColor_ReturnsCorrectHex(CardColor color, string expected)
    {
        var model = new ClipboardItem { Content = "test", CardColor = color };
        var vm = CreateVm(model);

        Assert.Equal(expected, vm.CardBorderColor);
    }

    #endregion

    #region HasCardColor and CardColor Tests

    [Fact]
    public void HasCardColor_WhenNone_ReturnsFalse()
    {
        var model = new ClipboardItem { Content = "test", CardColor = CardColor.None };
        var vm = CreateVm(model);

        Assert.False(vm.HasCardColor);
    }

    [Fact]
    public void HasCardColor_WhenSet_ReturnsTrue()
    {
        var model = new ClipboardItem { Content = "test", CardColor = CardColor.Red };
        var vm = CreateVm(model);

        Assert.True(vm.HasCardColor);
    }

    [Fact]
    public void CardColorVisibility_WhenNone_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "test", CardColor = CardColor.None };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.CardColorVisibility);
    }

    [Fact]
    public void CardColorVisibility_WhenSet_IsVisible()
    {
        var model = new ClipboardItem { Content = "test", CardColor = CardColor.Blue };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.CardColorVisibility);
    }

    #endregion

    #region Label Tests

    [Fact]
    public void HasLabel_WhenNull_ReturnsFalse()
    {
        var model = new ClipboardItem { Content = "test", Label = null };
        var vm = CreateVm(model);

        Assert.False(vm.HasLabel);
    }

    [Fact]
    public void HasLabel_WhenEmpty_ReturnsFalse()
    {
        var model = new ClipboardItem { Content = "test", Label = "" };
        var vm = CreateVm(model);

        Assert.False(vm.HasLabel);
    }

    [Fact]
    public void HasLabel_WhenSet_ReturnsTrue()
    {
        var model = new ClipboardItem { Content = "test", Label = "Important" };
        var vm = CreateVm(model);

        Assert.True(vm.HasLabel);
    }

    [Fact]
    public void LabelVisibility_WhenHasLabel_IsVisible()
    {
        var model = new ClipboardItem { Content = "test", Label = "Work" };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.LabelVisibility);
    }

    [Fact]
    public void LabelVisibility_WhenNoLabel_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.LabelVisibility);
    }

    [Fact]
    public void DefaultHeaderVisibility_WhenHasLabel_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "test", Label = "Labeled" };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.DefaultHeaderVisibility);
    }

    [Fact]
    public void DefaultHeaderVisibility_WhenNoLabel_IsVisible()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.DefaultHeaderVisibility);
    }

    #endregion

    #region PasteCount Tests

    [Fact]
    public void PasteCountDisplay_WhenZero_ShowsTimesZero()
    {
        var model = new ClipboardItem { Content = "test", PasteCount = 0 };
        var vm = CreateVm(model);

        Assert.Equal("×0", vm.PasteCountDisplay);
    }

    [Fact]
    public void PasteCountDisplay_WhenUnder1000_ShowsExactCount()
    {
        var model = new ClipboardItem { Content = "test", PasteCount = 42 };
        var vm = CreateVm(model);

        Assert.Equal("×42", vm.PasteCountDisplay);
    }

    [Fact]
    public void PasteCountDisplay_When999_ShowsExactCount()
    {
        var model = new ClipboardItem { Content = "test", PasteCount = 999 };
        var vm = CreateVm(model);

        Assert.Equal("×999", vm.PasteCountDisplay);
    }

    [Fact]
    public void PasteCountDisplay_When1000_ShowsK()
    {
        var model = new ClipboardItem { Content = "test", PasteCount = 1000 };
        var vm = CreateVm(model);

        Assert.Equal("×1K+", vm.PasteCountDisplay);
    }

    [Fact]
    public void PasteCountDisplay_WhenOver1000_ShowsK()
    {
        var model = new ClipboardItem { Content = "test", PasteCount = 5000 };
        var vm = CreateVm(model);

        Assert.Equal("×1K+", vm.PasteCountDisplay);
    }

    [Fact]
    public void PasteCountVisibility_WhenZero_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "test", PasteCount = 0 };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.PasteCountVisibility);
    }

    [Fact]
    public void PasteCountVisibility_WhenPositive_IsVisible()
    {
        var model = new ClipboardItem { Content = "test", PasteCount = 1 };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.PasteCountVisibility);
    }

    #endregion

    #region AppSource Tests

    [Fact]
    public void AppSource_WhenNull_ReturnsNull()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        Assert.Null(vm.AppSource);
    }

    [Fact]
    public void AppSource_WhenSet_ReturnsValue()
    {
        var model = new ClipboardItem { Content = "test", AppSource = "Chrome.exe" };
        var vm = CreateVm(model);

        Assert.Equal("Chrome.exe", vm.AppSource);
    }

    [Fact]
    public void AppSourceVisibility_WhenNull_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.AppSourceVisibility);
    }

    [Fact]
    public void AppSourceVisibility_WhenSet_IsVisible()
    {
        var model = new ClipboardItem { Content = "test", AppSource = "App" };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.AppSourceVisibility);
    }

    #endregion

    #region FileSize from Metadata

    [Fact]
    public void FileSize_WithFileSizeMetadata_ReturnsFormattedSize()
    {
        var model = new ClipboardItem
        {
            Content = "test.txt",
            Type = ClipboardContentType.File,
            Metadata = """{"file_size": 1024}"""
        };
        var vm = CreateVm(model);

        Assert.Equal("1 KB", vm.FileSize);
    }

    [Fact]
    public void FileSize_WithSizeMetadata_ReturnsFormattedSize()
    {
        var model = new ClipboardItem
        {
            Content = "image.png",
            Type = ClipboardContentType.Image,
            Metadata = """{"size": 2048}"""
        };
        var vm = CreateVm(model);

        Assert.Equal("2 KB", vm.FileSize);
    }

    [Fact]
    public void FileSize_WithNoMetadata_ReturnsNull()
    {
        var model = new ClipboardItem { Content = "test", Type = ClipboardContentType.Text };
        var vm = CreateVm(model);

        Assert.Null(vm.FileSize);
    }

    [Fact]
    public void FileSize_WithInvalidJson_ReturnsNull()
    {
        var model = new ClipboardItem
        {
            Content = "test",
            Metadata = "invalid json"
        };
        var vm = CreateVm(model);

        Assert.Null(vm.FileSize);
    }

    [Fact]
    public void FileSize_LargeBytes_FormatsAsMB()
    {
        var model = new ClipboardItem
        {
            Content = "video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = """{"file_size": 10485760}"""
        };
        var vm = CreateVm(model);

        Assert.Equal("10 MB", vm.FileSize);
    }

    [Fact]
    public void FileSize_SmallBytes_FormatsAsB()
    {
        var model = new ClipboardItem
        {
            Content = "small.txt",
            Type = ClipboardContentType.File,
            Metadata = """{"file_size": 100}"""
        };
        var vm = CreateVm(model);

        Assert.Equal("100 B", vm.FileSize);
    }

    [Fact]
    public void FileSizeVisibility_WhenHasSize_IsVisible()
    {
        var model = new ClipboardItem
        {
            Content = "test.txt",
            Type = ClipboardContentType.File,
            Metadata = """{"file_size": 1024}"""
        };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.FileSizeVisibility);
    }

    [Fact]
    public void FileSizeVisibility_WhenNoSize_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.FileSizeVisibility);
    }

    #endregion

    #region ImageDimensions Tests

    [Fact]
    public void ImageDimensions_WithWidthAndHeight_ReturnsFormatted()
    {
        var model = new ClipboardItem
        {
            Content = "img.png",
            Type = ClipboardContentType.Image,
            Metadata = """{"width": 1920, "height": 1080}"""
        };
        var vm = CreateVm(model);

        Assert.Equal("1920×1080", vm.ImageDimensions);
    }

    [Fact]
    public void ImageDimensions_NonImageType_ReturnsNull()
    {
        var model = new ClipboardItem
        {
            Content = "test",
            Type = ClipboardContentType.Text,
            Metadata = """{"width": 100, "height": 100}"""
        };
        var vm = CreateVm(model);

        Assert.Null(vm.ImageDimensions);
    }

    [Fact]
    public void ImageDimensions_NoMetadata_ReturnsNull()
    {
        var model = new ClipboardItem
        {
            Content = "img.png",
            Type = ClipboardContentType.Image
        };
        var vm = CreateVm(model);

        Assert.Null(vm.ImageDimensions);
    }

    [Fact]
    public void ImageDimensionsVisibility_WhenPresent_IsVisible()
    {
        var model = new ClipboardItem
        {
            Content = "img.png",
            Type = ClipboardContentType.Image,
            Metadata = """{"width": 800, "height": 600}"""
        };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.ImageDimensionsVisibility);
    }

    #endregion

    #region MediaDuration Tests

    [Fact]
    public void MediaDuration_WithDurationMetadata_ReturnsFormatted()
    {
        var model = new ClipboardItem
        {
            Content = "video.mp4",
            Type = ClipboardContentType.Video,
            Metadata = """{"duration": 125}"""
        };
        var vm = CreateVm(model);

        Assert.Equal("2:05", vm.MediaDuration);
    }

    [Fact]
    public void MediaDuration_OverOneHour_ReturnsHourFormat()
    {
        var model = new ClipboardItem
        {
            Content = "movie.mp4",
            Type = ClipboardContentType.Video,
            Metadata = """{"duration": 3665}"""
        };
        var vm = CreateVm(model);

        Assert.Equal("1:01:05", vm.MediaDuration);
    }

    [Fact]
    public void MediaDuration_NoMetadata_ReturnsNull()
    {
        var model = new ClipboardItem
        {
            Content = "video.mp4",
            Type = ClipboardContentType.Video
        };
        var vm = CreateVm(model);

        Assert.Null(vm.MediaDuration);
    }

    [Fact]
    public void DurationVisibility_WhenHasDuration_IsVisible()
    {
        var model = new ClipboardItem
        {
            Content = "audio.mp3",
            Type = ClipboardContentType.Audio,
            Metadata = """{"duration": 60}"""
        };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.DurationVisibility);
    }

    [Fact]
    public void DurationVisibility_WhenNoDuration_IsCollapsed()
    {
        var model = new ClipboardItem
        {
            Content = "audio.mp3",
            Type = ClipboardContentType.Audio
        };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.DurationVisibility);
    }

    #endregion

    #region Visibility Tests

    [Fact]
    public void IsTextVisible_ForTextType_IsVisible()
    {
        var model = new ClipboardItem { Content = "hello", Type = ClipboardContentType.Text };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.IsTextVisible);
    }

    [Fact]
    public void IsTextVisible_ForImageType_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "img.png", Type = ClipboardContentType.Image };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.IsTextVisible);
    }

    [Fact]
    public void IsTextVisible_ForVideoType_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "vid.mp4", Type = ClipboardContentType.Video };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.IsTextVisible);
    }

    [Fact]
    public void IsTextVisible_ForAudioType_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "audio.mp3", Type = ClipboardContentType.Audio };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.IsTextVisible);
    }

    [Fact]
    public void IsTextVisible_ForLinkType_IsVisible()
    {
        var model = new ClipboardItem { Content = "https://example.com", Type = ClipboardContentType.Link };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.IsTextVisible);
    }

    [Fact]
    public void IsTextVisible_ForFileType_IsVisible()
    {
        var model = new ClipboardItem { Content = "doc.pdf", Type = ClipboardContentType.File };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.IsTextVisible);
    }

    [Fact]
    public void MediaThumbnailVisibility_ForVideo_IsVisible()
    {
        var model = new ClipboardItem { Content = "vid.mp4", Type = ClipboardContentType.Video };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.MediaThumbnailVisibility);
    }

    [Fact]
    public void MediaThumbnailVisibility_ForAudio_IsVisible()
    {
        var model = new ClipboardItem { Content = "song.mp3", Type = ClipboardContentType.Audio };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.MediaThumbnailVisibility);
    }

    [Fact]
    public void MediaThumbnailVisibility_ForText_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "text", Type = ClipboardContentType.Text };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.MediaThumbnailVisibility);
    }

    #endregion

    #region IsExpanded and ContentMaxLines Tests

    [Fact]
    public void IsExpanded_InitiallyFalse()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        Assert.False(vm.IsExpanded);
    }

    [Fact]
    public void ContentMaxLines_WhenCollapsed_ReturnsMinLines()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model, minLines: 3, maxLines: 12);

        Assert.Equal(3, vm.ContentMaxLines);
    }

    [Fact]
    public void ContentMaxLines_WhenExpanded_ReturnsMaxLines()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model, minLines: 3, maxLines: 12);

        vm.IsExpanded = true;

        Assert.Equal(12, vm.ContentMaxLines);
    }

    [Fact]
    public void ToggleExpanded_TogglesState()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        vm.ToggleExpanded();
        Assert.True(vm.IsExpanded);

        vm.ToggleExpanded();
        Assert.False(vm.IsExpanded);
    }

    [Fact]
    public void Collapse_SetsExpandedToFalse()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        vm.IsExpanded = true;
        vm.Collapse();

        Assert.False(vm.IsExpanded);
    }

    #endregion

    #region Pin Indicator Tests

    [Fact]
    public void PinIndicatorVisibility_WhenPinnedAndShowIndicator_IsVisible()
    {
        var model = new ClipboardItem { Content = "test", IsPinned = true };
        var vm = CreateVm(model, showPinIndicator: true);

        Assert.Equal(Visibility.Visible, vm.PinIndicatorVisibility);
    }

    [Fact]
    public void PinIndicatorVisibility_WhenPinnedButNoShowIndicator_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "test", IsPinned = true };
        var vm = CreateVm(model, showPinIndicator: false);

        Assert.Equal(Visibility.Collapsed, vm.PinIndicatorVisibility);
    }

    [Fact]
    public void PinIndicatorVisibility_WhenNotPinned_IsCollapsed()
    {
        var model = new ClipboardItem { Content = "test", IsPinned = false };
        var vm = CreateVm(model, showPinIndicator: true);

        Assert.Equal(Visibility.Collapsed, vm.PinIndicatorVisibility);
    }

    #endregion

    #region CanEdit Tests

    [Fact]
    public void CanEdit_WithEditAction_ReturnsTrue()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        Assert.True(vm.CanEdit);
    }

    [Fact]
    public void CanEdit_WithoutEditAction_ReturnsFalse()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = new ClipboardItemViewModel(model, _ => { }, (_, _) => { }, _ => { });

        Assert.False(vm.CanEdit);
    }

    #endregion

    #region Content Property Tests

    [Fact]
    public void Content_WhenModelContentNull_ReturnsEmpty()
    {
        var model = new ClipboardItem { Content = null! };
        var vm = CreateVm(model);

        Assert.Equal(string.Empty, vm.Content);
    }

    [Fact]
    public void Content_ReturnsModelContent()
    {
        var model = new ClipboardItem { Content = "Hello World" };
        var vm = CreateVm(model);

        Assert.Equal("Hello World", vm.Content);
    }

    #endregion

    #region IsFileType Tests

    [Theory]
    [InlineData(ClipboardContentType.File, true)]
    [InlineData(ClipboardContentType.Folder, true)]
    [InlineData(ClipboardContentType.Audio, true)]
    [InlineData(ClipboardContentType.Video, true)]
    [InlineData(ClipboardContentType.Text, false)]
    [InlineData(ClipboardContentType.Image, false)]
    [InlineData(ClipboardContentType.Link, false)]
    public void IsFileType_MatchesModelIsFileBasedType(ClipboardContentType type, bool expected)
    {
        var model = new ClipboardItem { Content = "test", Type = type };
        var vm = CreateVm(model);

        Assert.Equal(expected, vm.IsFileType);
    }

    #endregion

    #region RefreshLabelAndColor Tests

    [Fact]
    public void RefreshLabelAndColor_UpdatesProperties()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        var changedProperties = new System.Collections.Generic.List<string>();
        vm.PropertyChanged += (_, e) => changedProperties.Add(e.PropertyName!);

        vm.RefreshLabelAndColor();

        Assert.Contains("Label", changedProperties);
        Assert.Contains("HasLabel", changedProperties);
        Assert.Contains("LabelVisibility", changedProperties);
        Assert.Contains("CardColor", changedProperties);
        Assert.Contains("HasCardColor", changedProperties);
        Assert.Contains("CardBorderColor", changedProperties);
    }

    #endregion

    #region RefreshPasteCount Tests

    [Fact]
    public void RefreshPasteCount_NotifiesPropertyChanged()
    {
        var model = new ClipboardItem { Content = "test", PasteCount = 5 };
        var vm = CreateVm(model);

        var changedProperties = new System.Collections.Generic.List<string>();
        vm.PropertyChanged += (_, e) => changedProperties.Add(e.PropertyName!);

        vm.RefreshPasteCount();

        Assert.Contains("PasteCountDisplay", changedProperties);
        Assert.Contains("PasteCountVisibility", changedProperties);
    }

    #endregion

    #region RefreshFromModel Tests

    [Fact]
    public void RefreshFromModel_UpdatesContent()
    {
        var model = new ClipboardItem { Content = "original" };
        var vm = CreateVm(model);

        var updated = new ClipboardItem { Content = "updated", Metadata = null };
        vm.RefreshFromModel(updated);

        Assert.Equal("updated", vm.Content);
    }

    [Fact]
    public void RefreshFromModel_WithNull_Throws()
    {
        var model = new ClipboardItem { Content = "test" };
        var vm = CreateVm(model);

        Assert.Throws<System.ArgumentNullException>(() => vm.RefreshFromModel(null!));
    }

    [Fact]
    public void RefreshFromModel_FiresImagePathChangedEvent()
    {
        var model = new ClipboardItem { Content = "img.png", Type = ClipboardContentType.Image };
        var vm = CreateVm(model);

        var eventFired = false;
        vm.ImagePathChanged += (_, _) => eventFired = true;

        vm.RefreshFromModel(new ClipboardItem { Content = "new.png" });

        Assert.True(eventFired);
    }

    #endregion

    #region RefreshFileStatus Tests

    [Fact]
    public void RefreshFileStatus_NotifiesPropertyChanged()
    {
        var model = new ClipboardItem { Content = "test.txt", Type = ClipboardContentType.File };
        var vm = CreateVm(model);

        var changedProperties = new System.Collections.Generic.List<string>();
        vm.PropertyChanged += (_, e) => changedProperties.Add(e.PropertyName!);

        vm.RefreshFileStatus();

        Assert.Contains("IsFileAvailable", changedProperties);
        Assert.Contains("FileWarningVisibility", changedProperties);
    }

    #endregion

    #region PinIconGlyph and PinMenuText

    [Fact]
    public void PinIconGlyph_WhenPinned_ReturnsFilled()
    {
        var model = new ClipboardItem { Content = "test", IsPinned = true };
        var vm = CreateVm(model);

        Assert.Equal("\uE840", vm.PinIconGlyph);
    }

    [Fact]
    public void PinIconGlyph_WhenNotPinned_ReturnsOutline()
    {
        var model = new ClipboardItem { Content = "test", IsPinned = false };
        var vm = CreateVm(model);

        Assert.Equal("\uE718", vm.PinIconGlyph);
    }

    #endregion

    #region Command Tests

    [Fact]
    public void DeleteCommand_InvokesDeleteAction()
    {
        var deleteCalled = false;
        var model = new ClipboardItem { Content = "test" };
        var vm = new ClipboardItemViewModel(
            model,
            _ => deleteCalled = true,
            (_, _) => { },
            _ => { }
        );

        vm.DeleteCommand.Execute(null);

        Assert.True(deleteCalled);
    }

    [Fact]
    public void PasteCommand_InvokesPasteAction_WithFalsePlain()
    {
        bool? plainArg = null;
        var model = new ClipboardItem { Content = "test" };
        var vm = new ClipboardItemViewModel(
            model,
            _ => { },
            (_, plain) => plainArg = plain,
            _ => { }
        );

        vm.PasteCommand.Execute(null);

        Assert.False(plainArg);
    }

    [Fact]
    public void PastePlainCommand_InvokesPasteAction_WithTruePlain()
    {
        bool? plainArg = null;
        var model = new ClipboardItem { Content = "test" };
        var vm = new ClipboardItemViewModel(
            model,
            _ => { },
            (_, plain) => plainArg = plain,
            _ => { }
        );

        vm.PastePlainCommand.Execute(null);

        Assert.True(plainArg);
    }

    [Fact]
    public void TogglePinCommand_TogglesPinState()
    {
        var model = new ClipboardItem { Content = "test", IsPinned = false };
        var vm = new ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        vm.TogglePinCommand.Execute(null);

        Assert.True(vm.IsPinned);
    }

    [Fact]
    public void EditCommand_InvokesEditAction()
    {
        var editCalled = false;
        var model = new ClipboardItem { Content = "test" };
        var vm = new ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { },
            _ => editCalled = true
        );

        vm.EditCommand.Execute(null);

        Assert.True(editCalled);
    }

    #endregion

    #region MediaInfoVisibility Tests

    [Fact]
    public void MediaInfoVisibility_ForVideoWithDuration_IsVisible()
    {
        var model = new ClipboardItem
        {
            Content = "vid.mp4",
            Type = ClipboardContentType.Video,
            Metadata = """{"duration": 60}"""
        };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.MediaInfoVisibility);
    }

    [Fact]
    public void MediaInfoVisibility_ForTextType_IsCollapsed()
    {
        var model = new ClipboardItem
        {
            Content = "hello",
            Type = ClipboardContentType.Text
        };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Collapsed, vm.MediaInfoVisibility);
    }

    [Fact]
    public void MediaInfoVisibility_ForImageWithDimensions_IsVisible()
    {
        var model = new ClipboardItem
        {
            Content = "img.png",
            Type = ClipboardContentType.Image,
            Metadata = """{"width": 100, "height": 100}"""
        };
        var vm = CreateVm(model);

        Assert.Equal(Visibility.Visible, vm.MediaInfoVisibility);
    }

    #endregion
}
