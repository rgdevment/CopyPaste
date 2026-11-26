use windows::Win32::Globalization::GetUserDefaultLocaleName;
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_REGULAR, DWriteCreateFactory, IDWriteFactory, IDWriteFont, IDWriteFont1,
    IDWriteFontCollection, IDWriteFontFamily, IDWriteLocalizedStrings,
};
use windows::Win32::Graphics::Gdi::LOGFONTW;
use windows::Win32::UI::WindowsAndMessaging::{
    NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    SystemParametersInfoW,
};
use windows::core::{BOOL, HSTRING, Interface};
use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

const SUBSTITUTES: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes";
const ENGLISH: &str = "en-us";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family {
    pub name: String,
    pub local: String,
    pub mono: bool,
    pub symbol: bool,
}

pub fn families() -> Vec<Family> {
    let Some(collection) = system_collection() else {
        return Vec::new();
    };
    let user = user_locale();
    let count = unsafe { collection.GetFontFamilyCount() };
    (0..count)
        .filter_map(|index| unsafe { collection.GetFontFamily(index) }.ok())
        .filter_map(|family| family_of(&family, user.as_deref()))
        .collect()
}

pub fn message_face() -> Option<String> {
    let mut metrics = NONCLIENTMETRICSW {
        cbSize: std::mem::size_of::<NONCLIENTMETRICSW>() as u32,
        ..Default::default()
    };
    unsafe {
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            Some((&raw mut metrics).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .ok()?;
    let face = wide_until_nul(&metrics.lfMessageFont.lfFaceName);
    (!face.is_empty()).then_some(face)
}

pub fn substitutes() -> Vec<(String, String)> {
    let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(SUBSTITUTES) else {
        return Vec::new();
    };
    key.enum_values()
        .filter_map(Result::ok)
        .map(|(name, value)| (name, value.to_string()))
        .collect()
}

pub fn family_of_gdi(face: &str) -> Option<String> {
    let mut logfont = LOGFONTW::default();
    let wide: Vec<u16> = face.encode_utf16().collect();
    if wide.len() >= logfont.lfFaceName.len() {
        return None;
    }
    logfont.lfFaceName[..wide.len()].copy_from_slice(&wide);
    let factory = factory()?;
    let interop = unsafe { factory.GetGdiInterop() }.ok()?;
    let font = unsafe { interop.CreateFontFromLOGFONT(&raw const logfont) }.ok()?;
    let family = unsafe { font.GetFontFamily() }.ok()?;
    let names = unsafe { family.GetFamilyNames() }.ok()?;
    name_in(&names, ENGLISH).or_else(|| name_at(&names, 0))
}

fn factory() -> Option<IDWriteFactory> {
    unsafe { DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED) }.ok()
}

fn system_collection() -> Option<IDWriteFontCollection> {
    let factory = factory()?;
    let mut collection = None;
    unsafe { factory.GetSystemFontCollection(&raw mut collection, true) }.ok()?;
    collection
}

fn family_of(family: &IDWriteFontFamily, user: Option<&str>) -> Option<Family> {
    let names = unsafe { family.GetFamilyNames() }.ok()?;
    let name = name_in(&names, ENGLISH).or_else(|| name_at(&names, 0))?;
    let local = user
        .and_then(|locale| name_in(&names, locale))
        .unwrap_or_else(|| name.clone());
    let font = unsafe {
        family.GetFirstMatchingFont(
            DWRITE_FONT_WEIGHT_REGULAR,
            DWRITE_FONT_STRETCH_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
        )
    }
    .ok()?;
    Some(Family {
        name,
        local,
        mono: is_mono(&font),
        symbol: unsafe { font.IsSymbolFont() }.as_bool(),
    })
}

fn is_mono(font: &IDWriteFont) -> bool {
    font.cast::<IDWriteFont1>()
        .is_ok_and(|font| unsafe { font.IsMonospacedFont() }.as_bool())
}

fn name_in(names: &IDWriteLocalizedStrings, locale: &str) -> Option<String> {
    let (mut index, mut exists) = (0u32, BOOL(0));
    let locale = HSTRING::from(locale);
    unsafe { names.FindLocaleName(&locale, &raw mut index, &raw mut exists) }.ok()?;
    if !exists.as_bool() {
        return None;
    }
    name_at(names, index)
}

fn name_at(names: &IDWriteLocalizedStrings, index: u32) -> Option<String> {
    let length = unsafe { names.GetStringLength(index) }.ok()?;
    let mut buffer = vec![0u16; length as usize + 1];
    unsafe { names.GetString(index, &mut buffer) }.ok()?;
    let said = wide_until_nul(&buffer);
    (!said.is_empty()).then_some(said)
}

fn user_locale() -> Option<String> {
    let mut buffer = [0u16; 85];
    let written = unsafe { GetUserDefaultLocaleName(&mut buffer) };
    (written > 0).then(|| wide_until_nul(&buffer))
}

fn wide_until_nul(wide: &[u16]) -> String {
    let end = wide.iter().position(|&one| one == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

#[cfg(test)]
#[path = "fonts_test.rs"]
mod tests;
