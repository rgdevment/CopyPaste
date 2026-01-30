using CopyPaste.Core;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text.RegularExpressions;
using System.Threading.Channels;

namespace CopyPaste.Listener;

public sealed partial class WindowsClipboardListener(ClipboardService service) : IDisposable
{
    private const uint _cF_UNICODETEXT = 13;
    private const uint _cF_HDROP = 15;
    private const uint _cF_DIB = 8;
    private static readonly uint _cF_RTF = RegisterClipboardFormatW("Rich Text Format");
    private static readonly uint _cF_ExcludeHistory = RegisterClipboardFormatW("ExcludeClipboardContentFromMonitorProcessing");
    private static readonly uint _cF_CanInclude = RegisterClipboardFormatW("CanIncludeInClipboardHistory");

    private IntPtr _hwnd;
    private bool _disposed;

    private readonly Channel<ClipboardTask> _taskQueue = Channel.CreateUnbounded<ClipboardTask>();
    private readonly CancellationTokenSource _cts = new();

    // Debounce mechanism to prevent duplicate clipboard events
    private string? _lastContentHash;
    private DateTime _lastChangeTime = DateTime.MinValue;
    private readonly TimeSpan _debounceWindow = TimeSpan.FromMilliseconds(500);

    private sealed record ClipboardTask(ClipboardContentType Type, string? Text, byte[]? RtfBytes, byte[]? ImageBytes, Collection<string>? Files, string? Source);

    #region Win32 API
    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr CreateWindowExW(uint dwExStyle, string lpClassName, string lpWindowName, uint dwStyle, int x, int y, int nWidth, int nHeight, IntPtr hWndParent, IntPtr hMenu, IntPtr hInstance, IntPtr lpParam);

    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial uint RegisterClipboardFormatW(string lpszFormat);

