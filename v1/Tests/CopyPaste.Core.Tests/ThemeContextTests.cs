using System;
using CopyPaste.Core.Themes;
using Xunit;

namespace CopyPaste.Core.Tests;

public class ThemeContextTests
{
    private static readonly StubClipboardService _stubService = new();
    private static readonly MyMConfig _config = new();

    [Fact]
    public void Constructor_SetsServiceProperty()
    {
        var context = new ThemeContext(_stubService, _config, () => { }, () => { }, () => { });

        Assert.Same(_stubService, context.Service);
    }

    [Fact]
    public void Constructor_SetsConfigProperty()
    {
        var context = new ThemeContext(_stubService, _config, () => { }, () => { }, () => { });

        Assert.Same(_config, context.Config);
    }

    [Fact]
    public void Constructor_SetsOpenSettingsAction()
    {
        var called = false;
        var context = new ThemeContext(_stubService, _config, () => called = true, () => { }, () => { });

        context.OpenSettings();

        Assert.True(called);
    }

    [Fact]
    public void Constructor_SetsOpenHelpAction()
    {
        var called = false;
        var context = new ThemeContext(_stubService, _config, () => { }, () => called = true, () => { });

        context.OpenHelp();

        Assert.True(called);
    }

    [Fact]
    public void Constructor_SetsRequestExitAction()
    {
        var called = false;
        var context = new ThemeContext(_stubService, _config, () => { }, () => { }, () => called = true);

        context.RequestExit();

        Assert.True(called);
    }

    [Fact]
    public void Constructor_WithCustomConfig_ReflectsValues()
    {
        var customConfig = new MyMConfig { PageSize = 99, RetentionDays = 7 };
        var context = new ThemeContext(_stubService, customConfig, () => { }, () => { }, () => { });

        Assert.Equal(99, context.Config.PageSize);
        Assert.Equal(7, context.Config.RetentionDays);
    }

    private sealed class StubClipboardService : IClipboardService
    {
        public event Action<ClipboardItem>? OnItemAdded;
        public event Action<ClipboardItem>? OnThumbnailReady;
        public event Action<ClipboardItem>? OnItemReactivated;
        public int PasteIgnoreWindowMs { get; set; }

        public void AddText(string? text, ClipboardContentType type, string? source, byte[]? rtfBytes = null, byte[]? htmlBytes = null) { }
        public void AddImage(byte[]? dibData, string? source) { }
        public void AddFiles(System.Collections.ObjectModel.Collection<string>? files, ClipboardContentType type, string? source) { }
        public System.Collections.Generic.IEnumerable<ClipboardItem> GetHistory(int limit = 50, int skip = 0, string? query = null, bool? isPinned = null) => [];
        public System.Collections.Generic.IEnumerable<ClipboardItem> GetHistoryAdvanced(int limit, int skip, string? query, System.Collections.Generic.IReadOnlyCollection<ClipboardContentType>? types, System.Collections.Generic.IReadOnlyCollection<CardColor>? colors, bool? isPinned) => [];
        public void RemoveItem(Guid id) { }
        public void UpdatePin(Guid id, bool isPinned) { }
        public void UpdateLabelAndColor(Guid id, string? label, CardColor color) { }
        public ClipboardItem? MarkItemUsed(Guid id) => null;
        public void NotifyPasteInitiated(Guid itemId) { }

        // Suppress unused event warnings
        internal void SuppressWarnings()
        {
            OnItemAdded?.Invoke(null!);
            OnThumbnailReady?.Invoke(null!);
            OnItemReactivated?.Invoke(null!);
        }
    }
}
