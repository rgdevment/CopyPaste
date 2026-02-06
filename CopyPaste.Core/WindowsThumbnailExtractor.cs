using SkiaSharp;
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
        object? factoryObj = null;

        try
        {
            int hr = SHCreateItemFromParsingName(filePath, IntPtr.Zero, _iShellItemImageFactoryGuid, out shellItem);
            if (hr != 0 || shellItem == IntPtr.Zero)
            {
                AppLogger.Warn($"[WindowsThumbnailExtractor] SHCreateItemFromParsingName failed with HRESULT: 0x{hr:X8} for {Path.GetFileName(filePath)}");
                return null;
            }

            // Get the IShellItemImageFactory interface (RCW adds a COM reference)
            factoryObj = Marshal.GetObjectForIUnknown(shellItem);
            var factory = (IShellItemImageFactory)factoryObj;

            // Request thumbnail with multiple flag combinations if first attempt fails
            var size = new SIZE { cx = width, cy = width };

            // Try with ThumbnailOnly first (strict)
            factory.GetImage(size, SIIGBF.ThumbnailOnly | SIIGBF.BiggerSizeOk, out hBitmap);

            // If that fails, try without ThumbnailOnly (allows icon fallback)
            if (hBitmap == IntPtr.Zero)
            {
                AppLogger.Info($"[WindowsThumbnailExtractor] ThumbnailOnly flag failed, trying with relaxed flags for {Path.GetFileName(filePath)}");
                factory.GetImage(size, SIIGBF.BiggerSizeOk | SIIGBF.MemoryOnly, out hBitmap);
            }

            // Last resort: try without any special flags
            if (hBitmap == IntPtr.Zero)
            {
                AppLogger.Info($"[WindowsThumbnailExtractor] All special flags failed, trying default for {Path.GetFileName(filePath)}");
                factory.GetImage(size, SIIGBF.ResizeToFit, out hBitmap);
            }

            if (hBitmap == IntPtr.Zero)
            {
                AppLogger.Warn($"[WindowsThumbnailExtractor] GetImage returned null HBITMAP for {Path.GetFileName(filePath)}");
                return null;
            }

            // Convert HBITMAP to byte array using SkiaSharp (Native AOT compatible)
            var result = HBitmapToBytes(hBitmap);
            if (result == null)
            {
                AppLogger.Warn($"[WindowsThumbnailExtractor] HBitmapToBytes failed for {Path.GetFileName(filePath)}");
            }
            return result;
        }
        catch (COMException ex)
        {
            AppLogger.Exception(ex, $"[WindowsThumbnailExtractor] COM error for {Path.GetFileName(filePath)}");
            return null;
        }
        catch (InvalidCastException ex)
        {
            AppLogger.Exception(ex, $"[WindowsThumbnailExtractor] InvalidCast error for {Path.GetFileName(filePath)}");
            return null;
        }
        catch (Exception ex)
        {
            AppLogger.Exception(ex, $"[WindowsThumbnailExtractor] Unexpected error for {Path.GetFileName(filePath)}");
            return null;
        }
        finally
        {
            if (hBitmap != IntPtr.Zero) DeleteObject(hBitmap);
            if (factoryObj != null) Marshal.ReleaseComObject(factoryObj);
            if (shellItem != IntPtr.Zero) Marshal.Release(shellItem);
        }
    }

    [DllImport("gdi32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
#pragma warning disable SYSLIB1054
    private static extern int GetDIBits(IntPtr hdc, IntPtr hbmp, uint uStartScan, uint cScanLines,
#pragma warning restore SYSLIB1054
        [Out] byte[] lpvBits, ref BITMAPINFO lpbi, uint uUsage);

    [LibraryImport("gdi32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial IntPtr CreateCompatibleDC(IntPtr hdc);

    [LibraryImport("gdi32.dll")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool DeleteDC(IntPtr hdc);

    [LibraryImport("gdi32.dll", EntryPoint = "GetObjectW")]
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    private static partial int GetObject(IntPtr hObject, int nCount, ref BITMAP lpObject);

    private static byte[]? HBitmapToBytes(IntPtr hBitmap)
    {
        IntPtr hdc = IntPtr.Zero;

        try
        {
            // Get bitmap info
            var bmp = new BITMAP();
            if (GetObject(hBitmap, Marshal.SizeOf<BITMAP>(), ref bmp) == 0)
                return null;

            int width = bmp.bmWidth;
            int height = bmp.bmHeight;

            if (width <= 0 || height <= 0)
                return null;

            // Setup BITMAPINFO for 32-bit BGRA
            var bi = new BITMAPINFO
            {
                bmiHeader = new BITMAPINFOHEADER
                {
                    biSize = (uint)Marshal.SizeOf<BITMAPINFOHEADER>(),
                    biWidth = width,
                    biHeight = -height, // Negative for top-down
                    biPlanes = 1,
                    biBitCount = 32,
                    biCompression = 0 // BI_RGB
                }
            };

            // Allocate pixel buffer
            int stride = width * 4;
            int bufferSize = stride * height;
            byte[] pixelData = new byte[bufferSize];

            hdc = CreateCompatibleDC(IntPtr.Zero);
            if (hdc == IntPtr.Zero)
                return null;

            // Get the bits
            int scanLines = GetDIBits(hdc, hBitmap, 0, (uint)height, pixelData, ref bi, 0);
            if (scanLines == 0)
                return null;

            // Create SkiaSharp bitmap from raw pixel data
            var info = new SKImageInfo(width, height, SKColorType.Bgra8888, SKAlphaType.Premul);
            using var skBitmap = new SKBitmap(info);

            // Copy pixel data to SKBitmap
            var pixels = skBitmap.GetPixels();
            Marshal.Copy(pixelData, 0, pixels, bufferSize);

            // Encode to JPEG
            using var image = SKImage.FromBitmap(skBitmap);
            using var data = image.Encode(SKEncodedImageFormat.Jpeg, ConfigLoader.Config.ThumbnailQualityJpeg);

            return data.ToArray();
        }
        catch (Exception)
        {
            return null;
        }
        finally
        {
            if (hdc != IntPtr.Zero) DeleteDC(hdc);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct SIZE
    {
        public int cx;
        public int cy;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BITMAP
    {
        public int bmType;
        public int bmWidth;
        public int bmHeight;
        public int bmWidthBytes;
        public ushort bmPlanes;
        public ushort bmBitsPixel;
        public IntPtr bmBits;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BITMAPINFOHEADER
    {
        public uint biSize;
        public int biWidth;
        public int biHeight;
        public ushort biPlanes;
        public ushort biBitCount;
        public uint biCompression;
        public uint biSizeImage;
        public int biXPelsPerMeter;
        public int biYPelsPerMeter;
        public uint biClrUsed;
        public uint biClrImportant;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BITMAPINFO
    {
        public BITMAPINFOHEADER bmiHeader;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 1)]
        public uint[] bmiColors;
    }

    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    [Guid("bcc18b79-ba16-442f-80c4-8a59c30c463b")]
    private partial interface IShellItemImageFactory
    {
        void GetImage(SIZE size, SIIGBF flags, out IntPtr phbm);
    }
}
