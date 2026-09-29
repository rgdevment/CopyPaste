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
#[path = "ocr_test.rs"]
mod tests;
