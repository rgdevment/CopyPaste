use super::*;

#[test]
fn the_system_offers_an_engine() {
    assert!(is_available(), "Windows trae OCR desde la 10");
}

#[test]
fn what_is_not_an_image_reads_as_nothing() {
    assert_eq!(text_in(&[]), None);
    assert_eq!(text_in(b"esto no es una imagen"), None);
}

#[test]
fn an_image_without_text_reads_as_nothing() {
    let blank = image::RgbaImage::from_pixel(64, 64, image::Rgba([255, 255, 255, 255]));
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(blank)
        .write_to(&mut png, image::ImageFormat::Png)
        .expect("png");
    assert_eq!(text_in(&png.into_inner()), None);
}

#[test]
fn the_patience_is_generous_but_finite() {
    assert!(PATIENCE >= std::time::Duration::from_secs(1));
    assert!(PATIENCE <= std::time::Duration::from_secs(30));
}
