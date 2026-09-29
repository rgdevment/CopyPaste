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
