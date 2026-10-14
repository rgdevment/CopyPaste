use super::*;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cp-bundle-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Applications")).expect("applications");
    std::fs::create_dir_all(dir.join("Trash")).expect("trash");
    dir
}

fn app(at: &Path, id: &str) -> PathBuf {
    std::fs::create_dir_all(at.join("Contents").join("MacOS")).expect("bundle");
    std::fs::write(
        at.join("Contents").join("Info.plist"),
        format!("<key>CFBundleIdentifier</key>\n<string>{id}</string>"),
    )
    .expect("plist");
    at.to_path_buf()
}

#[test]
fn the_bundle_is_three_folders_above_the_program() {
    let exe = Path::new("/Applications/CopyPaste.app/Contents/MacOS/cp-gui");
    assert_eq!(
        bundle_of(exe),
        Some(PathBuf::from("/Applications/CopyPaste.app"))
    );
    assert_eq!(bundle_of(Path::new("/usr/local/bin/cp-gui")), None);
}

#[test]
fn a_copy_updated_in_place_takes_the_new_name() {
    let dir = scratch("moved");
    let old = app(
        &dir.join("Applications").join("CopyPaste.app"),
        "com.rgdevment.copypaste",
    );
    let got = tidy(&old, &dir.join("Trash")).expect("tidy");
    let named = dir.join("Applications").join("CoPanda.app");
    assert_eq!(got, Tidy::Moved(named.clone()));
    assert!(named.join("Contents").join("Info.plist").exists());
    assert!(!old.exists());
}

#[test]
fn the_old_name_stays_put_when_the_new_one_is_already_there() {
    let dir = scratch("stay");
    let old = app(
        &dir.join("Applications").join("CopyPaste.app"),
        "com.rgdevment.copypaste",
    );
    app(
        &dir.join("Applications").join("CoPanda.app"),
        "com.rgdevment.copypaste",
    );
    assert_eq!(tidy(&old, &dir.join("Trash")).expect("tidy"), Tidy::Stay);
    assert!(old.exists());
}

#[test]
fn the_new_copy_sends_the_old_one_to_the_trash() {
    let dir = scratch("binned");
    let old = app(
        &dir.join("Applications").join("CopyPaste.app"),
        "com.rgdevment.copypaste",
    );
    let new = app(
        &dir.join("Applications").join("CoPanda.app"),
        "com.rgdevment.copypaste",
    );
    std::fs::create_dir_all(dir.join("Trash").join("CopyPaste.app")).expect("earlier");
    let got = tidy(&new, &dir.join("Trash")).expect("tidy");
    assert_eq!(got, Tidy::Binned(dir.join("Trash").join("CopyPaste 2.app")));
    assert!(!old.exists());
    assert!(new.exists());
}

#[test]
fn an_app_of_someone_else_under_the_old_name_is_left_alone() {
    let dir = scratch("theirs");
    let theirs = app(
        &dir.join("Applications").join("CopyPaste.app"),
        "com.example.copypaste",
    );
    let new = app(
        &dir.join("Applications").join("CoPanda.app"),
        "com.rgdevment.copypaste",
    );
    assert_eq!(tidy(&new, &dir.join("Trash")).expect("tidy"), Tidy::Stay);
    assert!(theirs.exists());
}
