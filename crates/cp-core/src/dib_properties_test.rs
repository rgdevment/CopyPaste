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
