// CopyPaste - High performance clipboard manager
// Copyright (C) 2026 Mario Hidalgo G. (rgdevment)
//
// This software and all original source files in this repository are part of CopyPaste.
//
// CopyPaste is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// CopyPaste is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with CopyPaste. If not, see <https://github.com/rgdevment/CopyPaste/blob/main/LICENSE>.

using CopyPaste.Core;
using CopyPaste.Listener;
using Microsoft.UI.Xaml;
using System;
using System.Threading.Tasks;

namespace CopyPaste.UI;

public sealed partial class App : Application, IDisposable
{
    private Window? _window;
    private readonly WindowsClipboardListener? _listener;
    private readonly ClipboardService? _service;
    private bool _isDisposed;
    public bool IsExiting { get; private set; }

    public App()
    {
        InitializeComponent();
        StorageConfig.Initialize();

        // Initialize core components once
        var repository = new LiteDbRepository(StorageConfig.DatabasePath);
        _service = new ClipboardService(repository);
        _listener = new WindowsClipboardListener(_service);

        // Run listener in background
        Task.Run(() => _listener.Run());
    }

    protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
    {
        // Configure thumbnail settings before initializing services
        // Uncomment one of the profiles below or customize your own
        
        // Low Memory Profile (4-8GB RAM):
        // ThumbnailConfig.Width = 200;
        // ThumbnailConfig.QualityPng = 75;
        // ThumbnailConfig.QualityJpeg = 75;
        // ThumbnailConfig.GarbageCollectionThreshold = 500_000;
        // ThumbnailConfig.UIDecodeHeight = 180;

        // Balanced Profile (default - recommended):
        // ThumbnailConfig.Width = 250;
        // ThumbnailConfig.QualityPng = 85;
        // ThumbnailConfig.QualityJpeg = 85;
        // ThumbnailConfig.GarbageCollectionThreshold = 1_000_000;
        // ThumbnailConfig.UIDecodeHeight = 220;

        // High Quality Profile (16GB+ RAM):
        // ThumbnailConfig.Width = 300;
        // ThumbnailConfig.QualityPng = 90;
        // ThumbnailConfig.QualityJpeg = 90;
        // ThumbnailConfig.GarbageCollectionThreshold = 2_000_000;
        // ThumbnailConfig.UIDecodeHeight = 260;

        // Configure UI behavior (optional - defaults are already set)
        // UIConfig.PageSize = 20;                      // Items loaded per page
        // UIConfig.MaxItemsBeforeCleanup = 100;        // Max items in memory
        // UIConfig.ScrollLoadThreshold = 100;          // Pixels from bottom to load more
        // UIConfig.WindowWidth = 400;                  // Sidebar width
        // UIConfig.Hotkey.UseWinKey = true;            // Use Win key (false = Ctrl key)
        // UIConfig.Hotkey.VirtualKey = 0x56;           // V key (Win/Ctrl + Alt + V)

        _window = new MainWindow(_service!);

        _window.Activate();
    }

    public void BeginExit()
    {
        IsExiting = true;
        try
        {
            _listener?.Dispose();
        }
        catch (ObjectDisposedException)
        {
            // Ya está eliminado, ignorar
        }
        catch (Exception)
        {
            // Registrar o manejar según sea necesario, pero no suprimir silenciosamente
            throw;
        }

        // Close the window to trigger normal shutdown
        _window?.Close();

        // Ensure application exit
        Application.Current.Exit();
    }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    private void Dispose(bool disposing)
    {
        if (_isDisposed) return;
        if (disposing) _listener?.Dispose();
        _isDisposed = true;
    }
}
