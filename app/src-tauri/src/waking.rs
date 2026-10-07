#[derive(serde::Serialize, PartialEq, Eq, Debug)]
pub struct Waking {
    pub offered: bool,
    pub wakes: bool,
    pub theirs: bool,
    pub managed: bool,
}

impl Waking {
    #[cfg(not(target_os = "macos"))]
    fn none() -> Self {
        Self {
            offered: false,
            wakes: false,
            theirs: false,
            managed: false,
        }
    }
}

#[tauri::command]
pub fn waking() -> Waking {
    there::waking()
}

#[tauri::command]
pub fn wake(wanted: bool) -> Result<Waking, String> {
    there::wake(wanted).map_err(|why| why.to_string())?;
    Ok(there::waking())
}

#[cfg(windows)]
mod there {
    use super::Waking;
    use std::path::Path;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};

    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APPROVED: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    const NAME: &str = "CopyPaste";

    pub fn waking() -> Waking {
        let Ok(exe) = std::env::current_exe() else {
            return Waking::none();
        };
        if packaged(&exe) {
            return Waking {
                offered: true,
                wakes: false,
                theirs: false,
                managed: true,
            };
        }
        let ours = written().is_some_and(|said| ours(&said, &exe));
        let approved = approved();
        Waking {
            offered: true,
            wakes: ours && approved,
            theirs: ours && !approved,
            managed: false,
        }
    }

    pub fn wake(wanted: bool) -> std::io::Result<()> {
        let exe = std::env::current_exe()?;
        if packaged(&exe) {
            return Err(std::io::Error::other(
                "Windows manages the startup of the installed app in Settings > Apps > Startup",
            ));
        }
        if written().is_some_and(|said| !ours(&said, &exe)) {
            return Err(std::io::Error::other(
                "another program holds the startup entry under CopyPaste's name",
            ));
        }
        let key =
            winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN, KEY_WRITE)?;
        if !wanted {
            return match key.delete_value(NAME) {
                Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
                other => other,
            };
        }
        key.set_value(NAME, &format!("\"{}\"", exe.display()))?;
        approve()
    }

    fn approve() -> std::io::Result<()> {
        let held =
            winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(APPROVED, KEY_WRITE);
        let Ok(key) = held else {
            return Ok(());
        };
        match key.delete_value(NAME) {
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }

    fn written() -> Option<String> {
        winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(RUN)
            .ok()?
            .get_value::<String, _>(NAME)
            .ok()
    }

    fn approved() -> bool {
        let read = winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(APPROVED)
            .ok()
            .and_then(|key| key.get_raw_value(NAME).ok());
        read.is_none_or(|held| approves(&held.bytes))
    }

    pub fn packaged(exe: &Path) -> bool {
        crate::update::chosen(Some(exe)) == crate::update::Route::Store
    }

    pub fn approves(bytes: &[u8]) -> bool {
        !matches!(bytes.first(), Some(3 | 6))
    }

    pub fn ours(said: &str, exe: &Path) -> bool {
        let said = said.trim();
        let named = exe.display().to_string();
        let Some(rest) = said.strip_prefix('"') else {
            return said.eq_ignore_ascii_case(&named);
        };
        let Some((quoted, after)) = rest.split_once('"') else {
            return false;
        };
        !quoted.is_empty() && after.trim().is_empty() && quoted.eq_ignore_ascii_case(&named)
    }
}

#[cfg(target_os = "macos")]
mod there {
    use super::Waking;
    use cp_mac_sys::login::{self, LoginState};
    use std::path::{Path, PathBuf};

    const LABEL: &str = "com.rgdevment.copypaste";

    pub fn waking() -> Waking {
        migrate();
        let state = login::state();
        Waking {
            offered: state != LoginState::Missing,
            wakes: matches!(state, LoginState::On | LoginState::NeedsApproval),
            theirs: state == LoginState::NeedsApproval,
            managed: false,
        }
    }

    pub fn wake(wanted: bool) -> std::io::Result<()> {
        let exe = std::env::current_exe()?;
        if wanted && crate::update::mounted(Some(&exe)) {
            return Err(std::io::Error::other(
                "move CopyPaste to the Applications folder before starting it with the session",
            ));
        }
        if !wanted && let Some(old) = legacy() {
            retire(&old)?;
        }
        let state = login::state();
        let settled = if wanted {
            state == LoginState::On
        } else {
            matches!(state, LoginState::Off | LoginState::Missing)
        };
        if !settled {
            login::set(wanted).map_err(std::io::Error::other)?;
        }
        if wanted
            && login::state() == LoginState::On
            && let Some(old) = legacy()
        {
            retire(&old)?;
        }
        Ok(())
    }

    fn migrate() {
        let Some(old) = legacy().filter(|old| old.exists()) else {
            return;
        };
        let mounted = std::env::current_exe()
            .ok()
            .is_some_and(|exe| crate::update::mounted(Some(&exe)));
        if mounted {
            return;
        }
        if login::state() == LoginState::Off {
            let _ = login::set(true);
        }
        if login::state() == LoginState::On {
            let _ = retire(&old);
        }
    }

    pub fn legacy_in(home: &Path) -> PathBuf {
        home.join("Library")
            .join("LaunchAgents")
            .join(format!("{LABEL}.plist"))
    }

    fn legacy() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|home| legacy_in(Path::new(&home)))
    }

    pub fn retire(plist: &Path) -> std::io::Result<bool> {
        match std::fs::remove_file(plist) {
            Ok(()) => Ok(true),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(why) => Err(why),
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod there {
    use super::Waking;

    pub fn waking() -> Waking {
        Waking::none()
    }

    pub fn wake(_wanted: bool) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(all(test, windows))]
#[path = "waking_windows_test.rs"]
mod tests;

#[cfg(all(test, target_os = "macos"))]
#[path = "waking_macos_test.rs"]
mod tests;
