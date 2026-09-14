use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Com::StructuredStorage::{PROPVARIANT, PropVariantToStringAlloc};
use windows::Win32::UI::Shell::PropertiesSystem::{
    GPS_DEFAULT, IPropertyStore, SHGetPropertyStoreFromParsingName,
};
use windows::core::{GUID, PCWSTR};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaInfo {
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

impl MediaInfo {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    pub fn searchable(&self) -> String {
        [&self.title, &self.artist, &self.album]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

const PKEY_MEDIA_DURATION: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x64440490_4c8b_11d1_8b70_080036b11a03),
    pid: 3,
};
const PKEY_VIDEO_WIDTH: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x64440491_4c8b_11d1_8b70_080036b11a03),
    pid: 3,
};
const PKEY_VIDEO_HEIGHT: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x64440491_4c8b_11d1_8b70_080036b11a03),
    pid: 4,
};
const PKEY_TITLE: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xf29f85e0_4ff9_1068_ab91_08002b27b3d9),
    pid: 2,
};
const PKEY_MUSIC_ARTIST: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x56a3372e_ce9c_11d2_9f0e_006097c686f6),
    pid: 13,
};
const PKEY_MUSIC_ALBUM: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x56a3372e_ce9c_11d2_9f0e_006097c686f6),
    pid: 4,
};

pub fn info_for(path: &std::path::Path) -> Option<MediaInfo> {
    if !path.exists() {
        return None;
    }
    let _apartment = crate::com::Apartment::enter();
    let absolute = crate::com::shell_path(path)?;
    let wide: Vec<u16> = absolute
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let store: IPropertyStore =
        unsafe { SHGetPropertyStoreFromParsingName(PCWSTR(wide.as_ptr()), None, GPS_DEFAULT) }
            .ok()?;

    let info = MediaInfo {
        duration: hundred_nanos(&store, PKEY_MEDIA_DURATION),
        width: number(&store, PKEY_VIDEO_WIDTH),
        height: number(&store, PKEY_VIDEO_HEIGHT),
        title: text(&store, PKEY_TITLE),
        artist: text(&store, PKEY_MUSIC_ARTIST),
        album: text(&store, PKEY_MUSIC_ALBUM),
    };

    (!info.is_empty()).then_some(info)
}

fn value_of(store: &IPropertyStore, key: PROPERTYKEY) -> Option<PROPVARIANT> {
    unsafe { store.GetValue(&key) }.ok()
}

fn hundred_nanos(store: &IPropertyStore, key: PROPERTYKEY) -> Option<f64> {
    let raw = number_u64(store, key)?;
    let seconds = raw as f64 / 10_000_000.0;
    (seconds.is_finite() && seconds > 0.0).then_some(seconds)
}

fn number_u64(store: &IPropertyStore, key: PROPERTYKEY) -> Option<u64> {
    let value = value_of(store, key)?;
    let raw = u64::try_from(&value).ok()?;
    (raw > 0).then_some(raw)
}

fn number(store: &IPropertyStore, key: PROPERTYKEY) -> Option<u32> {
    let value = value_of(store, key)?;
    let raw = u32::try_from(&value).ok()?;
    (raw > 0).then_some(raw)
}

fn text(store: &IPropertyStore, key: PROPERTYKEY) -> Option<String> {
    let value = value_of(store, key)?;

    let raw = unsafe { PropVariantToStringAlloc(&value) }.ok()?;
    if raw.is_null() {
        return None;
    }

    let text = unsafe { raw.to_string() }.ok();

    unsafe { CoTaskMemFree(Some(raw.as_ptr().cast())) };
    text.filter(|text| !text.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_is_not_there_has_no_metadata() {
        assert_eq!(
            info_for(std::path::Path::new(r"C:\no-existe-nada.mp3")),
            None
        );
    }

    #[test]
    fn a_file_without_media_metadata_says_nothing() {
        let dir = std::env::temp_dir().join("cp-media");
        std::fs::create_dir_all(&dir).expect("carpeta");
        let path = dir.join("texto.txt");
        std::fs::write(&path, b"no soy un video").expect("archivo");
        assert_eq!(info_for(&path), None);
    }

    #[test]
    fn nothing_known_is_nothing_to_search() {
        assert!(MediaInfo::default().is_empty());
        assert_eq!(MediaInfo::default().searchable(), "");
    }

    #[test]
    fn what_is_known_becomes_searchable() {
        let info = MediaInfo {
            title: Some("Canción".into()),
            artist: Some("Alguien".into()),
            album: None,
            ..Default::default()
        };
        assert!(!info.is_empty());
        assert_eq!(info.searchable(), "Canción Alguien");
    }

    #[test]
    fn a_duration_alone_is_metadata_too() {
        let info = MediaInfo {
            duration: Some(12.5),
            ..Default::default()
        };
        assert!(!info.is_empty());
        assert_eq!(info.searchable(), "", "una duración no se busca por texto");
    }
}
