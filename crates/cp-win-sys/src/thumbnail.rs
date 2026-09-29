use std::os::windows::ffi::OsStrExt;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits,
    GetObjectW, HBITMAP, ReleaseDC,
};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_INCACHEONLY, SIIGBF_THUMBNAILONLY,
};
use windows::core::PCWSTR;

pub const SIDE: i32 = 256;
pub const SMALLEST: i32 = 64;

const _: () = assert!(SMALLEST < SIDE);

pub fn dib_of_file(path: &std::path::Path, side: i32) -> Option<Vec<u8>> {
    if side <= 0 {
        return None;
    }
    let _apartment = crate::com::Apartment::enter();
    let absolute = crate::com::shell_path(path)?;
    let wide: Vec<u16> = absolute
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let factory: IShellItemImageFactory =
        unsafe { SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None) }.ok()?;
    let wanted = windows::Win32::Foundation::SIZE { cx: side, cy: side };

    let cached = unsafe { factory.GetImage(wanted, SIIGBF_THUMBNAILONLY | SIIGBF_INCACHEONLY) };
    let bitmap = match cached {
        Ok(bitmap) => bitmap,

        Err(_) => unsafe { factory.GetImage(wanted, SIIGBF_THUMBNAILONLY) }.ok()?,
    };
    let dib = as_dib(bitmap, side);

    let _ = unsafe { DeleteObject(bitmap.into()) };
    dib
}

fn as_dib(bitmap: HBITMAP, side: i32) -> Option<Vec<u8>> {
    let mut shape = BITMAP::default();
    let wrote = i32::try_from(std::mem::size_of::<BITMAP>()).ok()?;

    let read = unsafe { GetObjectW(bitmap.into(), wrote, Some((&raw mut shape).cast())) };
    if read == 0 || is_an_icon(shape.bmWidth, shape.bmHeight, side) {
        return None;
    }
    let stride = usize::try_from(shape.bmWidth).ok()? * 4;
    let height = usize::try_from(shape.bmHeight).ok()?;
    let pixels = stride.checked_mul(height)?;

    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>()).ok()?,
            biWidth: shape.bmWidth,
            biHeight: shape.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: u32::try_from(pixels).ok()?,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut dib = vec![0u8; std::mem::size_of::<BITMAPINFOHEADER>() + pixels];

    let screen = unsafe { GetDC(None) };

    let lines = unsafe {
        GetDIBits(
            screen,
            bitmap,
            0,
            u32::try_from(shape.bmHeight).ok()?,
            Some(
                dib[std::mem::size_of::<BITMAPINFOHEADER>()..]
                    .as_mut_ptr()
                    .cast(),
            ),
            &raw mut info,
            DIB_RGB_COLORS,
        )
    };

    unsafe { ReleaseDC(None, screen) };
    if lines == 0 {
        return None;
    }

    let header = unsafe {
        std::slice::from_raw_parts(
            (&raw const info.bmiHeader).cast::<u8>(),
            std::mem::size_of::<BITMAPINFOHEADER>(),
        )
    };
    dib[..std::mem::size_of::<BITMAPINFOHEADER>()].copy_from_slice(header);
    Some(dib)
}

pub fn is_an_icon(width: i32, height: i32, asked_for: i32) -> bool {
    asked_for >= SIDE && width <= SMALLEST && height <= SMALLEST
}

#[cfg(test)]
#[path = "thumbnail_test.rs"]
mod tests;
