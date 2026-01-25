using System.Runtime.InteropServices;

namespace CopyPaste.Core;

[System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types")]
public static partial class WindowsThumbnailExtractor
{
    private static readonly Guid _iShellItemImageFactoryGuid = new("bcc18b79-ba16-442f-80c4-8a59c30c463b");

    [Flags]
    private enum SIIGBF : uint
    {
        ResizeToFit = 0x00000000,
        BiggerSizeOk = 0x00000001,
        MemoryOnly = 0x00000002,
        IconOnly = 0x00000004,
        ThumbnailOnly = 0x00000008,
        InCacheOnly = 0x00000010,
        ScaleUp = 0x00000100,
    }

    [LibraryImport("shell32.dll", SetLastError = true, StringMarshalling = StringMarshalling.Utf16)]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial int SHCreateItemFromParsingName(
        string pszPath,
        IntPtr pbc,
        in Guid riid,
        out IntPtr ppv);

    [LibraryImport("gdi32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool DeleteObject(IntPtr hObject);

    public static byte[]? GetThumbnail(string filePath, int width = 300)
    {
        if (string.IsNullOrWhiteSpace(filePath)) return null;
        if (!File.Exists(filePath)) return null;
        if (width <= 0 || width > 4096) return null;

        IntPtr shellItem = IntPtr.Zero;
        IntPtr hBitmap = IntPtr.Zero;

        try
        {
            int hr = SHCreateItemFromParsingName(filePath, IntPtr.Zero, _iShellItemImageFactoryGuid, out shellItem);
            if (hr != 0 || shellItem == IntPtr.Zero)
            {
                System.Diagnostics.Debug.WriteLine($"SHCreateItemFromParsingName failed: HRESULT 0x{hr:X8}");
                return null;
            }

            // Get the IShellItemImageFactory interface
            var factory = (IShellItemImageFactory)Marshal.GetObjectForIUnknown(shellItem);

            // Request thumbnail
            var size = new SIZE { cx = width, cy = width };
            factory.GetImage(size, SIIGBF.ThumbnailOnly | SIIGBF.BiggerSizeOk, out hBitmap);

            if (hBitmap == IntPtr.Zero)
            {
                System.Diagnostics.Debug.WriteLine($"GetImage returned null bitmap for: {Path.GetFileName(filePath)}");
                return null;
            }

            // Convert HBITMAP to byte array
            return HBitmapToPngBytes(hBitmap);
        }
        catch (COMException ex)
        {
            System.Diagnostics.Debug.WriteLine($"COM error extracting thumbnail: 0x{ex.HResult:X8} - {ex.Message}");
            return null;
        }
        catch (InvalidCastException ex)
        {
            System.Diagnostics.Debug.WriteLine($"Interface cast failed: {ex.Message}");
            return null;
        }
        catch (Exception ex) when (ex is OutOfMemoryException or ExternalException)
        {
            System.Diagnostics.Debug.WriteLine($"Resource error: {ex.GetType().Name} - {ex.Message}");
            return null;
        }
        finally
        {
            if (hBitmap != IntPtr.Zero) DeleteObject(hBitmap);
            if (shellItem != IntPtr.Zero) Marshal.Release(shellItem);
        }
    }

    private static byte[]? HBitmapToPngBytes(IntPtr hBitmap)
    {
        try
        {
            using var bitmap = System.Drawing.Image.FromHbitmap(hBitmap);
            using var ms = new MemoryStream();
            bitmap.Save(ms, System.Drawing.Imaging.ImageFormat.Png);
            return ms.ToArray();
        }
        catch (ExternalException ex)
        {
            System.Diagnostics.Debug.WriteLine($"GDI+ error converting bitmap: {ex.Message}");
            return null;
        }
        catch (ArgumentException ex)
        {
            System.Diagnostics.Debug.WriteLine($"Invalid bitmap handle: {ex.Message}");
            return null;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct SIZE
    {
        public int cx;
        public int cy;
    }

    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    [Guid("bcc18b79-ba16-442f-80c4-8a59c30c463b")]
    private partial interface IShellItemImageFactory
    {
        void GetImage(SIZE size, SIIGBF flags, out IntPtr phbm);
    }
}
