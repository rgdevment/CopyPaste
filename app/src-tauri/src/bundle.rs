use std::path::{Path, PathBuf};

const FORMER: &str = "CopyPaste.app";
const NAMED: &str = "CoPanda.app";
const OURS: &str = "<string>com.rgdevment.copypaste</string>";

#[derive(Debug, PartialEq, Eq)]
pub enum Tidy {
    Moved(PathBuf),
    Binned(PathBuf),
    Stay,
}

pub fn bundle_of(exe: &Path) -> Option<PathBuf> {
    let bundle = exe.parent()?.parent()?.parent()?;
    let named = bundle.extension().is_some_and(|one| one == "app");
    named.then(|| bundle.to_path_buf())
}

pub fn tidy(bundle: &Path, trash: &Path) -> std::io::Result<Tidy> {
    let name = bundle.file_name().and_then(|one| one.to_str());
    if name == Some(FORMER) {
        let named = bundle.with_file_name(NAMED);
        if named.exists() {
            return Ok(Tidy::Stay);
        }
        std::fs::rename(bundle, &named)?;
        return Ok(Tidy::Moved(named));
    }
    if name == Some(NAMED) {
        let former = bundle.with_file_name(FORMER);
        if !ours(&former) {
            return Ok(Tidy::Stay);
        }
        let binned = free_in(trash);
        std::fs::rename(&former, &binned)?;
        return Ok(Tidy::Binned(binned));
    }
    Ok(Tidy::Stay)
}

fn ours(bundle: &Path) -> bool {
    std::fs::read_to_string(bundle.join("Contents").join("Info.plist"))
        .is_ok_and(|plist| plist.contains(OURS))
}

fn free_in(trash: &Path) -> PathBuf {
    let first = trash.join(FORMER);
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| trash.join(format!("CopyPaste {n}.app")))
        .find(|one| !one.exists())
        .unwrap_or(first)
}

#[cfg(target_os = "macos")]
pub fn settle() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    if crate::update::mounted(Some(&exe)) {
        return false;
    }
    let Some(bundle) = bundle_of(&exe) else {
        return false;
    };
    let Some(home) = std::env::var_os("HOME") else {
        return false;
    };
    match tidy(&bundle, &Path::new(&home).join(".Trash")) {
        Ok(Tidy::Moved(named)) => {
            crate::note::note(&format!("the app now lives at {}", named.display()));
            reopen(&named)
        }
        Ok(Tidy::Binned(binned)) => {
            crate::note::note(&format!("the older copy went to {}", binned.display()));
            false
        }
        Ok(Tidy::Stay) => false,
        Err(why) => {
            crate::note::note(&format!("the app kept its folder name: {why}"));
            false
        }
    }
}

#[cfg(target_os = "macos")]
fn reopen(named: &Path) -> bool {
    std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("sleep 1; /usr/bin/open \"$0\"")
        .arg(named)
        .spawn()
        .is_ok()
}

#[cfg(test)]
#[path = "bundle_test.rs"]
mod tests;
