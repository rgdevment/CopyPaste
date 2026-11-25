use objc2_core_foundation::{CFArray, CFDictionary, CFRetained, CFString};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFontManagerCopyAvailableFontFamilyNames, CTFontStylisticClass,
    CTFontSymbolicTraits, kCTFontFamilyNameAttribute, kCTFontFamilyNameKey,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family {
    pub name: String,
    pub local: String,
    pub mono: bool,
    pub symbol: bool,
}

pub fn families() -> Vec<Family> {
    let names = unsafe { CTFontManagerCopyAvailableFontFamilyNames() };
    let names = unsafe { CFRetained::cast_unchecked::<CFArray<CFString>>(names) };
    names.iter().map(|name| family_of(&name)).collect()
}

fn family_of(name: &CFString) -> Family {
    let key = unsafe { kCTFontFamilyNameAttribute };
    let asked = CFDictionary::<CFString, CFString>::from_slices(&[key], &[name]);
    let descriptor = unsafe { CTFontDescriptor::with_attributes(asked.as_ref()) };
    let font = unsafe { CTFont::with_font_descriptor(&descriptor, 12.0, std::ptr::null()) };
    let traits = unsafe { font.symbolic_traits() };
    let class = traits.0 & CTFontSymbolicTraits::TraitClassMask.0;
    let family_key = unsafe { kCTFontFamilyNameKey };
    let local = unsafe { font.localized_name(family_key, std::ptr::null_mut()) };
    let name = name.to_string();
    Family {
        local: local.map_or_else(|| name.clone(), |local| local.to_string()),
        name,
        mono: traits.contains(CTFontSymbolicTraits::TraitMonoSpace),
        symbol: class == CTFontStylisticClass::ClassSymbolic.0,
    }
}

#[cfg(test)]
#[path = "fonts_test.rs"]
mod tests;
