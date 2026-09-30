use windows::Win32::System::DataExchange::GetClipboardFormatNameW;

pub const CF_TEXT: u32 = 1;
pub const CF_BITMAP: u32 = 2;
pub const CF_METAFILEPICT: u32 = 3;
pub const CF_SYLK: u32 = 4;
pub const CF_DIF: u32 = 5;
pub const CF_TIFF: u32 = 6;
pub const CF_OEMTEXT: u32 = 7;
pub const CF_DIB: u32 = 8;
pub const CF_PALETTE: u32 = 9;
pub const CF_UNICODETEXT: u32 = 13;
pub const CF_ENHMETAFILE: u32 = 14;
pub const CF_HDROP: u32 = 15;
pub const CF_LOCALE: u32 = 16;
pub const CF_DIBV5: u32 = 17;

const STANDARD: &[(u32, &str)] = &[
    (CF_TEXT, "CF_TEXT"),
    (CF_BITMAP, "CF_BITMAP"),
    (CF_METAFILEPICT, "CF_METAFILEPICT"),
    (CF_SYLK, "CF_SYLK"),
    (CF_DIF, "CF_DIF"),
    (CF_TIFF, "CF_TIFF"),
    (CF_OEMTEXT, "CF_OEMTEXT"),
    (CF_DIB, "CF_DIB"),
    (CF_PALETTE, "CF_PALETTE"),
    (CF_UNICODETEXT, "CF_UNICODETEXT"),
    (CF_ENHMETAFILE, "CF_ENHMETAFILE"),
    (CF_HDROP, "CF_HDROP"),
    (CF_LOCALE, "CF_LOCALE"),
    (CF_DIBV5, "CF_DIBV5"),
];

pub fn standard_name(id: u32) -> Option<&'static str> {
    STANDARD
        .iter()
        .find(|(known, _)| *known == id)
        .map(|(_, name)| *name)
}

pub fn standard_id(name: &str) -> Option<u32> {
    STANDARD
        .iter()
        .find(|(_, known)| *known == name)
        .map(|(id, _)| *id)
}

pub fn id_of(name: &str) -> Option<u32> {
    standard_id(name).or_else(|| crate::clipboard::register(name))
}

pub fn name_of(id: u32) -> String {
    if let Some(known) = standard_name(id) {
        return known.to_owned();
    }
    let mut buffer = [0u16; 256];

    let written = unsafe { GetClipboardFormatNameW(id, &mut buffer) };
    if written > 0 {
        String::from_utf16_lossy(&buffer[..written as usize])
    } else {
        format!("#{id}")
    }
}

#[cfg(test)]
#[path = "formats_test.rs"]
mod tests;
