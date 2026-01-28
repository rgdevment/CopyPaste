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
using System.IO;
using System.Threading;
using System.Threading.Tasks;

namespace CopyPaste.UI;

public sealed partial class App : Application, IDisposable
{
    private const string _launcherReadyEventName = "CopyPaste_AppReady";

    private Window? _window;
    private WindowsClipboardListener? _listener;
    private ClipboardService? _service;
    private CleanupService? _cleanupService;
    private SqliteRepository? _repository;
    private bool _isDisposed;
    public bool IsExiting { get; private set; }

    public App()
    {
        StorageConfig.Initialize();
        this.UnhandledException += OnUnhandledException;
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Initialize core services
        InitializeCoreServices();

        // Create and show main window
        _window = new MainWindow(_service!);
        _window.Activate();

        // Signal the native launcher that we're ready (it will close the splash)
        SignalLauncherReady();

        AppLogger.Info("Main window launched");
    }

    private void InitializeCoreServices()
    {
        // Initialize logger for error tracking
        AppLogger.Initialize();
        AppLogger.Info("Application starting...");

        // Load configuration once and cache it (never reloaded until app restart)
        var config = ConfigLoader.Config;

        // Register for Windows startup if configured
        RegisterForStartup();

        // Initialize core components
        _repository = new SqliteRepository(StorageConfig.DatabasePath);
        _service = new ClipboardService(_repository);
        _listener = new WindowsClipboardListener(_service);
        _cleanupService = new CleanupService(_repository, () => config.RetentionDays);

        // Configure paste timing (from cached config)
        _service.PasteIgnoreWindowMs = config.DuplicateIgnoreWindowMs;

        // Run listener in background
        Task.Run(() => _listener.Run());

        AppLogger.Info("Application initialized");
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

    /// <summary>
    /// Signals the native launcher that the app is ready.
    /// The launcher creates a Named Event and waits for this signal to close the splash screen.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types", Justification = "Launcher signaling is non-critical")]
    private static void SignalLauncherReady()
    {
        try
        {
            // Try to open the existing event created by the launcher
            using var readyEvent = EventWaitHandle.OpenExisting(_launcherReadyEventName);
            readyEvent.Set();
            AppLogger.Info("Signaled launcher that app is ready");
        }
        catch (WaitHandleCannotBeOpenedException)
        {
            // Event doesn't exist - app was started directly without launcher (e.g., debugging)
            // This is fine, just continue
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to signal launcher");
        }
    }

    private void OnUnhandledException(object sender, Microsoft.UI.Xaml.UnhandledExceptionEventArgs e)
    {
        AppLogger.Exception(e.Exception, "Unhandled exception");
        e.Handled = true;
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage(
        "Design",
        "CA1031:Do not catch general exception types",
        Justification = "Startup registration is non-critical - any failure should not prevent app from running")]
    private static void RegisterForStartup()
    {
        if (!ConfigLoader.Config.RunOnStartup) return;

        try
        {
            const string keyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
            const string appName = "CopyPaste";

            using var key = Microsoft.Win32.Registry.CurrentUser.OpenSubKey(keyPath, true);
            if (key == null) return;

            if (key.GetValue(appName) == null)
            {
                // Register the launcher (CopyPasteLauncher.exe) for Windows startup
                var appExePath = Environment.ProcessPath;
                if (!string.IsNullOrEmpty(appExePath))
                {
                    var appDir = Path.GetDirectoryName(appExePath)!;
                    var launcherPath = Path.Combine(appDir, "CopyPasteLauncher.exe");

                    // Use launcher if it exists, otherwise use current exe (for debugging)
                    var startupPath = File.Exists(launcherPath) ? launcherPath : appExePath;
                    key.SetValue(appName, $"\"{startupPath}\"");
                    AppLogger.Info($"Registered for Windows startup: {startupPath}");
                }
            }
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to register for startup");
        }
    }
}




