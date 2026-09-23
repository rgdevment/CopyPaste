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
    #[error("el archivo: {0}")]
    File(#[from] std::io::Error),
    #[error("la configuración no es TOML válido: {0}")]
    Malformed(#[from] toml::de::Error),
    #[error("la configuración no se pudo escribir: {0}")]
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
    pub wakes_with_session: bool,
    pub shortcut: String,
    pub hides_when_left: bool,
    #[serde(with = "count")]
    pub keeps_days: Option<u16>,
    #[serde(with = "count")]
    pub images_quota_mb: Option<u32>,
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
        let one = T::deserialize(reader)?;
        Ok((one != T::default()).then_some(one))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            locale: None,
            theme: Theme::default(),
            wakes_with_session: true,
            shortcut: SHORTCUT.to_owned(),
            hides_when_left: true,
            keeps_days: Some(KEEPS_DAYS),
            images_quota_mb: None,
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
        if self
            .locale
            .as_deref()
            .is_some_and(|one| one.trim().is_empty())
        {
            self.locale = None;
        }
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
    cfg!(windows) && matches!(why.raw_os_error(), Some(5) | Some(32))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("a temporary directory")
    }

    #[test]
    fn a_fresh_install_keeps_a_month_and_answers_to_a_shortcut() {
        let config = Config::default();
        assert_eq!(config.keeps_days, Some(30));
        assert_eq!(config.images_quota_mb, None);
        assert!(config.wakes_with_session);
        assert!(config.hides_when_left);
        assert_eq!(config.theme, Theme::System);
        assert_eq!(config.locale, None);
        assert!(config.shortcut.ends_with("Alt+V"), "{}", config.shortcut);
    }

    #[test]
    fn nothing_written_yet_reads_as_the_defaults() {
        let dir = a_dir();
        assert_eq!(read(&at(dir.path())).expect("reads"), Config::default());
    }

    #[test]
    fn what_goes_in_comes_back() {
        let dir = a_dir();
        let path = at(dir.path());
        let mine = Config {
            locale: Some("es".into()),
            theme: Theme::Dark,
            wakes_with_session: false,
            shortcut: "Ctrl+Shift+V".into(),
            hides_when_left: false,
            keeps_days: None,
            images_quota_mb: Some(512),
        };
        write(&path, &mine).expect("writes");
        assert_eq!(read(&path).expect("reads"), mine);
    }

    #[test]
    fn the_file_is_written_where_nothing_existed() {
        let dir = a_dir();
        let path = at(&dir.path().join("deeper"));
        write(&path, &Config::default()).expect("writes");
        assert!(path.exists());
        assert_eq!(leftovers(&dir), 0);
    }

    #[test]
    fn two_writers_at_once_leave_one_good_file_and_no_leftovers() {
        let dir = a_dir();
        let path = at(dir.path());
        std::thread::scope(|all| {
            for turn in 0..8 {
                let path = path.clone();
                all.spawn(move || {
                    let mine = Config {
                        keeps_days: Some(turn + 1),
                        ..Config::default()
                    };
                    write(&path, &mine).expect("writes");
                });
            }
        });
        let landed = read(&path).expect("reads");
        assert!(matches!(landed.keeps_days, Some(one) if (1..=8).contains(&one)));
        assert_eq!(leftovers(&dir), 0);
    }

    #[test]
    fn a_rename_that_is_refused_for_good_is_not_retried_forever() {
        let dir = a_dir();
        let path = at(dir.path());
        std::fs::create_dir(&path).expect("makes a directory");
        let why = write(&path, &Config::default()).expect_err("refuses");
        assert!(matches!(why, Error::File(_)), "{why}");
        assert_eq!(leftovers(&dir), 0);
    }

    fn leftovers(dir: &tempfile::TempDir) -> usize {
        std::fs::read_dir(dir.path())
            .expect("reads the directory")
            .filter_map(Result::ok)
            .filter(|one| one.file_name().to_string_lossy().ends_with(".new"))
            .count()
    }

    #[test]
    fn a_half_written_file_fills_the_rest_with_the_defaults() {
        let dir = a_dir();
        let path = at(dir.path());
        std::fs::write(&path, "theme = \"dark\"\n").expect("writes");
        let config = read(&path).expect("reads");
        assert_eq!(config.theme, Theme::Dark);
        assert_eq!(config.keeps_days, Config::default().keeps_days);
        assert_eq!(config.shortcut, Config::default().shortcut);
    }

    #[test]
    fn a_file_that_is_not_toml_says_so_instead_of_starting_over() {
        let dir = a_dir();
        let path = at(dir.path());
        std::fs::write(&path, "theme = = dark").expect("writes");
        let why = read(&path).expect_err("refuses");
        assert!(matches!(why, Error::Malformed(_)), "{why}");
        assert!(why.to_string().contains("TOML"), "{why}");
    }

    #[test]
    fn a_directory_where_the_file_should_be_is_not_mistaken_for_an_empty_one() {
        let dir = a_dir();
        let path = at(dir.path());
        std::fs::create_dir(&path).expect("makes a directory");
        assert!(matches!(read(&path), Err(Error::File(_))));
    }

    #[test]
    fn nothing_and_zero_mean_the_same_and_are_written_once() {
        let asked = Config {
            keeps_days: Some(0),
            images_quota_mb: Some(0),
            shortcut: "   ".into(),
            locale: Some("  ".into()),
            ..Config::default()
        }
        .sane();
        assert_eq!(asked.keeps_days, None);
        assert_eq!(asked.images_quota_mb, None);
        assert_eq!(asked.shortcut, SHORTCUT);
        assert_eq!(asked.locale, None);
    }

    #[test]
    fn what_is_read_is_sane_even_when_the_file_is_not() {
        let dir = a_dir();
        let path = at(dir.path());
        std::fs::write(&path, "keeps-days = 0\nimages-quota-mb = 0\n").expect("writes");
        let config = read(&path).expect("reads");
        assert_eq!(config.keeps_days, None);
        assert_eq!(config.images_quota_mb, None);
    }

    #[test]
    fn the_shortcut_loses_the_spaces_around_it() {
        let asked = Config {
            shortcut: " Ctrl+Shift+V ".into(),
            ..Config::default()
        }
        .sane();
        assert_eq!(asked.shortcut, "Ctrl+Shift+V");
    }

    #[test]
    fn the_file_reads_as_a_person_would_write_it() {
        let dir = a_dir();
        let path = at(dir.path());
        write(&path, &Config::default()).expect("writes");
        let said = std::fs::read_to_string(&path).expect("reads");
        assert!(said.contains("theme = \"system\""), "{said}");
        assert!(said.contains("wakes-with-session = true"), "{said}");
        assert!(said.contains("keeps-days = 30"), "{said}");
    }

    #[test]
    fn keeping_it_forever_survives_the_file() {
        let dir = a_dir();
        let path = at(dir.path());
        let mine = Config {
            keeps_days: None,
            ..Config::default()
        };
        write(&path, &mine).expect("writes");
        let said = std::fs::read_to_string(&path).expect("reads");
        assert!(said.contains("keeps-days = 0"), "{said}");
        assert_eq!(read(&path).expect("reads").keeps_days, None);
    }

    #[test]
    fn the_path_hangs_from_the_folder_it_is_given() {
        let where_it_is = at(Path::new("anywhere"));
        assert_eq!(where_it_is, Path::new("anywhere").join("config.toml"));
    }
}
