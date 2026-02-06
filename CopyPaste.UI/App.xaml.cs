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
using CopyPaste.UI.Localization;
using Microsoft.UI.Xaml;
using System;
using System.Diagnostics;
using System.IO;
using System.Threading;

namespace CopyPaste.UI;

public sealed partial class App : Application, IDisposable
{
    private const string _launcherReadyEventName = "CopyPaste_AppReady";
    private const string _mutexName = "Global\\CopyPaste_SingleInstance_Mutex";

    private Window? _window;
    private CopyPasteEngine? _engine;
    private Mutex? _singleInstanceMutex;
    private bool _isDisposed;
    public bool IsExiting { get; private set; }

    public App()
    {
        WaitForPreviousInstanceIfNeeded();

        if (!TryAcquireSingleInstance())
        {
            Environment.Exit(0);
            return;
        }

        StorageConfig.Initialize();
        this.UnhandledException += OnUnhandledException;
        InitializeComponent();
    }

    private static void WaitForPreviousInstanceIfNeeded()
    {
        var args = Environment.GetCommandLineArgs();
        for (int i = 0; i < args.Length - 1; i++)
        {
            if (args[i] == "--wait-for-pid" && int.TryParse(args[i + 1], out int pid))
            {
                try
                {
                    using var process = Process.GetProcessById(pid);
                    process.WaitForExit(5000);
                }
                catch (ArgumentException) { /* Process already exited */ }
                catch (InvalidOperationException) { /* Process already exited */ }
                break;
            }
        }
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        InitializeCoreServices();

        var config = _engine!.Config;
        L.Initialize(new LocalizationService(config.PreferredLanguage));

        _window = new MainWindow(_engine.Service, config);
        _window.Activate();

        SignalLauncherReady();
        AppLogger.Info("Main window launched");
    }

    private void InitializeCoreServices()
    {
        RegisterForStartup();
        _engine = new CopyPasteEngine(svc => new WindowsClipboardListener(svc));
        _engine.Start();
    }

    public void BeginExit()
    {
        IsExiting = true;
        AppLogger.Info("Application exiting...");

        _engine?.Dispose();

        L.Dispose();
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
            _engine?.Dispose();
            _singleInstanceMutex?.ReleaseMutex();
            _singleInstanceMutex?.Dispose();
        }
        _isDisposed = true;
    }

    /// <summary>
    /// Attempts to acquire the single instance mutex.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage(
        "Design",
        "CA1031:Do not catch general exception types",
        Justification = "Single instance check is non-critical - any failure should not prevent app from running")]
    private bool TryAcquireSingleInstance()
    {
        try
        {
            _singleInstanceMutex = new Mutex(true, _mutexName, out bool createdNew);

            if (!createdNew)
            {
                _singleInstanceMutex.Dispose();
                _singleInstanceMutex = null;
                ShowInstanceAlreadyRunningMessage();
                return false;
            }

            return true;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, "Failed to check for single instance");
            return true;
        }
    }

    [System.Diagnostics.CodeAnalysis.SuppressMessage(
        "Design",
        "CA1031:Do not catch general exception types",
        Justification = "Message display is non-critical")]
    private static void ShowInstanceAlreadyRunningMessage()
    {
        try
        {
            var hwnd = IntPtr.Zero;
            var message = "CopyPaste is already running.\n\nCheck the system tray for the application icon.";
            var caption = "CopyPaste";
            const uint MB_OK = 0x00000000;
            const uint MB_ICONINFORMATION = 0x00000040;

            _ = MessageBox(hwnd, message, caption, MB_OK | MB_ICONINFORMATION);
        }
        catch
        {
        }
    }

    [System.Runtime.InteropServices.DllImport("user32.dll", CharSet = System.Runtime.InteropServices.CharSet.Unicode)]
    private static extern int MessageBox(IntPtr hWnd, string text, string caption, uint type);

    /// <summary>
    /// Signals the native launcher that the app is ready.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types", Justification = "Launcher signaling is non-critical")]
    private static void SignalLauncherReady()
    {
        try
        {
            using var readyEvent = EventWaitHandle.OpenExisting(_launcherReadyEventName);
            readyEvent.Set();
            AppLogger.Info("Signaled launcher that app is ready");
        }
        catch (WaitHandleCannotBeOpenedException)
        {
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
    private void RegisterForStartup()
    {
        if (!_engine!.Config.RunOnStartup) return;

        try
        {
            const string keyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
            const string appName = "CopyPaste";

            using var key = Microsoft.Win32.Registry.CurrentUser.OpenSubKey(keyPath, true);
            if (key == null) return;

            if (key.GetValue(appName) == null)
            {
                var appExePath = Environment.ProcessPath;
                if (!string.IsNullOrEmpty(appExePath))
                {
                    var appDir = Path.GetDirectoryName(appExePath)!;
                    var launcherPath = Path.Combine(appDir, "CopyPaste.exe");
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




