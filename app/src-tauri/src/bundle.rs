use std::path::{Path, PathBuf};

const FORMER: &str = "CopyPaste.app";
const NAMED: &str = "CoPanda.app";
const OURS: &str = "<string>com.rgdevment.copypaste</string>";

#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    Rename { from: PathBuf, to: PathBuf },
    Bin(PathBuf),
    Stay,
}

pub fn bundle_of(exe: &Path) -> Option<PathBuf> {
    let bundle = exe.parent()?.parent()?.parent()?;
    let named = bundle.extension().is_some_and(|one| one == "app");
    named.then(|| bundle.to_path_buf())
}

pub fn plan(bundle: &Path) -> Plan {
    let name = bundle.file_name().and_then(|one| one.to_str());
    if name == Some(FORMER) {
        let to = bundle.with_file_name(NAMED);
        if to.exists() {
            return Plan::Stay;
        }
        return Plan::Rename {
            from: bundle.to_path_buf(),
            to,
        };
    }
    if name == Some(NAMED) {
        let former = bundle.with_file_name(FORMER);
        if ours(&former) {
            return Plan::Bin(former);
        }
    }
    Plan::Stay
}

fn ours(bundle: &Path) -> bool {
    std::fs::read_to_string(bundle.join("Contents").join("Info.plist"))
        .is_ok_and(|plist| plist.contains(OURS))
}

pub fn brewed(roots: &[&Path]) -> bool {
    roots.iter().any(|root| {
        ["copypaste", "copypaste-beta", "copanda", "copanda-beta"]
            .iter()
            .any(|cask| root.join(cask).exists())
    })
}

#[cfg(target_os = "macos")]
pub fn settle() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let Some(bundle) = bundle_of(&exe) else {
        return false;
    };
    let caskrooms = [
        Path::new("/opt/homebrew/Caskroom"),
        Path::new("/usr/local/Caskroom"),
    ];
    if crate::update::mounted(Some(&exe)) || brewed(&caskrooms) {
        return false;
    }
    match plan(&bundle) {
        Plan::Rename { from, to } => rename(&from, &to),
        Plan::Bin(former) => {
            bin(&former);
            false
        }
        Plan::Stay => false,
    }
}

#[cfg(target_os = "macos")]
fn rename(from: &Path, to: &Path) -> bool {
    let agent = std::env::var_os("HOME").map(|home| {
        Path::new(&home)
            .join("Library/LaunchAgents")
            .join("com.rgdevment.copypaste.plist")
    });
    if agent.is_some_and(|one| one.exists()) {
        return false;
    }
    if let Err(why) = std::fs::rename(from, to) {
        crate::note::note(&format!("the app could not take the name CoPanda: {why}"));
        return false;
    }
    if reopen(to) {
        crate::note::note(&format!("the app now lives at {}", to.display()));
        return true;
    }
    if let Err(why) = std::fs::rename(to, from) {
        crate::note::note(&format!("the app could not go back to its old name: {why}"));
    }
    false
}

#[cfg(target_os = "macos")]
fn bin(former: &Path) {
    if cp_mac_sys::files::running_from(former) {
        return;
    }
    match cp_mac_sys::files::trash(former) {
        Ok(()) => crate::note::note(&format!("{} went to the Trash", former.display())),
        Err(why) => crate::note::note(&format!("{} stayed: {why}", former.display())),
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
