use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::Com::{
    DVASPECT_CONTENT, FORMATETC, IDataObject, IStream, STGMEDIUM, STREAM_SEEK_SET, TYMED,
    TYMED_HGLOBAL, TYMED_ISTREAM,
};
use windows::Win32::System::Ole::{
    OleGetClipboard, OleInitialize, OleUninitialize, ReleaseStgMedium,
};

use crate::clipboard::global_bytes;

const CHUNK: usize = 64 * 1024;

pub struct Ole {
    ours: bool,
}

impl Ole {
    pub fn enter() -> Self {
        let entered = unsafe { OleInitialize(None) };
        Self {
            ours: entered.is_ok(),
        }
    }
}

impl Drop for Ole {
    fn drop(&mut self) {
        if self.ours {
            unsafe { OleUninitialize() };
        }
    }
}

pub fn indexed_contents(format: u32, count: usize, up_to: usize) -> Vec<Option<Vec<u8>>> {
    let _ole = Ole::enter();
    let Ok(data) = (unsafe { OleGetClipboard() }) else {
        return vec![None; count];
    };
    (0..count)
        .map(|index| {
            let indexed = i32::try_from(index).ok();
            indexed
                .and_then(|lindex| one_of(&data, format, lindex, up_to))
                .or_else(|| {
                    (count == 1)
                        .then(|| one_of(&data, format, -1, up_to))
                        .flatten()
                })
        })
        .collect()
}

fn one_of(data: &IDataObject, format: u32, lindex: i32, up_to: usize) -> Option<Vec<u8>> {
    let asked = FORMATETC {
        cfFormat: u16::try_from(format).ok()?,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex,
        tymed: (TYMED_HGLOBAL.0 | TYMED_ISTREAM.0) as u32,
    };

    let mut medium: STGMEDIUM = unsafe { data.GetData(&asked) }.ok()?;
    let bytes = match TYMED(medium.tymed as i32) {
        TYMED_HGLOBAL => {
            let global: HGLOBAL = unsafe { medium.u.hGlobal };
            global_bytes(global, up_to)
        }
        TYMED_ISTREAM => {
            let stream: Option<IStream> = unsafe { (*medium.u.pstm).clone() };
            stream.and_then(|stream| stream_bytes(&stream, up_to))
        }
        _ => None,
    };

    unsafe { ReleaseStgMedium(&mut medium) };
    bytes
}

fn stream_bytes(stream: &IStream, up_to: usize) -> Option<Vec<u8>> {
    let _ = unsafe { stream.Seek(0, STREAM_SEEK_SET, None) };
    let mut out = Vec::new();
    let mut chunk = vec![0u8; CHUNK];
    loop {
        let mut read = 0u32;

        let status =
            unsafe { stream.Read(chunk.as_mut_ptr().cast(), CHUNK as u32, Some(&mut read)) };
        if status.is_err() && read == 0 {
            return None;
        }
        if read == 0 {
            break;
        }
        out.extend_from_slice(&chunk[..read as usize]);
        if out.len() > up_to {
            return None;
        }
    }
    (!out.is_empty()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_twice_is_not_a_problem() {
        let first = Ole::enter();
        let second = Ole::enter();
        drop(second);
        drop(first);
    }

    #[test]
    fn asking_for_nothing_gets_nothing_and_does_not_touch_the_clipboard() {
        assert!(indexed_contents(0xC000, 0, CHUNK).is_empty());
    }

    #[test]
    fn a_format_nobody_offers_comes_back_absent_for_every_index() {
        let seen = indexed_contents(0xFFFE, 3, CHUNK);
        assert_eq!(seen, vec![None, None, None]);
    }
}
