use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextSize {
    Small,
    #[default]
    Normal,
    Large,
    Larger,
}

impl TextSize {
    #[must_use]
    pub fn zoom(self) -> f32 {
        match self {
            Self::Small => 0.92,
            Self::Normal => 1.0,
            Self::Large => 1.08,
            Self::Larger => 1.16,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Density {
    Compact,
    #[default]
    Normal,
    Comfortable,
}

impl Density {
    #[must_use]
    pub fn shut_lines(self) -> i32 {
        match self {
            Self::Compact => 1,
            Self::Normal => 2,
            Self::Comfortable => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Accent {
    #[default]
    Indigo,
    Blue,
    Teal,
    Green,
    Amber,
    Rose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swatch {
    pub accent: u32,
    pub dim: u32,
    pub selected: u32,
    pub edge: u32,
}

const fn swatch(accent: u32, dim: u32, selected: u32, edge: u32) -> Swatch {
    Swatch {
        accent,
        dim,
        selected,
        edge,
    }
}

impl Accent {
    pub const ALL: [Self; 6] = [
        Self::Indigo,
        Self::Blue,
        Self::Teal,
        Self::Green,
        Self::Amber,
        Self::Rose,
    ];

    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Indigo => "indigo",
            Self::Blue => "blue",
            Self::Teal => "teal",
            Self::Green => "green",
            Self::Amber => "amber",
            Self::Rose => "rose",
        }
    }

    #[must_use]
    pub fn swatch(self, light: bool) -> Swatch {
        if light {
            self.on_light()
        } else {
            self.on_dark()
        }
    }

    fn on_light(self) -> Swatch {
        match self {
            Self::Indigo => swatch(0x4F46E5, 0x6B63EA, 0xE9E9FB, 0xB4B1EC),
            Self::Blue => swatch(0x1D4ED8, 0x3B6FE0, 0xE3ECFC, 0xA9C1F2),
            Self::Teal => swatch(0x0F766E, 0x2A8C84, 0xDFF3F0, 0x9DD3CC),
            Self::Green => swatch(0x166534, 0x2F8F4E, 0xE2F3E7, 0xA4D6B3),
            Self::Amber => swatch(0x9A4A08, 0xC26A1E, 0xFBEEDD, 0xEDC596),
            Self::Rose => swatch(0xBE123C, 0xD0365A, 0xFBE4EA, 0xEDAFC0),
        }
    }

    fn on_dark(self) -> Swatch {
        match self {
            Self::Indigo => swatch(0xA5B4FC, 0x7C86C9, 0x282C46, 0x4A5085),
            Self::Blue => swatch(0x93C5FD, 0x6D94C6, 0x22304A, 0x3E5A85),
            Self::Teal => swatch(0x5EEAD4, 0x4AA89A, 0x1E3639, 0x33706A),
            Self::Green => swatch(0x86EFAC, 0x5FAE7C, 0x213A2E, 0x3C7653),
            Self::Amber => swatch(0xFCD34D, 0xC2A040, 0x3A3220, 0x7A6630),
            Self::Rose => swatch(0xFDA4AF, 0xC67C86, 0x3E2632, 0x7E4A57),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Font {
    pub id: &'static str,
    pub label: &'static str,
    pub family: &'static str,
    pub file: &'static str,
    pub css: &'static str,
}

const fn font(
    id: &'static str,
    label: &'static str,
    family: &'static str,
    file: &'static str,
    css: &'static str,
) -> Font {
    Font {
        id,
        label,
        family,
        file,
        css,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fonts {
    pub text: Font,
    pub texts: &'static [Font],
    pub code: Font,
    pub codes: &'static [Font],
}

pub const MAC_DIR: &str = "/System/Library/Fonts";

pub const MAC: Fonts = Fonts {
    text: font("", "SF Pro", "System Font", "SFNS.ttf", "-apple-system"),
    texts: &[
        font(
            "sf-pro-rounded",
            "SF Pro Rounded",
            ".SF NS Rounded",
            "SFNSRounded.ttf",
            "ui-rounded",
        ),
        font(
            "helvetica-neue",
            "Helvetica Neue",
            "Helvetica Neue",
            "HelveticaNeue.ttc",
            "\"Helvetica Neue\"",
        ),
        font(
            "avenir-next",
            "Avenir Next",
            "Avenir Next",
            "Avenir Next.ttc",
            "\"Avenir Next\"",
        ),
    ],
    code: font("", "Menlo", "Menlo", "Menlo.ttc", "Menlo"),
    codes: &[font(
        "sf-mono",
        "SF Mono",
        ".SF NS Mono",
        "SFNSMono.ttf",
        "ui-monospace",
    )],
};

pub const WINDOWS: Fonts = Fonts {
    text: font("", "Segoe UI", "Segoe UI", "segoeui.ttf", "\"Segoe UI\""),
    texts: &[
        font(
            "segoe-ui-variable",
            "Segoe UI Variable",
            "Segoe UI Variable Text",
            "SegUIVar.ttf",
            "\"Segoe UI Variable Text\"",
        ),
        font("calibri", "Calibri", "Calibri", "calibri.ttf", "Calibri"),
        font("arial", "Arial", "Arial", "arial.ttf", "Arial"),
    ],
    code: font("", "Consolas", "Consolas", "consola.ttf", "Consolas"),
    codes: &[font(
        "cascadia-mono",
        "Cascadia Mono",
        "Cascadia Mono",
        "CascadiaMono.ttf",
        "\"Cascadia Mono\"",
    )],
};

#[must_use]
pub fn fonts_here() -> Fonts {
    if cfg!(target_os = "macos") {
        MAC
    } else {
        WINDOWS
    }
}

#[must_use]
pub fn fonts_dir_here() -> std::path::PathBuf {
    if cfg!(target_os = "macos") {
        return Path::new(MAC_DIR).to_path_buf();
    }
    let windows = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".to_owned());
    Path::new(&windows).join("Fonts")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family {
    pub name: String,
    pub local: String,
    pub mono: bool,
    pub symbol: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Installed {
    pub families: Vec<Family>,
    pub system: Option<String>,
}

const SYMBOLS: [&str; 6] = [
    "Marlett",
    "Segoe MDL2 Assets",
    "Segoe Fluent Icons",
    "Webdings",
    "Wingdings",
    "Symbol",
];

#[must_use]
pub fn offerable(family: &Family) -> bool {
    let name = family.name.to_lowercase();
    !family.symbol
        && !name.starts_with('.')
        && !SYMBOLS.iter().any(|one| {
            let one = one.to_lowercase();
            name == one || name.starts_with(&format!("{one} "))
        })
}

fn same(one: &str, other: &str) -> bool {
    one.eq_ignore_ascii_case(other)
}

#[must_use]
pub fn installed<'a>(name: &str, families: &'a [Family]) -> Option<&'a Family> {
    families
        .iter()
        .find(|one| same(&one.name, name) || same(&one.local, name))
}

#[must_use]
pub fn substituted(face: &str, entries: &[(String, String)]) -> String {
    let bare = |said: &str| said.split(',').next().unwrap_or(said).trim().to_owned();
    entries
        .iter()
        .find(|(key, _)| same(key.trim(), face))
        .or_else(|| entries.iter().find(|(key, _)| same(&bare(key), face)))
        .map(|(_, value)| bare(value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| face.to_owned())
}

#[must_use]
pub fn css_of(family: &str) -> String {
    format!("\"{}\"", family.replace('\\', "\\\\").replace('"', "\\\""))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Offered {
    pub id: String,
    pub label: String,
    pub css: String,
    #[serde(skip)]
    pub family: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paint {
    pub id: &'static str,
    pub light: String,
    pub dark: String,
    pub light_selected: String,
    pub dark_selected: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choices {
    pub text: Offered,
    pub texts: Vec<Offered>,
    pub code: Offered,
    pub codes: Vec<Offered>,
    pub plain_system: bool,
    pub accents: Vec<Paint>,
}

impl Choices {
    #[must_use]
    pub fn text_family(&self, asked: Option<&str>) -> &str {
        &worn(asked, &self.text, &self.texts).family
    }

    #[must_use]
    pub fn code_family(&self, asked: Option<&str>) -> &str {
        &worn(asked, &self.code, &self.codes).family
    }
}

#[must_use]
pub fn worn<'a>(asked: Option<&str>, fallback: &'a Offered, among: &'a [Offered]) -> &'a Offered {
    asked
        .and_then(|asked| among.iter().find(|one| one.id == asked))
        .unwrap_or(fallback)
}

fn kept(one: &Font) -> Offered {
    Offered {
        id: one.id.to_owned(),
        label: one.label.to_owned(),
        css: one.css.to_owned(),
        family: one.family.to_owned(),
    }
}

fn found(one: &Family) -> Offered {
    Offered {
        id: one.name.clone(),
        label: one.local.clone(),
        css: css_of(&one.name),
        family: one.name.clone(),
    }
}

#[must_use]
pub fn hex(rgb: u32) -> String {
    format!("#{:06X}", rgb & 0x00FF_FFFF)
}

fn offered(
    fixed: &[Font],
    skipped: Option<&str>,
    families: &[Family],
    dir: &Path,
    exists: &dyn Fn(&Path) -> bool,
) -> Vec<Offered> {
    let fixed: Vec<&Font> = fixed
        .iter()
        .filter(|one| exists(&dir.join(one.file)) || installed(one.family, families).is_some())
        .collect();
    let taken = |name: &str| {
        skipped.is_some_and(|skipped| same(name, skipped))
            || fixed.iter().any(|one| same(name, one.family))
    };
    let mut all: Vec<Offered> = fixed
        .iter()
        .map(|one| kept(one))
        .chain(
            families
                .iter()
                .filter(|one| offerable(one) && !taken(&one.name))
                .map(found),
        )
        .collect();
    all.sort_by_key(|one| one.label.to_lowercase());
    all
}

fn system_text(fonts: &Fonts, installed_here: &Installed) -> (Offered, bool) {
    let system = installed_here
        .system
        .as_deref()
        .and_then(|name| installed(name, &installed_here.families))
        .filter(|one| offerable(one));
    match system {
        Some(one) if !same(&one.name, fonts.text.family) => (
            Offered {
                id: String::new(),
                ..found(one)
            },
            false,
        ),
        _ => (kept(&fonts.text), true),
    }
}

#[must_use]
pub fn choices(
    fonts: &Fonts,
    installed_here: &Installed,
    dir: &Path,
    exists: &dyn Fn(&Path) -> bool,
) -> Choices {
    let (text, plain_system) = system_text(fonts, installed_here);
    let monos: Vec<Family> = installed_here
        .families
        .iter()
        .filter(|one| one.mono)
        .cloned()
        .collect();
    Choices {
        text,
        texts: offered(fonts.texts, None, &installed_here.families, dir, exists),
        code: kept(&fonts.code),
        codes: offered(fonts.codes, Some(fonts.code.family), &monos, dir, exists),
        plain_system,
        accents: Accent::ALL
            .iter()
            .map(|one| Paint {
                id: one.key(),
                light: hex(one.swatch(true).accent),
                dark: hex(one.swatch(false).accent),
                light_selected: hex(one.swatch(true).selected),
                dark_selected: hex(one.swatch(false).selected),
            })
            .collect(),
    }
}

#[cfg(target_os = "windows")]
use cp_win_sys::fonts as system_fonts;

#[cfg(target_os = "macos")]
use cp_mac_sys::fonts as system_fonts;

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn families_here() -> Vec<Family> {
    system_fonts::families()
        .into_iter()
        .map(|one| Family {
            name: one.name,
            local: one.local,
            mono: one.mono,
            symbol: one.symbol,
        })
        .collect()
}

#[must_use]
pub fn installed_here() -> Installed {
    #[cfg(target_os = "windows")]
    {
        let system = system_fonts::message_face().map(|face| {
            let swapped = substituted(&face, &system_fonts::substitutes());
            system_fonts::family_of_gdi(&swapped).unwrap_or(swapped)
        });
        Installed {
            families: families_here(),
            system,
        }
    }
    #[cfg(target_os = "macos")]
    {
        Installed {
            families: families_here(),
            system: None,
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Installed::default()
    }
}

#[must_use]
pub fn choices_here() -> Choices {
    choices(
        &fonts_here(),
        &installed_here(),
        &fonts_dir_here(),
        &|path: &Path| path.exists(),
    )
}

pub mod lenient {
    use serde::de::DeserializeOwned;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error, StrDeserializer};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, T>(value: &T, writer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        value.serialize(writer)
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Said {
        Word(String),
        Other(serde::de::IgnoredAny),
    }

    pub fn deserialize<'de, D, T>(reader: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned + Default,
    {
        let Said::Word(said) = Said::deserialize(reader)? else {
            return Ok(T::default());
        };
        let plain: StrDeserializer<'_, Error> = said.as_str().into_deserializer();
        Ok(T::deserialize(plain).unwrap_or_default())
    }
}

#[cfg(test)]
#[path = "look_test.rs"]
mod tests;
