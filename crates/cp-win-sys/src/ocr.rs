use windows::Graphics::Imaging::{BitmapDecoder, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};
use windows::core::RuntimeType;
use windows_future::AsyncStatus;
use windows_future::IAsyncOperation;

pub const PATIENCE: std::time::Duration = std::time::Duration::from_secs(5);

pub fn is_available() -> bool {
    OcrEngine::TryCreateFromUserProfileLanguages().is_ok()
}

pub fn text_in(image: &[u8]) -> Option<String> {
    let engine = OcrEngine::TryCreateFromUserProfileLanguages().ok()?;
    let bitmap = bitmap_of(image)?;
    let result = finished(engine.RecognizeAsync(&bitmap).ok()?)?;
    let text = result.Text().ok()?.to_string_lossy();
    (!text.trim().is_empty()).then_some(text)
}

fn finished<T: RuntimeType>(operation: IAsyncOperation<T>) -> Option<T> {
    let until = std::time::Instant::now() + PATIENCE;
    while operation.Status().ok()? == AsyncStatus::Started {
        if std::time::Instant::now() > until {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    operation.GetResults().ok()
}

fn bitmap_of(image: &[u8]) -> Option<SoftwareBitmap> {
    let stream = InMemoryRandomAccessStream::new().ok()?;
    let writer = DataWriter::CreateDataWriter(&stream.GetOutputStreamAt(0).ok()?).ok()?;
    writer.WriteBytes(image).ok()?;
    finished(writer.StoreAsync().ok()?)?;
    finished(writer.FlushAsync().ok()?)?;
    stream.Seek(0).ok()?;
    let decoder = finished(BitmapDecoder::CreateAsync(&stream).ok()?)?;
    finished(decoder.GetSoftwareBitmapAsync().ok()?)
}

#[cfg(test)]
mod tests {
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
}
