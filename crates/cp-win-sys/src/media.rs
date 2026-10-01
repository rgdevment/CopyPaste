use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Com::StructuredStorage::{PROPVARIANT, PropVariantToStringAlloc};
use windows::Win32::UI::Shell::PropertiesSystem::{
    GETPROPERTYSTOREFLAGS, GPS_DEFAULT, GPS_OPENSLOWITEM, IPropertyStore, PSGetNameFromPropertyKey,
    SHGetPropertyStoreFromParsingName,
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

fn store_for(path: &std::path::Path, how: GETPROPERTYSTOREFLAGS) -> Option<IPropertyStore> {
    if !path.exists() {
        return None;
    }
    let absolute = crate::com::shell_path(path)?;
    let wide: Vec<u16> = absolute
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe { SHGetPropertyStoreFromParsingName(PCWSTR(wide.as_ptr()), None, how) }.ok()
}

pub fn info_for(path: &std::path::Path) -> Option<MediaInfo> {
    let _apartment = crate::com::Apartment::enter();
    let store = store_for(path, GPS_DEFAULT)?;

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

pub struct Published {
    pub how_many: u32,
    pub said: Vec<(String, String)>,
}

pub fn everything_about(path: &std::path::Path) -> Option<Published> {
    let _apartment = crate::com::Apartment::enter();
    let store = store_for(path, GPS_OPENSLOWITEM)?;
    let how_many = unsafe { store.GetCount() }.unwrap_or(0);
    let mut said = Vec::new();
    for at in 0..how_many {
        let mut key = PROPERTYKEY::default();
        if unsafe { store.GetAt(at, &raw mut key) }.is_err() {
            continue;
        }
        let name = named(key).unwrap_or_else(|| format!("{:?}:{}", key.fmtid, key.pid));
        if let Some(value) = text(&store, key) {
            said.push((name, value));
        }
    }
    Some(Published { how_many, said })
}

fn named(key: PROPERTYKEY) -> Option<String> {
    let raw = unsafe { PSGetNameFromPropertyKey(&key) }.ok()?;
    if raw.is_null() {
        return None;
    }
    let name = unsafe { raw.to_string() }.ok();
    unsafe { CoTaskMemFree(Some(raw.as_ptr().cast())) };
    name
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
#[path = "media_test.rs"]
mod tests;
