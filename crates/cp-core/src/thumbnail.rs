pub const MAX_SIDE: u32 = 256;

#[cfg(target_os = "windows")]
pub const THUMBNAILS_FILES: bool = true;
#[cfg(not(target_os = "windows"))]
pub const THUMBNAILS_FILES: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub fn size_of(bytes: &[u8]) -> Option<Size> {
    let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let (width, height) = reader.into_dimensions().ok()?;
    Some(Size { width, height })
}

pub fn of_wave(bars: &[u8], tallest: u8, width: u32, height: u32) -> Option<Vec<u8>> {
    if bars.is_empty() || width == 0 || height == 0 || tallest == 0 {
        return None;
    }
    let ink = image::Rgba([124u8, 134, 201, 255]);
    let mut canvas = image::RgbaImage::new(width, height);
    let gap = 1u32;
    let span = width / u32::try_from(bars.len()).ok()?.max(1);
    let thick = span.saturating_sub(gap).max(1);
    for (at, bar) in bars.iter().enumerate() {
        let left = u32::try_from(at).ok()? * span;
        let tall = (u32::from(*bar) * (height - 1) / u32::from(tallest)).max(1);
        let top = (height - tall) / 2;
        for x in left..(left + thick).min(width) {
            for y in top..(top + tall).min(height) {
                canvas.put_pixel(x, y, ink);
            }
        }
    }
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(canvas)
        .write_to(&mut out, image::ImageFormat::Png)
        .ok()?;
    Some(out.into_inner())
}

pub fn of_image(bytes: &[u8], max_side: u32) -> Option<Vec<u8>> {
    let decoded = image::load_from_memory(bytes).ok()?;
    let longest_side = decoded.width().max(decoded.height());
    let scaled = if longest_side > max_side {
        decoded.thumbnail(max_side, max_side)
    } else {
        decoded
    };
    let mut out = std::io::Cursor::new(Vec::new());
    scaled
        .write_to(&mut out, image::ImageFormat::Png)
        .ok()
        .map(|()| out.into_inner())
}

#[cfg(test)]
#[path = "thumbnail_test.rs"]
mod tests;
