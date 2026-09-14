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
    if too_large(png.len()) {
        return None;
    }
    let decoded = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?;
    let mut bmp = std::io::Cursor::new(Vec::new());
    decoded.write_to(&mut bmp, image::ImageFormat::Bmp).ok()?;
    let bmp = bmp.into_inner();
    (bmp.len() > FILE_HEADER).then(|| bmp[FILE_HEADER..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dib {
        header_size: u32,
        width: i32,
        height: i32,
        bit_count: u16,
        compression: u32,
        clr_used: u32,
        table: Vec<u8>,
        pixels: Vec<u8>,
    }

    impl Dib {
        fn rgb32(width: i32, height: i32) -> Self {
            Self {
                header_size: INFO_HEADER as u32,
                width,
                height,
                bit_count: 32,
                compression: 0,
                clr_used: 0,
                table: Vec::new(),
                pixels: [0x40, 0x80, 0xC0, 0xFF].repeat((width * height.abs()) as usize),
            }
        }

        fn build(&self) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(&self.header_size.to_le_bytes());
            out.extend_from_slice(&self.width.to_le_bytes());
            out.extend_from_slice(&self.height.to_le_bytes());
            out.extend_from_slice(&1u16.to_le_bytes());
            out.extend_from_slice(&self.bit_count.to_le_bytes());
            out.extend_from_slice(&self.compression.to_le_bytes());
            out.extend_from_slice(&(self.pixels.len() as u32).to_le_bytes());
            out.extend_from_slice(&2835i32.to_le_bytes());
            out.extend_from_slice(&2835i32.to_le_bytes());
            out.extend_from_slice(&self.clr_used.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.resize(self.header_size as usize, 0);
            out.extend_from_slice(&self.table);
            out.extend_from_slice(&self.pixels);
            out
        }
    }

    #[test]
    fn a_plain_dib_puts_the_pixels_right_after_the_header() {
        let dib = Dib::rgb32(2, 2).build();
        assert_eq!(pixel_offset(&dib), Some(INFO_HEADER));
    }

    #[test]
    fn bitfield_masks_follow_the_classic_header() {
        let mut dib = Dib::rgb32(2, 2);
        dib.compression = BI_BITFIELDS;
        dib.table = vec![0; 12];
        assert_eq!(pixel_offset(&dib.build()), Some(INFO_HEADER + 12));
    }

    #[test]
    fn bitfield_masks_live_inside_the_newer_headers() {
        for size in [108u32, 124] {
            let mut dib = Dib::rgb32(2, 2);
            dib.header_size = size;
            dib.compression = BI_BITFIELDS;
            assert_eq!(
                pixel_offset(&dib.build()),
                Some(size as usize),
                "cabecera de {size} bytes"
            );
        }
    }

    #[test]
    fn a_palette_below_nine_bits_shifts_the_pixels() {
        let mut dib = Dib::rgb32(4, 1);
        dib.bit_count = 8;
        dib.pixels = vec![0, 1, 2, 3];
        dib.table = vec![0; 256 * RGBQUAD];
        assert_eq!(
            pixel_offset(&dib.build()),
            Some(INFO_HEADER + 256 * RGBQUAD),
            "sin biClrUsed se asume la paleta entera"
        );
    }

    #[test]
    fn a_declared_palette_size_is_honoured() {
        let mut dib = Dib::rgb32(4, 1);
        dib.bit_count = 8;
        dib.clr_used = 16;
        dib.pixels = vec![0, 1, 2, 3];
        dib.table = vec![0; 16 * RGBQUAD];
        assert_eq!(pixel_offset(&dib.build()), Some(INFO_HEADER + 16 * RGBQUAD));
    }

    #[test]
    fn a_dirty_palette_count_above_eight_bits_is_ignored() {
        let mut dib = Dib::rgb32(2, 2);
        dib.clr_used = 1_000_000;
        assert_eq!(
            pixel_offset(&dib.build()),
            Some(INFO_HEADER),
            "lo que no cabe no se resta"
        );
    }

    #[test]
    fn a_palette_that_does_fit_above_eight_bits_is_applied() {
        let mut dib = Dib::rgb32(2, 2);
        dib.clr_used = 2;
        dib.table = vec![0; 2 * RGBQUAD];
        assert_eq!(pixel_offset(&dib.build()), Some(INFO_HEADER + 2 * RGBQUAD));
    }

    #[test]
    fn a_header_that_fills_the_whole_buffer_is_still_a_header() {
        let mut dib = Dib::rgb32(1, 1);
        dib.pixels = Vec::new();
        let raw = dib.build();
        assert_eq!(raw.len(), INFO_HEADER);
        assert_eq!(
            header(&raw).map(|head| head.size),
            Some(INFO_HEADER as u32),
            "medir justo lo que hay no es medir de más"
        );
        assert_eq!(pixel_offset(&raw), Some(INFO_HEADER));
    }

    #[test]
    fn a_palette_one_byte_too_long_is_left_out() {
        let mut dib = Dib::rgb32(2, 2);
        dib.clr_used = 2;
        dib.pixels = vec![0; 5];
        let raw = dib.build();
        assert_eq!(raw.len(), INFO_HEADER + 5);
        assert_eq!(
            pixel_offset(&raw),
            Some(INFO_HEADER),
            "ocho bytes de paleta no caben en cinco"
        );
    }

    #[test]
    fn the_masks_count_towards_whether_the_palette_fits() {
        let mut dib = Dib::rgb32(2, 2);
        dib.compression = BI_BITFIELDS;
        dib.clr_used = 3;
        dib.table = vec![0; 12];
        dib.pixels = vec![0; 8];
        let raw = dib.build();
        assert_eq!(raw.len(), INFO_HEADER + 12 + 8);
        assert_eq!(
            pixel_offset(&raw),
            Some(INFO_HEADER + 12),
            "doce de máscaras más doce de paleta no caben en veinte"
        );
    }

    #[test]
    fn nonsense_is_not_a_bitmap() {
        assert_eq!(header(&[]), None);
        assert_eq!(header(&[0; 8]), None, "ni siquiera llega a la cabecera");
        assert_eq!(header(&[0; 40]), None, "una cabecera que dice medir cero");
        let mut lying = Dib::rgb32(2, 2).build();
        lying[0..4].copy_from_slice(&9999u32.to_le_bytes());
        assert_eq!(header(&lying), None, "dice medir más que todo el buffer");
        assert_eq!(as_bmp(&[]), None);
        assert_eq!(to_png(&[]), None);
    }

    #[test]
    fn the_file_header_points_at_the_pixels() {
        let dib = Dib::rgb32(2, 2).build();
        let bmp = as_bmp(&dib).expect("bmp");
        assert_eq!(&bmp[0..2], b"BM");
        assert_eq!(
            u32::from_le_bytes([bmp[2], bmp[3], bmp[4], bmp[5]]) as usize,
            FILE_HEADER + dib.len(),
            "el tamaño declarado es el del archivo entero"
        );
        assert_eq!(
            u32::from_le_bytes([bmp[10], bmp[11], bmp[12], bmp[13]]) as usize,
            FILE_HEADER + INFO_HEADER
        );
        assert_eq!(&bmp[FILE_HEADER..], &dib[..], "el DIB viaja intacto");
    }

    #[test]
    fn an_alpha_channel_nobody_wrote_is_not_transparency() {
        let mut dib = Dib::rgb32(2, 2);
        dib.pixels = [0x40, 0x80, 0xC0, 0x00].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Opaque);
    }

    #[test]
    fn an_alpha_channel_fully_opaque_is_opaque() {
        let dib = Dib::rgb32(2, 2).build();
        assert_eq!(alpha(&dib), Alpha::Opaque);
    }

    #[test]
    fn a_mixed_alpha_channel_is_real_transparency() {
        let mut dib = Dib::rgb32(2, 2);
        dib.pixels = vec![
            0x40, 0x80, 0xC0, 0xFF, //
            0x40, 0x80, 0xC0, 0x00, //
            0x40, 0x80, 0xC0, 0xFF, //
            0x40, 0x80, 0xC0, 0xFF,
        ];
        assert_eq!(
            alpha(&dib.build()),
            Alpha::Real,
            "unos a cero y otros opacos es un recorte con bordes"
        );
    }

    #[test]
    fn a_partial_alpha_value_is_real_transparency() {
        let mut dib = Dib::rgb32(2, 2);
        dib.pixels = [0x40, 0x80, 0xC0, 0x7F].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Real);
    }

    #[test]
    fn a_header_promising_more_than_the_buffer_holds_has_no_alpha() {
        let mut dib = Dib::rgb32(1, 1);
        dib.compression = BI_BITFIELDS;
        dib.pixels = Vec::new();
        let raw = dib.build();
        assert_eq!(raw.len(), INFO_HEADER);
        assert_eq!(pixel_offset(&raw), None, "las máscaras no caben");
        assert_eq!(alpha(&raw), Alpha::Absent);
        assert_eq!(to_png(&raw), None);
    }

    #[test]
    fn a_synthesised_v5_declaring_no_alpha_mask_has_no_alpha() {
        let mut dib = Dib::rgb32(2, 2);
        dib.header_size = 124;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        let raw = dib.build();
        assert_eq!(u32::from_le_bytes([raw[52], raw[53], raw[54], raw[55]]), 0);
        assert_eq!(
            alpha(&raw),
            Alpha::Absent,
            "el cuarto byte parece alfa, pero la cabecera dice que no lo es"
        );
    }

    #[test]
    fn a_synthesised_v5_does_not_become_half_transparent() {
        let mut dib = Dib::rgb32(4, 4);
        dib.header_size = 124;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(16);
        let png = to_png(&dib.build()).expect("png");
        let back = image::load_from_memory(&png).expect("se relee").to_rgba8();
        assert!(
            back.pixels().all(|pixel| pixel[3] == 0xFF),
            "una transparencia que nadie declaró no se graba"
        );
    }

    #[test]
    fn the_shortest_header_that_declares_alpha_is_fifty_six_bytes() {
        let mut dib = Dib::rgb32(2, 2);
        dib.header_size = 56;
        dib.compression = BI_BITFIELDS;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        let mut raw = dib.build();
        raw[40..44].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
        raw[44..48].copy_from_slice(&0x0000_FF00u32.to_le_bytes());
        raw[48..52].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(pixel_offset(&raw), Some(56), "las máscaras van dentro");
        assert_eq!(
            alpha(&raw),
            Alpha::Real,
            "declara la máscara de alfa y hay que leerla"
        );
    }

    #[test]
    fn a_fifty_six_byte_header_with_no_mask_has_no_alpha() {
        let mut dib = Dib::rgb32(2, 2);
        dib.header_size = 56;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Absent);
    }

    #[test]
    fn bytes_past_the_pixel_array_do_not_vote_on_the_alpha() {
        let mut dib = Dib::rgb32(4, 4);
        dib.header_size = 124;
        dib.compression = BI_BITFIELDS;
        dib.pixels = [0x20, 0x60, 0xA0, 0x00].repeat(16);
        let mut raw = dib.build();
        raw[40..44].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
        raw[44..48].copy_from_slice(&0x0000_FF00u32.to_le_bytes());
        raw[48..52].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(alpha(&raw), Alpha::Opaque);

        raw.extend_from_slice(&[0x00, 0x00, 0x00, 0xFF]);
        assert_eq!(
            alpha(&raw),
            Alpha::Opaque,
            "cuatro bytes de cola daban la captura por transparente"
        );
        let back = image::load_from_memory(&to_png(&raw).expect("png"))
            .expect("se relee")
            .to_rgba8();
        assert!(back.pixels().all(|pixel| pixel[3] == 0xFF));
    }

    #[test]
    fn a_declared_mask_turns_the_same_pixels_into_real_alpha() {
        let mut dib = Dib::rgb32(2, 2);
        dib.header_size = 124;
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        let mut raw = dib.build();
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(alpha(&raw), Alpha::Real);
    }

    #[test]
    fn the_classic_header_has_no_alpha_mask_to_declare() {
        let mut dib = Dib::rgb32(2, 2);
        dib.compression = BI_BITFIELDS;
        dib.table = vec![0; 12];
        dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Real);
    }

    #[test]
    fn below_thirty_two_bits_there_is_no_alpha_to_read() {
        let mut dib = Dib::rgb32(2, 2);
        dib.bit_count = 24;
        dib.pixels = [0x40, 0x80, 0xC0].repeat(4);
        assert_eq!(alpha(&dib.build()), Alpha::Absent);
    }

    #[test]
    fn a_bitmap_becomes_a_fraction_of_its_size_as_png() {
        let mut dib = Dib::rgb32(200, 200);
        dib.pixels = [0x20, 0x60, 0xA0, 0xFF].repeat(200 * 200);
        let raw = dib.build();
        let png = to_png(&raw).expect("png");
        assert!(
            png.len() * 20 < raw.len(),
            "{} bytes de DIB contra {} de PNG",
            raw.len(),
            png.len()
        );
    }

    #[test]
    fn the_size_limit_falls_between_the_last_accepted_byte_and_the_first_refused() {
        assert!(!too_large(LARGEST_BITMAP - 1));
        assert!(!too_large(LARGEST_BITMAP));
        assert!(too_large(LARGEST_BITMAP + 1));
        assert!(too_large(usize::MAX));
        assert!(!too_large(0));
    }

    #[test]
    fn a_bitmap_too_large_to_be_a_screen_is_refused_before_it_is_copied() {
        let mut dib = Dib::rgb32(1, 1);
        dib.pixels = vec![0; LARGEST_BITMAP + 1 - INFO_HEADER];
        let raw = dib.build();
        assert!(too_large(raw.len()));
        assert_eq!(to_png(&raw), None);
    }

    #[test]
    fn a_png_becomes_a_bitmap_the_clipboard_understands() {
        let dib = Dib::rgb32(4, 4).build();
        let png = to_png(&dib).expect("png");
        let back = from_png(&png).expect("dib");
        let head = header(&back).expect("cabecera");
        assert_eq!((head.width, head.height.abs()), (4, 4));
        assert!(pixel_offset(&back).is_some());
    }

    #[test]
    fn the_round_trip_keeps_the_colours_where_they_were() {
        let mut dib = Dib::rgb32(2, 2);
        dib.pixels = vec![
            0x00, 0x00, 0xFF, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];
        let once = image::load_from_memory(&to_png(&dib.build()).expect("png"))
            .expect("relee")
            .to_rgba8();
        let twice = image::load_from_memory(
            &to_png(&from_png(&to_png(&dib.build()).expect("png")).expect("dib")).expect("png"),
        )
        .expect("relee")
        .to_rgba8();
        assert_eq!(once.as_raw(), twice.as_raw());
    }

    #[test]
    fn what_is_not_a_png_is_not_a_bitmap_either() {
        assert_eq!(from_png(&[]), None);
        assert_eq!(from_png(b"esto no es un png"), None);
        assert_eq!(from_png(&vec![0u8; LARGEST_BITMAP + 1]), None);
    }

    #[test]
    fn a_png_from_a_dib_is_a_png() {
        let dib = Dib::rgb32(4, 4).build();
        let png = to_png(&dib).expect("png");
        assert_eq!(&png[1..4], b"PNG");
        let back = image::load_from_memory(&png).expect("se relee");
        assert_eq!((back.width(), back.height()), (4, 4));
    }

    #[test]
    fn a_capture_with_an_unwritten_alpha_does_not_become_invisible() {
        let mut dib = Dib::rgb32(4, 4);
        dib.pixels = [0x20, 0x60, 0xA0, 0x00].repeat(16);
        let png = to_png(&dib.build()).expect("png");
        let back = image::load_from_memory(&png).expect("se relee").to_rgba8();
        assert!(
            back.pixels().all(|pixel| pixel[3] == 0xFF),
            "la imagen tiene que verse"
        );
    }

    #[test]
    fn a_declared_alpha_channel_left_at_zero_is_not_an_invisible_capture() {
        let mut dib = Dib::rgb32(4, 4);
        dib.header_size = 124;
        dib.compression = BI_BITFIELDS;
        dib.pixels = [0x20, 0x60, 0xA0, 0x00].repeat(16);
        let mut raw = dib.build();
        raw[40..44].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
        raw[44..48].copy_from_slice(&0x0000_FF00u32.to_le_bytes());
        raw[48..52].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(
            alpha(&raw),
            Alpha::Opaque,
            "todo a cero no es transparencia"
        );
        let png = to_png(&raw).expect("png");
        let back = image::load_from_memory(&png).expect("se relee").to_rgba8();
        assert!(
            back.pixels().all(|pixel| pixel[3] == 0xFF),
            "la captura tiene que verse"
        );
    }

    #[test]
    fn real_transparency_survives_the_trip() {
        let mut dib = Dib::rgb32(2, 2);
        dib.compression = BI_BITFIELDS;
        dib.header_size = 124;
        dib.pixels = vec![
            0x20, 0x60, 0xA0, 0x00, //
            0x20, 0x60, 0xA0, 0x80, //
            0x20, 0x60, 0xA0, 0xC0, //
            0x20, 0x60, 0xA0, 0xFF,
        ];
        let mut raw = dib.build();
        raw[40..44].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
        raw[44..48].copy_from_slice(&0x0000_FF00u32.to_le_bytes());
        raw[48..52].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
        raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
        assert_eq!(alpha(&raw), Alpha::Real);
        let png = to_png(&raw).expect("png");
        let back = image::load_from_memory(&png).expect("se relee").to_rgba8();
        let seen: Vec<u8> = back.pixels().map(|pixel| pixel[3]).collect();
        assert!(
            seen.contains(&0x00) && seen.contains(&0xFF),
            "los cuatro niveles de alfa llegaron como {seen:?}"
        );
    }

    #[test]
    fn a_truncated_bitmap_is_refused_not_panicked_on() {
        let dib = Dib::rgb32(100, 100).build();
        for cut in [0, 1, 20, 39, 40, 41, 100, 500] {
            let short = &dib[..cut.min(dib.len())];
            let _ = header(short);
            let _ = pixel_offset(short);
            let _ = alpha(short);
            let _ = as_bmp(short);
            let _ = to_png(short);
        }
    }
}

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn no_pile_of_bytes_can_bring_the_process_down(
            bytes in prop::collection::vec(any::<u8>(), 0..600),
        ) {
            let _ = header(&bytes);
            let _ = pixel_offset(&bytes);
            let _ = alpha(&bytes);
            let _ = as_bmp(&bytes);
            let _ = to_png(&bytes);
        }

        #[test]
        fn the_pixels_always_start_inside_the_buffer(
            bytes in prop::collection::vec(any::<u8>(), 0..600),
        ) {
            if let Some(at) = pixel_offset(&bytes) {
                prop_assert!(at <= bytes.len());
                prop_assert!(at >= super::INFO_HEADER);
            }
        }

        #[test]
        fn the_bmp_is_the_dib_with_a_header_in_front(
            bytes in prop::collection::vec(any::<u8>(), 0..600),
        ) {
            if let Some(bmp) = as_bmp(&bytes) {
                prop_assert_eq!(bmp.len(), bytes.len() + super::FILE_HEADER);
                prop_assert_eq!(&bmp[super::FILE_HEADER..], &bytes[..]);
            }
        }
    }
}
