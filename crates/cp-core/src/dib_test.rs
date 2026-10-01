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
            "header of {size} bytes"
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
        "without biClrUsed the whole palette is assumed"
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
        "what doesn't fit isn't subtracted"
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
        "measuring exactly what's there isn't measuring too much"
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
        "eight bytes of palette don't fit in five"
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
        "twelve of masks plus twelve of palette don't fit in twenty"
    );
}

#[test]
fn nonsense_is_not_a_bitmap() {
    assert_eq!(header(&[]), None);
    assert_eq!(header(&[0; 8]), None, "doesn't even reach the header");
    assert_eq!(
        header(&[0; 40]),
        None,
        "a header that claims to measure zero"
    );
    let mut lying = Dib::rgb32(2, 2).build();
    lying[0..4].copy_from_slice(&9999u32.to_le_bytes());
    assert_eq!(
        header(&lying),
        None,
        "claims to measure more than the whole buffer"
    );
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
        "the declared size is that of the whole file"
    );
    assert_eq!(
        u32::from_le_bytes([bmp[10], bmp[11], bmp[12], bmp[13]]) as usize,
        FILE_HEADER + INFO_HEADER
    );
    assert_eq!(&bmp[FILE_HEADER..], &dib[..], "the DIB travels intact");
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
        0x40, 0x80, 0xC0, 0xFF, 0x40, 0x80, 0xC0, 0x00, 0x40, 0x80, 0xC0, 0xFF, 0x40, 0x80, 0xC0,
        0xFF,
    ];
    assert_eq!(
        alpha(&dib.build()),
        Alpha::Real,
        "some at zero and others opaque is a cutout with edges"
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
    assert_eq!(pixel_offset(&raw), None, "the masks don't fit");
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
        "the fourth byte looks like alpha, but the header says it isn't"
    );
}