    [LibraryImport("user32.dll", EntryPoint = "GetMessageW")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GetMessage(out MSG lpMsg, IntPtr hWnd, uint wMsgFilterMin, uint wMsgFilterMax);

    [LibraryImport("user32.dll", EntryPoint = "DispatchMessageW")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr DispatchMessage(ref MSG lpMsg);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool TranslateMessage(ref MSG lpMsg);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool AddClipboardFormatListener(IntPtr hwnd);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool OpenClipboard(IntPtr hWndNewOwner);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool CloseClipboard();

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr GetClipboardData(uint uFormat);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool IsClipboardFormatAvailable(uint format);

    [LibraryImport("kernel32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr GlobalLock(IntPtr hMem);

    [LibraryImport("kernel32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool GlobalUnlock(IntPtr hMem);

    [LibraryImport("kernel32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial nuint GlobalSize(IntPtr hMem);

    [LibraryImport("shell32.dll", StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial uint DragQueryFileW(IntPtr hDrop, uint iFile, [Out] char[]? lpszFile, uint cch);

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr GetClipboardOwner();

    [LibraryImport("user32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    #endregion

    public void Stop()
    {
        _cts.Cancel();
        _taskQueue.Writer.Complete();
    }

    public void Run()
    {
        _hwnd = CreateWindowExW(0, "Static", "CopyPasteHost", 0, 0, 0, 0, 0, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
        if (_hwnd == IntPtr.Zero)
        {
            AppLogger.Error("Failed to create clipboard listener window");
            return;
        }

        if (AddClipboardFormatListener(_hwnd))
        {
            AppLogger.Info("Clipboard listener started");
            _ = Task.Run(ProcessQueueAsync);

            while (GetMessage(out var msg, IntPtr.Zero, 0, 0))
            {
                if (msg.message == 0x031D) OnClipboardChanged();
                TranslateMessage(ref msg);
                DispatchMessage(ref msg);
            }
        }
        else
        {
            AppLogger.Error("Failed to add clipboard format listener");
        }
    }

    private async Task ProcessQueueAsync()
    {
        try
        {
            await foreach (var task in _taskQueue.Reader.ReadAllAsync(_cts.Token).ConfigureAwait(false))
            {
                await ProcessTaskWithRetryAsync(task).ConfigureAwait(false);
            }
        }
        catch (OperationCanceledException)
        {
            AppLogger.Info("Clipboard queue processing stopped");
        }
    }

    private async Task ProcessTaskWithRetryAsync(ClipboardTask task)
    {
        const int maxRetries = 3;
        int attempt = 0;

        while (attempt < maxRetries && !_cts.IsCancellationRequested)
        {
            try
            {
                DispatchToService(task);
                return;
            }
            catch (IOException ex)
            {
                attempt++;
                AppLogger.Warn($"IO error processing clipboard (attempt {attempt}/{maxRetries}): {ex.Message}");
                if (attempt < maxRetries)
                {
                    await Task.Delay(500 * attempt, _cts.Token).ConfigureAwait(false);
                }
            }
            catch (UnauthorizedAccessException ex)
            {
                AppLogger.Warn($"Access denied processing clipboard: {ex.Message}");
                return;
            }
            catch (Exception ex) when (ex is not OperationCanceledException)
            {
                AppLogger.Exception(ex, "Unexpected error processing clipboard task");
                return;
            }
        }
    }

    private void DispatchToService(ClipboardTask task)
    {
        switch (task.Type)
        {
            case ClipboardContentType.Text:
            case ClipboardContentType.Link:
                service.AddText(task.Text, task.Type, task.Source, task.RtfBytes);
                break;
            case ClipboardContentType.Image when task.ImageBytes != null:
                service.AddImage(task.ImageBytes, task.Source);
                break;
            case ClipboardContentType.Image:
            case ClipboardContentType.File:
            case ClipboardContentType.Folder:
            case ClipboardContentType.Audio:
            case ClipboardContentType.Video:
                service.AddFiles(task.Files, task.Type, task.Source);
                break;
        }
    }

    private void OnClipboardChanged()
    {
        Thread.Sleep(50);

        if (!OpenClipboard(_hwnd))
        {
            AppLogger.Warn("Failed to open clipboard - may be locked by another process");
            return;
        }

        try
        {
            if (ShouldExcludeFromHistory()) return;

            // Calculate content hash for deduplication
            string? contentHash = CalculateClipboardHash();
            if (contentHash != null && IsDuplicateChange(contentHash))
            {
                return; // Skip duplicate clipboard event
            }

            string? source = GetClipboardSource();

            if (IsClipboardFormatAvailable(_cF_HDROP))
            {
                var files = ExtractFilePaths();
                if (files.Count > 0)
                {
                    var type = DetectFileCollectionType(files);
                    _taskQueue.Writer.TryWrite(new ClipboardTask(type, null, null, null, files, source));
                }
            }
            else if (IsClipboardFormatAvailable(_cF_DIB))
            {
                var bytes = ExtractBytes(_cF_DIB);
                if (bytes != null && bytes.Length > 0)
                {
                    _taskQueue.Writer.TryWrite(new ClipboardTask(ClipboardContentType.Image, null, null, bytes, null, source));
                }
            }
            else if (IsClipboardFormatAvailable(_cF_UNICODETEXT))
            {
                var text = ExtractText();
                if (!string.IsNullOrEmpty(text))
                {
                    var type = DetectTextType(text);
                    byte[]? rtfBytes = IsClipboardFormatAvailable(_cF_RTF) ? ExtractBytes(_cF_RTF) : null;
                    _taskQueue.Writer.TryWrite(new ClipboardTask(type, text, rtfBytes, null, null, source));
                }
            }
        }
        catch (Exception ex) when (ex is not OutOfMemoryException)
        {
            AppLogger.Exception(ex, "Clipboard processing error");
        }
        finally { CloseClipboard(); }
    }

    private bool IsDuplicateChange(string contentHash)
    {
        var now = DateTime.UtcNow;
        if (contentHash == _lastContentHash && (now - _lastChangeTime) < _debounceWindow)
        {
            AppLogger.Info($"Skipping duplicate clipboard event (same content within {_debounceWindow.TotalMilliseconds}ms)");
            return true;
        }

        _lastContentHash = contentHash;
        _lastChangeTime = now;
        return false;
    }

    private static string? CalculateClipboardHash()
    {
        try
        {
            // Create a signature of what's currently on clipboard
            var signature = new List<string>();

            if (IsClipboardFormatAvailable(_cF_UNICODETEXT))
            {
                var text = ExtractText();
                if (!string.IsNullOrEmpty(text))
                {
                    // For text, use first 100 chars to avoid hashing huge content
                    var sample = text.Length > 100 ? text.Substring(0, 100) : text;
                    signature.Add($"TEXT:{sample}");
                }
            }

            if (IsClipboardFormatAvailable(_cF_HDROP))
            {
                var files = ExtractFilePaths();
                if (files.Count > 0)
                {
                    signature.Add($"FILES:{string.Join("|", files)}");
                }
            }

            if (IsClipboardFormatAvailable(_cF_DIB))
            {
                // For images, just mark that an image is present
                // Computing full hash would be expensive
                signature.Add("IMAGE:present");
            }

            if (signature.Count == 0) return null;

            // Simple hash of the signature
            var combined = string.Join("||", signature);
            var hash = SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(combined));
            return Convert.ToHexString(hash);
        }
        catch (Exception ex) when (ex is OutOfMemoryException or AccessViolationException)
        {
            AppLogger.Exception(ex, "Failed to calculate clipboard hash");
            return null;
        }
    }

    private static bool ShouldExcludeFromHistory()
    {
        if (IsClipboardFormatAvailable(_cF_ExcludeHistory)) return true;

        if (IsClipboardFormatAvailable(_cF_CanInclude))
        {
            var data = ExtractBytes(_cF_CanInclude);
            if (data != null && data.Length >= 4 && BitConverter.ToInt32(data, 0) == 0) return true;
        }

        return false;
    }

    internal static ClipboardContentType DetectTextType(string? text)
    {
        if (string.IsNullOrWhiteSpace(text)) return ClipboardContentType.Text;
        return UrlRegex().IsMatch(text.Trim()) ? ClipboardContentType.Link : ClipboardContentType.Text;
    }

    internal static ClipboardContentType DetectFileCollectionType(Collection<string> files)
    {
        // Only detect specific media types for single files.
        // Multiple files (mixed or same type) are treated as File to avoid expensive thumbnail generation.
        if (files == null || files.Count != 1) return ClipboardContentType.File;

        string path = files[0];
        if (Directory.Exists(path)) return ClipboardContentType.Folder;

        return FileExtensions.GetContentType(Path.GetExtension(path));
    }

    private static string? GetClipboardSource()
    {
        try
        {
            IntPtr owner = GetClipboardOwner();
            if (owner == IntPtr.Zero) return null;

            _ = GetWindowThreadProcessId(owner, out uint processId);
            if (processId == 0) return null;

            using var process = Process.GetProcessById((int)processId);
            return process.ProcessName;
        }
        catch (Exception ex) when (ex is ArgumentException or InvalidOperationException or System.ComponentModel.Win32Exception)
        {
            // Process no longer exists or access denied - expected scenarios
            return null;
        }
    }

    [GeneratedRegex(@"^https?://[^\s]+$", RegexOptions.IgnoreCase)]
    private static partial Regex UrlRegex();

    private static byte[]? ExtractBytes(uint format)
    {
        IntPtr hData = GetClipboardData(format);
        if (hData == IntPtr.Zero) return null;

        IntPtr pData = GlobalLock(hData);
        if (pData == IntPtr.Zero)
        {
            return null;
        }

        try
        {
            nuint rawSize = GlobalSize(hData);
            if (rawSize == 0 || rawSize > int.MaxValue)
            {
                return null;
            }

            int size = (int)rawSize;
            byte[] buffer = new byte[size];
            Marshal.Copy(pData, buffer, 0, size);
            return buffer;
        }
        catch (OutOfMemoryException ex)
        {
            AppLogger.Exception(ex, "Out of memory extracting clipboard data");
            return null;
        }
        finally { GlobalUnlock(hData); }
    }

    private static string? ExtractText()
    {
        IntPtr hData = GetClipboardData(_cF_UNICODETEXT);
        if (hData == IntPtr.Zero) return null;

        IntPtr pData = GlobalLock(hData);
        if (pData == IntPtr.Zero)
        {
            return null;
        }

        try
        {
            return Marshal.PtrToStringUni(pData);
        }
        catch (Exception ex) when (ex is AccessViolationException or ArgumentException)
        {
            AppLogger.Exception(ex, "Text extraction failed");
            return null;
        }
        finally { GlobalUnlock(hData); }
    }

    private static Collection<string> ExtractFilePaths()
    {
        List<string> files = [];

        try
        {
            IntPtr hData = GetClipboardData(_cF_HDROP);
            if (hData == IntPtr.Zero) return new Collection<string>(files);

            uint count = DragQueryFileW(hData, 0xFFFFFFFF, null, 0);
            if (count == 0 || count > 10000)
            {
                return new Collection<string>(files);
            }

            for (uint i = 0; i < count; i++)
            {
                uint length = DragQueryFileW(hData, i, null, 0);
                if (length == 0 || length > 32767) continue; // MAX_PATH extended

                char[] buffer = new char[length + 1];
                if (DragQueryFileW(hData, i, buffer, (uint)buffer.Length) > 0)
                {
                    string path = new string(buffer).TrimEnd('\0');
                    if (!string.IsNullOrWhiteSpace(path))
                    {
                        files.Add(path);
                    }
                }
            }
        }
        catch (Exception ex) when (ex is OutOfMemoryException or AccessViolationException)
        {
            AppLogger.Exception(ex, "File path extraction failed");
        }

        return new Collection<string>(files);
    }

    public void Dispose()
    {
        if (_disposed) return;
        _cts.Cancel();
        _cts.Dispose();
        _disposed = true;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct MSG { public IntPtr hwnd; public uint message; public IntPtr wParam; public IntPtr lParam; public uint time; public System.Drawing.Point pt; }
}

