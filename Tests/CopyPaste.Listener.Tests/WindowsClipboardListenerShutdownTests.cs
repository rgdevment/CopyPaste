using CopyPaste.Core;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using Xunit;

namespace CopyPaste.Listener.Tests;

public sealed class WindowsClipboardListenerShutdownTests
{
    [Fact]
    public void Shutdown_BeforeRun_DoesNotThrow()
    {
        using var listener = new WindowsClipboardListener(new StubClipboardService());

        var exception = Record.Exception(() => listener.Shutdown());

        Assert.Null(exception);
    }

    [Fact]
    public void Shutdown_BeforeRun_DoesNotRequireWindow()
    {
        // _hwnd is IntPtr.Zero before Run() — Shutdown should skip PostMessage
        using var listener = new WindowsClipboardListener(new StubClipboardService());

        var exception = Record.Exception(() => listener.Shutdown());

        Assert.Null(exception);
    }

    [Fact]
    public void Shutdown_WithDifferentServiceInstances_DoesNotThrow()
    {
        using var listener1 = new WindowsClipboardListener(new StubClipboardService());
        using var listener2 = new WindowsClipboardListener(new StubClipboardService());

        var exception = Record.Exception(() =>
        {
            listener1.Shutdown();
            listener2.Shutdown();
        });

        Assert.Null(exception);
    }

    [Fact]
    public void Dispose_AfterShutdown_DoesNotThrow()
    {
#pragma warning disable CA2000 // Testing Dispose behavior directly
        var listener = new WindowsClipboardListener(new StubClipboardService());
#pragma warning restore CA2000
        listener.Shutdown();

        var exception = Record.Exception(() => listener.Dispose());

        Assert.Null(exception);
    }

    [Fact]
    public void Dispose_WithoutRunOrShutdown_DoesNotThrow()
    {
#pragma warning disable CA2000 // Testing Dispose behavior directly
        var listener = new WindowsClipboardListener(new StubClipboardService());
#pragma warning restore CA2000

        var exception = Record.Exception(() => listener.Dispose());

        Assert.Null(exception);
    }

    [Fact]
    public void Dispose_CalledTwiceAfterShutdown_DoesNotThrow()
    {
#pragma warning disable CA2000 // Testing Dispose behavior directly
        var listener = new WindowsClipboardListener(new StubClipboardService());
#pragma warning restore CA2000
        listener.Shutdown();
        listener.Dispose();

        var exception = Record.Exception(() => listener.Dispose());

        Assert.Null(exception);
    }

    private sealed class StubClipboardService : IClipboardService
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
}
