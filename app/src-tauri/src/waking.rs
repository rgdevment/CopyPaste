#[derive(serde::Serialize, PartialEq, Eq, Debug)]
pub struct Waking {
    pub offered: bool,
    pub wakes: bool,
    pub theirs: bool,
}

impl Waking {
    fn none() -> Self {
        Self {
            offered: false,
            wakes: false,
            theirs: false,
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
        let ours = written().is_some_and(|said| ours(&said, &exe));
        let approved = approved();
        Waking {
            offered: true,
            wakes: ours && approved,
            theirs: ours && !approved,
        }
    }

    pub fn wake(wanted: bool) -> std::io::Result<()> {
        let exe = std::env::current_exe()?;
        if written().is_some_and(|said| !ours(&said, &exe)) {
            return Err(std::io::Error::other(
                "otro programa ocupa el arranque con el nombre de CopyPaste",
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
    use std::path::{Path, PathBuf};

    const LABEL: &str = "com.rgdevment.copypaste";

    pub fn waking() -> Waking {
        let Ok(exe) = std::env::current_exe() else {
            return Waking::none();
        };
        let Some(plist) = at() else {
            return Waking::none();
        };
        let written = std::fs::read_to_string(&plist).ok();
        let ours = written
            .as_deref()
            .and_then(program_in)
            .is_some_and(|said| ours(&said, &exe));
        Waking {
            offered: true,
            wakes: ours,
            theirs: written.is_some() && !ours,
        }
    }

    pub fn wake(wanted: bool) -> std::io::Result<()> {
        let exe = std::env::current_exe()?;
        let plist =
            at().ok_or_else(|| std::io::Error::other("no se encontró la carpeta del usuario"))?;
        if let Ok(written) = std::fs::read_to_string(&plist)
            && program_in(&written).is_some_and(|said| !ours(&said, &exe))
        {
            return Err(std::io::Error::other(
                "otro programa ocupa el arranque con el nombre de CopyPaste",
            ));
        }
        if !wanted {
            return match std::fs::remove_file(&plist) {
                Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
                other => other,
            };
        }
        if let Some(dir) = plist.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&plist, agent_for(&exe))
    }

    fn at() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(
            PathBuf::from(home)
                .join("Library")
                .join("LaunchAgents")
                .join(format!("{LABEL}.plist")),
        )
    }

    pub fn agent_for(exe: &Path) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n\
             <dict>\n\
             \t<key>Label</key>\n\t<string>{LABEL}</string>\n\
             \t<key>ProgramArguments</key>\n\t<array>\n\t\t<string>{}</string>\n\t</array>\n\
             \t<key>RunAtLoad</key>\n\t<true/>\n\
             \t<key>ProcessType</key>\n\t<string>Interactive</string>\n\
             </dict>\n\
             </plist>\n",
            escaped(&exe.display().to_string())
        )
    }

    fn escaped(said: &str) -> String {
        said.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    fn unescaped(said: &str) -> String {
        said.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
    }

    pub fn program_in(said: &str) -> Option<String> {
        let after = said.split_once("<key>ProgramArguments</key>")?.1;
        let array = after.split_once("<array>")?.1;
        let (array, _) = array.split_once("</array>")?;
        let value = array.split_once("<string>")?.1;
        let (value, _) = value.split_once("</string>")?;
        Some(unescaped(value.trim()))
    }

    pub fn ours(said: &str, exe: &Path) -> bool {
        !said.is_empty() && Path::new(said) == exe
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
mod tests {
    use super::there::{approves, ours};
    use std::path::Path;

    #[test]
    fn only_the_two_bytes_that_mean_the_person_turned_it_off_count_as_off() {
        assert!(!approves(&[3, 0, 0, 0]));
        assert!(!approves(&[6, 0, 0, 0]));
        assert!(approves(&[2, 0, 0, 0]));
        assert!(approves(&[0, 0, 0, 0]));
        assert!(approves(&[1, 0, 0, 0]));
        assert!(approves(&[]));
    }

    #[test]
    fn an_entry_that_names_another_program_is_not_ours() {
        let exe = Path::new(r"C:\Programas\CopyPaste\cp-gui.exe");
        assert!(ours(r"C:\Programas\CopyPaste\cp-gui.exe", exe));
        assert!(ours(r#""C:\Programas\CopyPaste\cp-gui.exe""#, exe));
        assert!(ours(r#"  "c:\programas\copypaste\CP-GUI.EXE"  "#, exe));
        assert!(!ours(r"C:\Otro\cp-gui.exe", exe));
        assert!(!ours(r#""""#, exe));
        assert!(!ours("", exe));
    }

    #[test]
    fn an_entry_with_arguments_is_not_taken_for_the_bare_path() {
        let exe = Path::new(r"C:\Programas\CopyPaste\cp-gui.exe");
        assert!(!ours(r"C:\Programas\CopyPaste\cp-gui.exe --hushed", exe));
        assert!(!ours(
            r#""C:\Programas\CopyPaste\cp-gui.exe" --hushed"#,
            exe
        ));
        assert!(!ours(r#""C:\Programas\CopyPaste\cp-gui.exe"#, exe));
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::there::{agent_for, ours, program_in};
    use std::path::Path;

    #[test]
    fn the_agent_names_the_binary_that_wrote_it() {
        let exe = Path::new("/Applications/CopyPaste.app/Contents/MacOS/CopyPaste");
        let written = agent_for(exe);
        assert_eq!(program_in(&written).as_deref(), exe.to_str());
        assert!(written.contains("<key>RunAtLoad</key>"));
        assert!(written.starts_with("<?xml"));
    }

    #[test]
    fn an_entry_that_names_another_program_is_not_ours() {
        let exe = Path::new("/Applications/CopyPaste.app/Contents/MacOS/CopyPaste");
        assert!(ours(
            "/Applications/CopyPaste.app/Contents/MacOS/CopyPaste",
            exe
        ));
        assert!(!ours(
            "/Applications/Otro.app/Contents/MacOS/CopyPaste",
            exe
        ));
        assert!(!ours("", exe));
    }

    #[test]
    fn a_path_the_xml_had_to_escape_comes_back_whole() {
        let exe = Path::new("/Users/alguien/Rock & Roll/CopyPaste.app/Contents/MacOS/CopyPaste");
        let written = agent_for(exe);
        assert!(written.contains("&amp;"), "el & crudo rompe el plist");
        assert_eq!(program_in(&written).as_deref(), exe.to_str());
        assert!(ours(&program_in(&written).expect("ruta"), exe));
    }

    #[test]
    fn a_plist_without_program_arguments_says_nothing() {
        assert_eq!(program_in("<plist><dict></dict></plist>"), None);
        assert_eq!(program_in(""), None);
    }
}
