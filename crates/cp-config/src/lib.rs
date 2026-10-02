use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const FILE: &str = "config.toml";

#[cfg(target_os = "macos")]
pub const SHORTCUT: &str = "Cmd+Alt+V";
#[cfg(not(target_os = "macos"))]
pub const SHORTCUT: &str = "Ctrl+Alt+V";

pub const KEEPS_DAYS: u16 = 30;

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
    #[serde(with = "count")]
    pub images_quota_mb: Option<u32>,
    pub welcomed: Option<String>,
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
            images_quota_mb: None,
            welcomed: None,
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
        if self.keeps_days == Some(0) {
            self.keeps_days = None;
        }
        if self.images_quota_mb == Some(0) {
            self.images_quota_mb = None;
        }
        self.locale = self
            .locale
            .take()
            .map(|one| one.trim().to_owned())
            .filter(|one| !one.is_empty());
        self.welcomed = self
            .welcomed
            .take()
            .map(|one| one.trim().to_owned())
            .filter(|one| !one.is_empty());
        self
    }
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