#[test]
fn a_synthesised_v5_does_not_become_half_transparent() {
    let mut dib = Dib::rgb32(4, 4);
    dib.header_size = 124;
    dib.pixels = [0x20, 0x60, 0xA0, 0x7F].repeat(16);
    let png = to_png(&dib.build()).expect("png");
    let back = image::load_from_memory(&png).expect("rereads").to_rgba8();
    assert!(
        back.pixels().all(|pixel| pixel[3] == 0xFF),
        "a transparency nobody declared isn't recorded"
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
    assert_eq!(pixel_offset(&raw), Some(56), "the masks go inside");
    assert_eq!(
        alpha(&raw),
        Alpha::Real,
        "it declares the alpha mask, so it has to be read"
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
        "four trailing bytes used to make the capture look transparent"
    );
    let back = image::load_from_memory(&to_png(&raw).expect("png"))
        .expect("rereads")
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
        "{} bytes of DIB against {} of PNG",
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
    let head = header(&back).expect("header");
    assert_eq!((head.width, head.height.abs()), (4, 4));
    assert!(pixel_offset(&back).is_some());
}

#[test]
fn the_round_trip_keeps_the_colours_where_they_were() {
    let mut dib = Dib::rgb32(2, 2);
    dib.pixels = vec![
        0x00, 0x00, 0xFF, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF,
    ];
    let once = image::load_from_memory(&to_png(&dib.build()).expect("png"))
        .expect("rereads")
        .to_rgba8();
    let twice = image::load_from_memory(
        &to_png(&from_png(&to_png(&dib.build()).expect("png")).expect("dib")).expect("png"),
    )
    .expect("rereads")
    .to_rgba8();
    assert_eq!(once.as_raw(), twice.as_raw());
}

#[test]
fn what_is_not_a_png_is_not_a_bitmap_either() {
    assert_eq!(from_png(&[]), None);
    assert_eq!(from_png(b"this is not a png"), None);
    assert_eq!(from_png(&vec![0u8; LARGEST_BITMAP + 1]), None);
}

#[test]
fn a_jpeg_becomes_a_bitmap_too_and_a_png_is_not_a_jpeg() {
    let dib = Dib::rgb32(4, 4).build();
    let png = to_png(&dib).expect("png");
    let mut jpeg = Vec::new();
    image::load_from_memory(&png)
        .expect("rereads")
        .to_rgb8()
        .write_to(
            &mut std::io::Cursor::new(&mut jpeg),
            image::ImageFormat::Jpeg,
        )
        .expect("jpeg");
    let back = from_jpeg(&jpeg).expect("dib");
    let head = header(&back).expect("header");
    assert_eq!((head.width, head.height.abs()), (4, 4));
    assert_eq!(
        head.bit_count, 24,
        "a JPEG carries no alpha and neither does the DIB"
    );
    assert_eq!(
        from_jpeg(&png),
        None,
        "a PNG doesn't sneak in through the JPEG door"
    );
    assert_eq!(from_jpeg(&[]), None);
    assert_eq!(from_jpeg(&vec![0u8; LARGEST_BITMAP + 1]), None);
}

#[test]
fn a_decoded_image_with_no_pixels_is_no_bitmap() {
    assert_eq!(from_image(&image::DynamicImage::new_rgb8(0, 0)), None);
    assert_eq!(from_image(&image::DynamicImage::new_rgba8(3, 0)), None);
    let opaque = from_image(&image::DynamicImage::new_rgb8(1, 1)).expect("dib");
    assert_eq!(header(&opaque).expect("header").bit_count, 24);
    let translucent = from_image(&image::DynamicImage::new_rgba8(1, 1)).expect("dib");
    assert_eq!(header(&translucent).expect("header").bit_count, 32);
}

#[test]
fn a_png_from_a_dib_is_a_png() {
    let dib = Dib::rgb32(4, 4).build();
    let png = to_png(&dib).expect("png");
    assert_eq!(&png[1..4], b"PNG");
    let back = image::load_from_memory(&png).expect("rereads");
    assert_eq!((back.width(), back.height()), (4, 4));
}

#[test]
fn a_capture_with_an_unwritten_alpha_does_not_become_invisible() {
    let mut dib = Dib::rgb32(4, 4);
    dib.pixels = [0x20, 0x60, 0xA0, 0x00].repeat(16);
    let png = to_png(&dib.build()).expect("png");
    let back = image::load_from_memory(&png).expect("rereads").to_rgba8();
    assert!(
        back.pixels().all(|pixel| pixel[3] == 0xFF),
        "the image has to be visible"
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
    assert_eq!(alpha(&raw), Alpha::Opaque, "all zero is not transparency");
    let png = to_png(&raw).expect("png");
    let back = image::load_from_memory(&png).expect("rereads").to_rgba8();
    assert!(
        back.pixels().all(|pixel| pixel[3] == 0xFF),
        "the capture has to be visible"
    );
}

#[test]
fn real_transparency_survives_the_trip() {
    let mut dib = Dib::rgb32(2, 2);
    dib.compression = BI_BITFIELDS;
    dib.header_size = 124;
    dib.pixels = vec![
        0x20, 0x60, 0xA0, 0x00, 0x20, 0x60, 0xA0, 0x80, 0x20, 0x60, 0xA0, 0xC0, 0x20, 0x60, 0xA0,
        0xFF,
    ];
    let mut raw = dib.build();
    raw[40..44].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
    raw[44..48].copy_from_slice(&0x0000_FF00u32.to_le_bytes());
    raw[48..52].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
    raw[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes());
    assert_eq!(alpha(&raw), Alpha::Real);
    let png = to_png(&raw).expect("png");
    let back = image::load_from_memory(&png).expect("rereads").to_rgba8();
    let seen: Vec<u8> = back.pixels().map(|pixel| pixel[3]).collect();
    assert!(
        seen.contains(&0x00) && seen.contains(&0xFF),
        "the four alpha levels arrived as {seen:?}"
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

fn negative_width_rgb32(width: i32, height: i32) -> Vec<u8> {
    let pixels = [0x40u8, 0x80, 0xC0, 0xFF]
        .repeat(width.unsigned_abs() as usize * height.unsigned_abs() as usize);
    let mut out = Vec::new();
    out.extend_from_slice(&(INFO_HEADER as u32).to_le_bytes());
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes());
    out.extend_from_slice(&2835i32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.resize(INFO_HEADER, 0);
    out.extend_from_slice(&pixels);
    out
}

#[test]
fn a_negative_width_is_read_by_alpha_but_refused_by_to_png() {
    let dib = negative_width_rgb32(-4, 4);
    assert_eq!(header(&dib).map(|head| head.width), Some(-4));
    assert_ne!(
        alpha(&dib),
        Alpha::Absent,
        "a negative width is accepted via its magnitude and the pixels are read anyway"
    );
    assert_eq!(
        to_png(&dib),
        None,
        "the same negative width is refused once it reaches the BMP decoder, so alpha() \
         and to_png() disagree about whether this bitmap is usable"
    );
}

#[test]
fn a_zero_width_or_height_is_refused_by_to_png() {
    for dib in [negative_width_rgb32(0, 4), negative_width_rgb32(4, 0)] {
        assert_eq!(to_png(&dib), None);
    }
}
