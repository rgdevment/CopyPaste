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
    private readonly CleanupService? _cleanupService;
    private readonly SqliteRepository? _repository;
    private bool _isDisposed;
    public bool IsExiting { get; private set; }

    public App()
    {
        InitializeComponent();

        // Initialize logger first for error tracking
        AppLogger.Initialize();
        AppLogger.Info("Application starting...");

        StorageConfig.Initialize();

        // Initialize core components (Native AOT-compatible SQLite)
        _repository = new SqliteRepository(StorageConfig.DatabasePath);
        _service = new ClipboardService(_repository);
        _listener = new WindowsClipboardListener(_service);
        _cleanupService = new CleanupService(_repository, () => UIConfig.RetentionDays);

        // Configure paste timing from UIConfig
        _service.PasteIgnoreWindowMs = PasteConfig.DuplicateIgnoreWindowMs;

        // Run listener in background
        Task.Run(() => _listener.Run());

        AppLogger.Info("Application initialized successfully");
    }

    protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
    {
        _window = new MainWindow(_service!);

        _window.Activate();
        AppLogger.Info("Main window launched");
    }

    public void BeginExit()
    {
        IsExiting = true;
        AppLogger.Info("Application exiting...");
        try
        {
            _listener?.Dispose();
            _repository?.Dispose();
        }
        catch (ObjectDisposedException)
        {
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Error during exit");
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
        if (disposing)
        {
            _listener?.Dispose();
            _cleanupService?.Dispose();
            _repository?.Dispose();
        }
        _isDisposed = true;
    }
}
