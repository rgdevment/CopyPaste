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
fn an_unknown_key_is_ignored_not_rejected() {
    let dir = a_dir();
    let path = at(dir.path());
    std::fs::write(
        &path,
        "theme = \"dark\"\nthis-setting-does-not-exist = \"whatever\"\n",
    )
    .expect("writes");
    let config = read(&path).expect("an unknown key does not refuse the whole file");
    assert_eq!(config.theme, Theme::Dark);
}

#[test]
fn a_value_of_the_wrong_type_is_reported_as_malformed_not_guessed() {
    let dir = a_dir();
    let path = at(dir.path());
    std::fs::write(&path, "keeps-days = \"treinta\"\n").expect("writes");
    let why = read(&path).expect_err("a string is not a count of days");
    assert!(matches!(why, Error::Malformed(_)), "{why}");
}

#[test]
fn a_read_only_file_cannot_be_overwritten() {
    let dir = a_dir();
    let path = at(dir.path());
    write(&path, &Config::default()).expect("writes the first time");
    let mut permissions = std::fs::metadata(&path).expect("stat").permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&path, permissions).expect("marked read-only");

    let outcome = write(
        &path,
        &Config {
            theme: Theme::Dark,
            ..Config::default()
        },
    );
    assert!(
        outcome.is_err(),
        "a read-only settings file must not be silently replaced"
    );
    assert_eq!(
        read(&path).expect("still readable").theme,
        Theme::System,
        "the previous, still read-only content is the one that survives"
    );

    let mut permissions = std::fs::metadata(&path).expect("stat").permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(false);
    std::fs::set_permissions(&path, permissions).expect("cleared for cleanup");
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
fn a_shortcut_that_matches_no_real_key_combination_is_accepted_untouched() {
    let asked = Config {
        shortcut: "this is not a shortcut".into(),
        ..Config::default()
    }
    .sane();
    assert_eq!(
        asked.shortcut, "this is not a shortcut",
        "sane() only resets an empty or blank shortcut to the default; it never checks \
         that what is left looks like Ctrl/Alt/Shift plus a key, so garbage from a hand \
         edited config.toml reaches the rest of the app unchanged, and whether a given \
         combination is already taken by the OS or another app is never checked here at all"
    );
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

#[test]
fn an_existing_but_empty_file_reads_as_the_defaults() {
    let dir = a_dir();
    let path = at(dir.path());
    std::fs::write(&path, "").expect("writes");
    assert_eq!(read(&path).expect("reads"), Config::default());
}

#[test]
fn a_boolean_where_a_string_is_expected_is_rejected() {
    let dir = a_dir();
    let path = at(dir.path());
    std::fs::write(&path, "shortcut = true\n").expect("writes");
    assert!(matches!(read(&path), Err(Error::Malformed(_))));
}

#[test]
fn a_negative_count_is_a_parse_error_not_something_sane_clamps_to_zero() {
    let dir = a_dir();
    let path = at(dir.path());
    std::fs::write(&path, "keeps-days = -1\n").expect("writes");
    assert!(
        matches!(read(&path), Err(Error::Malformed(_))),
        "only an explicit 0 is normalised by sane(); a negative count never gets that far \
         because keeps-days is unsigned, so it fails to parse at all instead"
    );
}

#[test]
fn the_largest_representable_counts_round_trip() {
    let dir = a_dir();
    let path = at(dir.path());
    let mine = Config {
        keeps_days: Some(u16::MAX),
        images_quota_mb: Some(u32::MAX),
        ..Config::default()
    };
    write(&path, &mine).expect("writes");
    assert_eq!(read(&path).expect("reads"), mine);
}
