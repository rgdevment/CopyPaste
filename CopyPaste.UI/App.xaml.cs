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
        this.UnhandledException += OnUnhandledException;

        InitializeComponent();

        // Initialize logger for error tracking
        AppLogger.Initialize();
        AppLogger.Info("Application starting...");

        StorageConfig.Initialize();

        // Register for Windows startup if configured
        RegisterForStartup();

        // Initialize core components
        _repository = new SqliteRepository(StorageConfig.DatabasePath);
        _service = new ClipboardService(_repository);
        _listener = new WindowsClipboardListener(_service);
        _cleanupService = new CleanupService(_repository, () => UIConfig.RetentionDays);

        // Configure paste timing
        _service.PasteIgnoreWindowMs = PasteConfig.DuplicateIgnoreWindowMs;

        // Run listener in background
        Task.Run(() => _listener.Run());

        AppLogger.Info("Application initialized");
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

        _window?.Close();
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

    private void OnUnhandledException(object sender, Microsoft.UI.Xaml.UnhandledExceptionEventArgs e)
    {
        AppLogger.Exception(e.Exception, "Unhandled exception");
        e.Handled = true;
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types", 
        Justification = "Startup registration is non-critical - any failure should not prevent app from running")]
    private static void RegisterForStartup()
    {
        if (!StartupConfig.RunOnStartup) return;

        try
        {
            const string keyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
            const string appName = "CopyPaste";

            using var key = Microsoft.Win32.Registry.CurrentUser.OpenSubKey(keyPath, true);
            if (key == null) return;

            if (key.GetValue(appName) == null)
            {
                var exePath = System.Environment.ProcessPath;
                if (!string.IsNullOrEmpty(exePath))
                {
                    key.SetValue(appName, $"\"{exePath}\"");
                    AppLogger.Info("Registered for Windows startup");
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to register for startup");
        }
    }
}




