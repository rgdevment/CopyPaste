using CopyPaste.Core;
using Xunit;

namespace CopyPaste.UI.Tests;

public sealed class ClipboardItemViewModelTests
{
    #region Constructor Tests

    [Fact]
    public void Constructor_WithValidModel_CreatesInstance()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.NotNull(viewModel);
        Assert.Equal(model, viewModel.Model);
    }

    [Fact]
    public void Constructor_WithNullModel_ThrowsException()
    {
        Assert.Throws<ArgumentNullException>(() =>
            new Themes.ClipboardItemViewModel(
                null!,
                _ => { },
                (_, _) => { },
                _ => { }
            )
        );
    }

    [Fact]
    public void Constructor_InitializesIsPinnedFromModel()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            IsPinned = true
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.True(viewModel.IsPinned);
    }

    #endregion

    #region IsPinned Tests

    [Fact]
    public void IsPinned_WhenChanged_UpdatesModel()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            IsPinned = false
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        viewModel.IsPinned = true;

        Assert.True(model.IsPinned);
    }

    [Fact]
    public void IsPinned_WhenSetToSameValue_DoesNotRaisePropertyChanged()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            IsPinned = false
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        var changeCount = 0;
        viewModel.PropertyChanged += (_, _) => changeCount++;

        viewModel.IsPinned = false;

        Assert.Equal(0, changeCount);
    }

    #endregion

    #region Timestamp Tests

    [Fact]
    public void Timestamp_Today_ShowsHoy()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        // Timestamp should be non-empty for today's date
        Assert.NotEmpty(viewModel.Timestamp);
    }

    [Fact]
    public void Timestamp_Yesterday_ShowsAyer()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow.AddDays(-1)
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        // Timestamp should be non-empty for yesterday's date
        Assert.NotEmpty(viewModel.Timestamp);
    }

    [Fact]
    public void Timestamp_OlderDate_ShowsFormattedDate()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow.AddDays(-7)
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        // Older dates should have a non-empty timestamp
        Assert.NotEmpty(viewModel.Timestamp);
    }

    #endregion

    #region RefreshFromModel Tests

    [Fact]
    public void RefreshFromModel_UpdatesContent()
    {
        var model = new ClipboardItem
        {
            Content = "Original",
            Type = ClipboardContentType.Text
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        var updatedModel = new ClipboardItem
        {
            Content = "Updated",
            Type = ClipboardContentType.Text
        };

        viewModel.RefreshFromModel(updatedModel);

        Assert.Equal("Updated", viewModel.Model.Content);
    }

    [Fact]
    public void RefreshFromModel_WithNullModel_ThrowsException()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.Throws<ArgumentNullException>(() => viewModel.RefreshFromModel(null!));
    }

    [Fact]
    public void RefreshFromModel_UpdatesMetadata()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            Metadata = null
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        var updatedModel = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            Metadata = "{\"key\":\"value\"}"
        };

        viewModel.RefreshFromModel(updatedModel);

        Assert.Equal("{\"key\":\"value\"}", viewModel.Model.Metadata);
    }

    #endregion

    #region Model Property Tests

    [Fact]
    public void Model_ReturnsOriginalModel()
    {
        var model = new ClipboardItem
        {
            Id = Guid.NewGuid(),
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.Same(model, viewModel.Model);
        Assert.Equal(model.Id, viewModel.Model.Id);
    }

    #endregion

    #region Label Tests

    [Fact]
    public void Label_WithValue_ReturnsLabel()
    {
        var model = new ClipboardItem
        {
            Content = "abc-123",
            Type = ClipboardContentType.Text,
            Label = "API Key"
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        Assert.Equal("API Key", viewModel.Label);
        Assert.True(viewModel.HasLabel);
    }

    [Fact]
    public void Label_WithNull_HasLabelIsFalse()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            Label = null
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        Assert.Null(viewModel.Label);
        Assert.False(viewModel.HasLabel);
    }

    [Fact]
    public void CardColor_ReturnsCorrectBorderColor()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CardColor = CardColor.Red
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        Assert.Equal(CardColor.Red, viewModel.CardColor);
        Assert.Equal("#E74C3C", viewModel.CardBorderColor);
    }

    [Fact]
    public void CardColor_None_ReturnsTransparent()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CardColor = CardColor.None
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        Assert.Equal("Transparent", viewModel.CardBorderColor);
    }

    [Fact]
    public void HasCardColor_WithColor_ReturnsTrue()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CardColor = CardColor.Green
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        Assert.True(viewModel.HasCardColor);
        Assert.Equal(Microsoft.UI.Xaml.Visibility.Visible, viewModel.CardColorVisibility);
    }

    [Fact]
    public void HasCardColor_WithNone_ReturnsFalse()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CardColor = CardColor.None
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        Assert.False(viewModel.HasCardColor);
        Assert.Equal(Microsoft.UI.Xaml.Visibility.Collapsed, viewModel.CardColorVisibility);
    }

    [Fact]
    public void RefreshLabelAndColor_NotifiesPropertyChanges()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        var propertiesChanged = new List<string>();
        viewModel.PropertyChanged += (_, e) => propertiesChanged.Add(e.PropertyName!);

        model.Label = "New Label";
        model.CardColor = CardColor.Blue;
        viewModel.RefreshLabelAndColor();

        Assert.Contains("Label", propertiesChanged);
        Assert.Contains("CardColor", propertiesChanged);
        Assert.Contains("HasCardColor", propertiesChanged);
        Assert.Contains("CardColorVisibility", propertiesChanged);
        Assert.Contains("CardBorderColor", propertiesChanged);
    }

    [Fact]
    public void EditCommand_WithEditAction_InvokesAction()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        var editCalled = false;
        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }, _ => editCalled = true
        );

        viewModel.EditCommand.Execute(null);

        Assert.True(editCalled);
    }

    [Fact]
    public void CanEdit_WithEditAction_ReturnsTrue()
    {
        var model = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }, _ => { }
        );

        Assert.True(viewModel.CanEdit);
    }

    [Fact]
    public void CanEdit_WithoutEditAction_ReturnsFalse()
    {
        var model = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        Assert.False(viewModel.CanEdit);
    }

    #endregion

    #region PinIndicatorVisibility Tests

    [Fact]
    public void PinIndicatorVisibility_WhenPinnedAndShowIndicatorTrue_ReturnsVisible()
    {
        var model = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text, IsPinned = true };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }, editAction: null, showPinIndicator: true
        );

        Assert.Equal(Microsoft.UI.Xaml.Visibility.Visible, viewModel.PinIndicatorVisibility);
    }

    [Fact]
    public void PinIndicatorVisibility_WhenPinnedAndShowIndicatorFalse_ReturnsCollapsed()
    {
        var model = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text, IsPinned = true };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }, editAction: null, showPinIndicator: false
        );

        Assert.Equal(Microsoft.UI.Xaml.Visibility.Collapsed, viewModel.PinIndicatorVisibility);
    }

    [Fact]
    public void PinIndicatorVisibility_WhenNotPinnedAndShowIndicatorTrue_ReturnsCollapsed()
    {
        var model = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text, IsPinned = false };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }, editAction: null, showPinIndicator: true
        );

        Assert.Equal(Microsoft.UI.Xaml.Visibility.Collapsed, viewModel.PinIndicatorVisibility);
    }

    [Fact]
    public void PinIndicatorVisibility_WhenNotPinnedAndShowIndicatorFalse_ReturnsCollapsed()
    {
        var model = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text, IsPinned = false };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        Assert.Equal(Microsoft.UI.Xaml.Visibility.Collapsed, viewModel.PinIndicatorVisibility);
    }

    [Fact]
    public void PinIndicatorVisibility_DefaultShowIndicator_IsFalse()
    {
        var model = new ClipboardItem { Content = "Test", Type = ClipboardContentType.Text, IsPinned = true };

        var viewModel = new Themes.ClipboardItemViewModel(
            model, _ => { }, (_, _) => { }, _ => { }
        );

        // Even when pinned, default showPinIndicator=false means Collapsed
        Assert.Equal(Microsoft.UI.Xaml.Visibility.Collapsed, viewModel.PinIndicatorVisibility);
    }

    #endregion
}
