use super::*;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cp-bundle-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
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
fn a_copy_updated_in_place_is_meant_to_take_the_new_name() {
    let dir = scratch("rename");
    let old = app(&dir.join("CopyPaste.app"), "com.rgdevment.copypaste");
    assert_eq!(
        plan(&old),
        Plan::Rename {
            from: old.clone(),
            to: dir.join("CoPanda.app")
        }
    );
}

#[test]
fn the_old_name_stays_put_when_the_new_one_is_already_there() {
    let dir = scratch("stay");
    let old = app(&dir.join("CopyPaste.app"), "com.rgdevment.copypaste");
    app(&dir.join("CoPanda.app"), "com.rgdevment.copypaste");
    assert_eq!(plan(&old), Plan::Stay);
}

#[test]
fn the_new_copy_means_to_bin_the_old_one() {
    let dir = scratch("bin");
    let old = app(&dir.join("CopyPaste.app"), "com.rgdevment.copypaste");
    let new = app(&dir.join("CoPanda.app"), "com.rgdevment.copypaste");
    assert_eq!(plan(&new), Plan::Bin(old));
}

#[test]
fn an_app_of_someone_else_under_the_old_name_is_left_alone() {
    let dir = scratch("theirs");
    app(&dir.join("CopyPaste.app"), "com.example.copypaste");
    let new = app(&dir.join("CoPanda.app"), "com.rgdevment.copypaste");
    assert_eq!(plan(&new), Plan::Stay);
}

#[test]
fn a_copy_homebrew_installed_is_left_to_homebrew() {
    let dir = scratch("brew");
    let empty = dir.join("empty");
    std::fs::create_dir_all(&empty).expect("empty");
    let brew = dir.join("Caskroom");
    std::fs::create_dir_all(brew.join("copypaste-beta")).expect("cask");
    assert!(!brewed(&[&empty]));
    assert!(brewed(&[&empty, &brew]));
}
