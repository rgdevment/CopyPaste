pub const MAX_SIDE: u32 = 256;

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
mod tests {
    use super::*;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let buffer = image::RgbaImage::from_fn(width, height, |x, y| {
            image::Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255])
        });
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(buffer)
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("png");
        out.into_inner()
    }

    #[test]
    fn the_size_is_read_without_decoding_the_whole_image() {
        let bytes = png(1920, 1080);
        assert_eq!(
            size_of(&bytes),
            Some(Size {
                width: 1920,
                height: 1080
            })
        );
    }

    #[test]
    fn a_thumbnail_keeps_the_proportions() {
        let bytes = png(1600, 400);
        let thumb = of_image(&bytes, MAX_SIDE).expect("miniatura");
        let size = size_of(&thumb).expect("tamaño");
        assert_eq!(size.width, MAX_SIDE);
        assert_eq!(size.height, MAX_SIDE / 4, "4:1 sigue siendo 4:1");
    }

    #[test]
    fn a_tall_image_is_bounded_by_its_height() {
        let bytes = png(300, 1200);
        let size = size_of(&of_image(&bytes, MAX_SIDE).expect("miniatura")).expect("tamaño");
        assert_eq!(size.height, MAX_SIDE);
        assert!(size.width < MAX_SIDE);
    }

    #[test]
    fn an_image_that_already_fits_is_not_enlarged() {
        for (width, height) in [(1, 1), (64, 64), (MAX_SIDE, MAX_SIDE), (100, 250)] {
            let bytes = png(width, height);
            let size = size_of(&of_image(&bytes, MAX_SIDE).expect("miniatura")).expect("tamaño");
            assert_eq!(
                size,
                Size { width, height },
                "una miniatura mayor que su original ocupa más y se ve peor"
            );
        }
    }

    #[test]
    fn a_thumbnail_weighs_much_less_than_the_original() {
        let bytes = png(2000, 2000);
        let thumb = of_image(&bytes, MAX_SIDE).expect("miniatura");
        assert!(
            thumb.len() * 10 < bytes.len(),
            "miniatura de {} bytes para un original de {}",
            thumb.len(),
            bytes.len()
        );
    }

    #[test]
    fn something_that_is_not_an_image_is_refused_not_guessed() {
        assert!(size_of(b"esto no es una imagen").is_none());
        assert!(of_image(b"esto no es una imagen", MAX_SIDE).is_none());
        assert!(size_of(&[]).is_none());
    }

    #[test]
    fn a_truncated_image_does_not_panic() {
        let bytes = png(500, 500);
        let half = &bytes[..bytes.len() / 2];
        assert!(of_image(half, MAX_SIDE).is_none());
    }

    #[test]
    fn a_wide_image_that_exceeds_only_the_width_is_still_scaled_down() {
        let bytes = png(100, 10);
        let size = size_of(&of_image(&bytes, 50).expect("miniatura")).expect("tamaño");
        assert_eq!(
            size,
            Size {
                width: 50,
                height: 5
            }
        );
    }

    #[test]
    fn a_tall_image_that_exceeds_only_the_height_is_still_scaled_down() {
        let bytes = png(10, 100);
        let size = size_of(&of_image(&bytes, 50).expect("miniatura")).expect("tamaño");
        assert_eq!(
            size,
            Size {
                width: 5,
                height: 50
            }
        );
    }

    #[test]
    fn scaling_to_the_size_it_already_has_changes_nothing() {
        use image::{DynamicImage, RgbaImage};
        let mut img = RgbaImage::new(64, 40);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = image::Rgba([(x * 3) as u8, (y * 7) as u8, 90, 255]);
        }
        let decoded = DynamicImage::ImageRgba8(img);
        let untouched = decoded.clone();
        let through_thumbnail = decoded.thumbnail(64, 64);
        assert_eq!(
            untouched.to_rgba8().into_raw(),
            through_thumbnail.to_rgba8().into_raw(),
            "con un lado igual al maximo, thumbnail no altera un solo pixel"
        );
    }
}
