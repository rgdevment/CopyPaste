const FILE_HEADER: usize = 14;
const INFO_HEADER: usize = 40;
const BI_BITFIELDS: u32 = 3;
const RGBQUAD: usize = 4;

pub const LARGEST_BITMAP: usize = 256 * 1024 * 1024;

const _: () = assert!(7680 * 4320 * 4 < LARGEST_BITMAP);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub size: u32,
    pub width: i32,
    pub height: i32,
    pub bit_count: u16,
    pub compression: u32,
    pub clr_used: u32,
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    bytes
        .get(at..at + 4)
        .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]))
}

pub fn header(dib: &[u8]) -> Option<Header> {
    let size = u32_at(dib, 0)?;
    if (size as usize) < INFO_HEADER || size as usize > dib.len() {
        return None;
    }
    Some(Header {
        size,
        width: u32_at(dib, 4)? as i32,
        height: u32_at(dib, 8)? as i32,
        bit_count: u16::from_le_bytes([*dib.get(14)?, *dib.get(15)?]),
        compression: u32_at(dib, 16)?,
        clr_used: u32_at(dib, 32)?,
    })
}

pub fn pixel_offset(dib: &[u8]) -> Option<usize> {
    let head = header(dib)?;
    let mut table = 0usize;

    if head.compression == BI_BITFIELDS && head.size as usize == INFO_HEADER {
        table += 3 * 4;
    }

    if head.bit_count <= 8 {
        let colors = if head.clr_used > 0 {
            head.clr_used
        } else {
            1u32 << head.bit_count
        };
        table += colors as usize * RGBQUAD;
    } else if head.clr_used != 0 {
        let claimed = head.clr_used as usize * RGBQUAD;
        if head.size as usize + table + claimed <= dib.len() {
            table += claimed;
        }
    }

    let offset = head.size as usize + table;
    (offset <= dib.len()).then_some(offset)
}

pub fn as_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    let pixels = pixel_offset(dib)?;
    let mut bmp = Vec::with_capacity(FILE_HEADER + dib.len());
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&((FILE_HEADER + dib.len()) as u32).to_le_bytes());
    bmp.extend_from_slice(&0u16.to_le_bytes());
    bmp.extend_from_slice(&0u16.to_le_bytes());
    bmp.extend_from_slice(&((FILE_HEADER + pixels) as u32).to_le_bytes());
    bmp.extend_from_slice(dib);
    Some(bmp)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alpha {
    Absent,
    Opaque,
    Real,
}

fn pixel_len(head: Header) -> Option<usize> {
    let width = usize::try_from(head.width.unsigned_abs()).ok()?;
    let height = usize::try_from(head.height.unsigned_abs()).ok()?;
    let stride = width
        .checked_mul(usize::from(head.bit_count))?
        .checked_add(31)?
        / 32
        * 4;
    stride.checked_mul(height)
}

fn pixels_of(dib: &[u8]) -> Option<&[u8]> {
    let head = header(dib)?;
    let start = pixel_offset(dib)?;
    let pixels = dib.get(start..)?;
    Some(match pixel_len(head) {
        Some(len) if len <= pixels.len() => &pixels[..len],
        _ => pixels,
    })
}

fn declared_alpha_mask(dib: &[u8], head: Header) -> Option<u32> {
    (head.size as usize >= 56)
        .then(|| u32_at(dib, 52))
        .flatten()
}

pub fn alpha(dib: &[u8]) -> Alpha {
    let Some(head) = header(dib) else {
        return Alpha::Absent;
    };
    if head.bit_count != 32 {
        return Alpha::Absent;
    }
    if declared_alpha_mask(dib, head) == Some(0) {
        return Alpha::Absent;
    }
    let Some(pixels) = pixels_of(dib) else {
        return Alpha::Absent;
    };
    let mut seen_zero = false;
    let mut seen_full = false;
    let mut seen_between = false;
    for chunk in pixels.as_chunks::<4>().0 {
        match chunk[3] {
            0x00 => seen_zero = true,
            0xFF => seen_full = true,
            _ => seen_between = true,
        }
    }
    if seen_between || (seen_zero && seen_full) {
        Alpha::Real
    } else {
        Alpha::Opaque
    }
}

fn too_large(len: usize) -> bool {
    len > LARGEST_BITMAP
}

pub fn to_png(dib: &[u8]) -> Option<Vec<u8>> {
    if too_large(dib.len()) {
        return None;
    }
    let head = header(dib)?;
    let mut bmp = as_bmp(dib)?;
    if head.bit_count == 32 && alpha(dib) != Alpha::Real {
        let from = FILE_HEADER + pixel_offset(dib)?;
        let pixels = bmp.get_mut(from..)?;
        let to = pixel_len(head).unwrap_or(pixels.len()).min(pixels.len());
        for chunk in pixels.get_mut(..to)?.as_chunks_mut::<4>().0 {
            chunk[3] = 0xFF;
        }
    }
    let decoded = image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp).ok()?;
    let mut out = std::io::Cursor::new(Vec::new());
    decoded
        .write_to(&mut out, image::ImageFormat::Png)
        .ok()
        .map(|()| out.into_inner())
}

pub fn from_png(png: &[u8]) -> Option<Vec<u8>> {
    from_encoded(png, image::ImageFormat::Png)
}

pub fn from_jpeg(jpeg: &[u8]) -> Option<Vec<u8>> {
    from_encoded(jpeg, image::ImageFormat::Jpeg)
}

fn from_encoded(encoded: &[u8], format: image::ImageFormat) -> Option<Vec<u8>> {
    if too_large(encoded.len()) {
        return None;
    }
    let decoded = image::load_from_memory_with_format(encoded, format).ok()?;
    from_image(&decoded)
}

pub fn from_image(decoded: &image::DynamicImage) -> Option<Vec<u8>> {
    if decoded.width() == 0 || decoded.height() == 0 {
        return None;
    }
    let mut bmp = std::io::Cursor::new(Vec::new());
    decoded.write_to(&mut bmp, image::ImageFormat::Bmp).ok()?;
    let bmp = bmp.into_inner();
    bmp.get(FILE_HEADER..)
        .filter(|pixels| !pixels.is_empty())
        .map(<[u8]>::to_vec)
}

#[cfg(test)]
#[path = "dib_test.rs"]
mod tests;

#[cfg(test)]
#[path = "dib_properties_test.rs"]
mod properties;
