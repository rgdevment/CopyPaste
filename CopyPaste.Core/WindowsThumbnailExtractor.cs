using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

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
        if (!File.Exists(filePath)) return null;

        IntPtr shellItem = IntPtr.Zero;
        IntPtr hBitmap = IntPtr.Zero;

        try
        {
            int hr = SHCreateItemFromParsingName(filePath, IntPtr.Zero, _iShellItemImageFactoryGuid, out shellItem);
            if (hr != 0 || shellItem == IntPtr.Zero) return null;

            // Get the IShellItemImageFactory interface
            var factory = (IShellItemImageFactory)Marshal.GetObjectForIUnknown(shellItem);

            // Request thumbnail
            var size = new SIZE { cx = width, cy = width };
            factory.GetImage(size, SIIGBF.ThumbnailOnly | SIIGBF.BiggerSizeOk, out hBitmap);

            if (hBitmap == IntPtr.Zero) return null;

            // Convert HBITMAP to byte array
            return HBitmapToPngBytes(hBitmap);
        }
        catch
        {
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
        catch
        {
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
