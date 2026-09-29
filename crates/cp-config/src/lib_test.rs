use super::*;

fn a_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("a temporary directory")
}

#[test]
fn a_fresh_install_keeps_a_month_and_answers_to_a_shortcut() {
    let config = Config::default();
    assert_eq!(config.keeps_days, Some(30));
    assert_eq!(config.images_quota_mb, None);
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
    assert!(said.contains("keeps-days = 30"), "{said}");
}

#[test]
fn nothing_written_as_null_is_taken_for_nothing_at_all() {
    let asked: Config = serde_json::from_str(
        r#"{"locale":null,"theme":"system","shortcut":"Ctrl+Alt+V","hides-when-left":true,"keeps-days":null,"images-quota-mb":null}"#,
    )
    .expect("what the window sends gets in");
    assert_eq!(asked.keeps_days, None);
    assert_eq!(asked.images_quota_mb, None);
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
