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

#[must_use]
pub fn present(
    among: &'static [Font],
    dir: &Path,
    exists: &dyn Fn(&Path) -> bool,
) -> Vec<&'static Font> {
    among
        .iter()
        .filter(|one| exists(&dir.join(one.file)))
        .collect()
}

#[must_use]
pub fn family_of(
    asked: Option<&str>,
    fallback: Font,
    among: &'static [Font],
    dir: &Path,
    exists: &dyn Fn(&Path) -> bool,
) -> &'static str {
    let Some(asked) = asked else {
        return fallback.family;
    };
    present(among, dir, exists)
        .into_iter()
        .find(|one| one.id == asked)
        .map_or(fallback.family, |one| one.family)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Offered {
    pub id: &'static str,
    pub label: &'static str,
    pub css: &'static str,
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
pub struct Choices {
    pub text: Offered,
    pub texts: Vec<Offered>,
    pub code: Offered,
    pub codes: Vec<Offered>,
    pub accents: Vec<Paint>,
}

fn offered(one: &Font) -> Offered {
    Offered {
        id: one.id,
        label: one.label,
        css: one.css,
    }
}

#[must_use]
pub fn hex(rgb: u32) -> String {
    format!("#{:06X}", rgb & 0x00FF_FFFF)
}

#[must_use]
pub fn choices(fonts: &Fonts, dir: &Path, exists: &dyn Fn(&Path) -> bool) -> Choices {
    Choices {
        text: offered(&fonts.text),
        texts: present(fonts.texts, dir, exists)
            .into_iter()
            .map(offered)
            .collect(),
        code: offered(&fonts.code),
        codes: present(fonts.codes, dir, exists)
            .into_iter()
            .map(offered)
            .collect(),
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

#[must_use]
pub fn choices_here() -> Choices {
    choices(&fonts_here(), &fonts_dir_here(), &|path: &Path| {
        path.exists()
    })
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
