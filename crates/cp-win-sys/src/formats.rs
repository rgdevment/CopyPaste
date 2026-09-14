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
    // SAFETY: the buffer is live and its length is passed as declared.
    let written = unsafe { GetClipboardFormatNameW(id, &mut buffer) };
    if written > 0 {
        String::from_utf16_lossy(&buffer[..written as usize])
    } else {
        format!("#{id}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_standard_ids_are_the_ones_windows_documents() {
        for (id, name) in [
            (1u32, "CF_TEXT"),
            (8, "CF_DIB"),
            (13, "CF_UNICODETEXT"),
            (15, "CF_HDROP"),
            (17, "CF_DIBV5"),
        ] {
            assert_eq!(standard_name(id), Some(name));
        }
    }

    #[test]
    fn a_registered_id_is_not_a_standard_one() {
        assert_eq!(standard_name(49_161), None);
        assert_eq!(standard_name(0), None);
    }

    #[test]
    fn every_standard_id_maps_to_exactly_one_name() {
        for (id, _) in STANDARD {
            let matches = STANDARD.iter().filter(|(other, _)| other == id).count();
            assert_eq!(matches, 1, "el identificador {id} está dos veces");
        }
    }

    #[test]
    fn a_name_goes_back_to_the_id_it_came_from() {
        for id in [CF_TEXT, CF_DIB, CF_UNICODETEXT, CF_HDROP, CF_DIBV5] {
            assert_eq!(standard_id(&name_of(id)), Some(id));
        }
    }

    #[test]
    fn a_name_that_is_not_standard_has_no_standard_id() {
        assert_eq!(standard_id("Rich Text Format"), None);
        assert_eq!(standard_id(""), None);
    }

    #[test]
    fn a_registered_name_still_resolves_to_an_id() {
        let id = id_of("Rich Text Format").expect("se registra");
        assert!(id >= 0xC000, "los registrados viven por encima de 0xC000");
        assert_eq!(id_of("Rich Text Format"), Some(id), "y siempre el mismo");
        assert_eq!(name_of(id), "Rich Text Format");
    }

    #[test]
    fn an_unknown_id_still_gets_a_name() {
        assert_eq!(name_of(0), "#0");
    }
}
