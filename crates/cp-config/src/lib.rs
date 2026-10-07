use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub mod look;

pub use look::{Accent, Density, TextSize};

pub const FILE: &str = "config.toml";

pub const SHORTCUT: &str = "Ctrl+Alt+V";

#[cfg(target_os = "macos")]
const FORMER_SHORTCUT: &str = "Cmd+Alt+V";

pub const KEEPS_DAYS: u16 = 30;

pub const IMAGES_QUOTA_MB: u32 = 5120;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the file: {0}")]
    File(#[from] std::io::Error),
    #[error("the settings are not valid TOML: {0}")]
    Malformed(#[from] toml::de::Error),
    #[error("the settings could not be written: {0}")]
    Unwritable(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub locale: Option<String>,
    pub theme: Theme,
    pub shortcut: String,
    pub hides_when_left: bool,
    #[serde(with = "count")]
    pub keeps_days: Option<u16>,
    #[serde(with = "count", default)]
    pub images_quota_mb: Option<u32>,
    pub welcomed: Option<String>,
    #[serde(with = "look::lenient")]
    pub text_size: TextSize,
    #[serde(with = "look::lenient")]
    pub density: Density,
    pub font: Option<String>,
    pub code_font: Option<String>,
    #[serde(with = "look::lenient")]
    pub accent: Accent,
}

mod count {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, T>(value: &Option<T>, writer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize + Default,
    {
        match value {
            Some(one) => one.serialize(writer),
            None => T::default().serialize(writer),
        }
    }

    pub fn deserialize<'de, D, T>(reader: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + Default + PartialEq,
    {
        let one = Option::<T>::deserialize(reader)?.unwrap_or_default();
        Ok((one != T::default()).then_some(one))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            locale: None,
            theme: Theme::default(),
            shortcut: SHORTCUT.to_owned(),
            hides_when_left: true,
            keeps_days: Some(KEEPS_DAYS),
            images_quota_mb: Some(IMAGES_QUOTA_MB),
            welcomed: None,
            text_size: TextSize::default(),
            density: Density::default(),
            font: None,
            code_font: None,
            accent: Accent::default(),
        }
    }
}

impl Config {
    #[must_use]
    pub fn sane(mut self) -> Self {
        if self.shortcut.trim().is_empty() {
            self.shortcut = SHORTCUT.to_owned();
        }
        self.shortcut = self.shortcut.trim().to_owned();
        #[cfg(target_os = "macos")]
        if self.shortcut == FORMER_SHORTCUT {
            self.shortcut = SHORTCUT.to_owned();
        }
        if self.keeps_days == Some(0) {
            self.keeps_days = None;
        }
        if self.images_quota_mb == Some(0) {
            self.images_quota_mb = None;
        }
        self.locale = trimmed(self.locale.take());
        self.welcomed = trimmed(self.welcomed.take());
        self.font = trimmed(self.font.take());
        self.code_font = trimmed(self.code_font.take());
        self
    }
}

fn trimmed(said: Option<String>) -> Option<String> {
    said.map(|one| one.trim().to_owned())
        .filter(|one| !one.is_empty())
}

#[must_use]
pub fn unlimited_for_the_former(config: &Config) -> Option<Config> {
    (config.images_quota_mb == Some(IMAGES_QUOTA_MB)).then(|| Config {
        images_quota_mb: None,
        ..config.clone()
    })
}

pub fn lift_for_the_former(path: &Path) -> Result<Option<Config>, Error> {
    let Some(lifted) = unlimited_for_the_former(&read(path)?) else {
        return Ok(None);
    };
    write(path, &lifted)?;
    Ok(Some(lifted))
}

#[must_use]
pub fn limited_again(config: &Config) -> Option<Config> {
    config.images_quota_mb.is_none().then(|| Config {
        images_quota_mb: Some(IMAGES_QUOTA_MB),
        ..config.clone()
    })
}

pub fn put_back_after_the_former(path: &Path) -> Result<Option<Config>, Error> {
    let Some(limited) = limited_again(&read(path)?) else {
        return Ok(None);
    };
    write(path, &limited)?;
    Ok(Some(limited))
}

#[must_use]
pub fn at(dir: &Path) -> PathBuf {
    dir.join(FILE)
}

pub fn read(path: &Path) -> Result<Config, Error> {
    let said = match std::fs::read_to_string(path) {
        Ok(said) => said,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(why) => return Err(why.into()),
    };
    Ok(toml::from_str::<Config>(&said)?.sane())
}

pub fn read_or_reset(path: &Path, at_ms: i64) -> Result<Config, Error> {
    match read(path) {
        Err(Error::Malformed(_)) => reset(path, at_ms),
        Err(Error::File(why)) if why.kind() == std::io::ErrorKind::InvalidData => {
            reset(path, at_ms)
        }
        other => other,
    }
}

fn reset(path: &Path, at_ms: i64) -> Result<Config, Error> {
    let kept = path.with_file_name(format!("config.broken-{at_ms}.toml"));
    std::fs::copy(path, kept)?;
    let fresh = Config {
        keeps_days: None,
        images_quota_mb: None,
        ..Config::default()
    };
    write(path, &fresh)?;
    Ok(fresh)
}

pub fn write(path: &Path, config: &Config) -> Result<(), Error> {
    let said = toml::to_string_pretty(&config.clone().sane())?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    static TURN: AtomicU64 = AtomicU64::new(0);
    let turn = TURN.fetch_add(1, Ordering::Relaxed);
    let meanwhile = path.with_extension(format!("toml.{}.{turn}.new", std::process::id()));
    let wrote = poured(&meanwhile, said.as_bytes()).and_then(|()| renamed(&meanwhile, path));
    if wrote.is_err() {
        let _ = std::fs::remove_file(&meanwhile);
    }
    Ok(wrote?)
}

fn poured(meanwhile: &Path, said: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(meanwhile)?;
    file.write_all(said)?;
    file.sync_all()
}

fn renamed(meanwhile: &Path, path: &Path) -> std::io::Result<()> {
    let mut wait = 10;
    for _ in 0..6 {
        match std::fs::rename(meanwhile, path) {
            Ok(()) => return Ok(()),
            Err(why) if !for_a_moment(&why) => return Err(why),
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(wait)),
        }
        wait *= 2;
    }
    std::fs::rename(meanwhile, path)
}

fn for_a_moment(why: &std::io::Error) -> bool {
    cfg!(windows) && matches!(why.raw_os_error(), Some(32) | Some(33))
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
