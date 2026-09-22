use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::UI::Shell::{
    ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

use crate::com::{Apartment, shell_path};

const RUNS_WHEN_OPENED: [&str; 8] = ["bat", "cmd", "com", "exe", "lnk", "msi", "ps1", "scr"];

const LAUNCHED: usize = 32;

pub fn runs_when_opened(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| RUNS_WHEN_OPENED.contains(&ext.to_ascii_lowercase().as_str()))
}

pub fn open(path: &Path) -> bool {
    if runs_when_opened(path) {
        return reveal(path);
    }
    let Some(wide) = wide_of(path) else {
        return false;
    };
    let _apartment = Apartment::enter();

    let instance = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(wide.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    instance.0 as usize > LAUNCHED
}

pub fn reveal(path: &Path) -> bool {
    let Some(wide) = wide_of(path) else {
        return false;
    };
    let _apartment = Apartment::enter();

    let list = unsafe { ILCreateFromPathW(PCWSTR(wide.as_ptr())) };
    if list.is_null() {
        return false;
    }

    let selected = unsafe { SHOpenFolderAndSelectItems(list, None, 0) };

    unsafe { ILFree(Some(list)) };
    selected.is_ok()
}

fn wide_of(path: &Path) -> Option<Vec<u16>> {
    let absolute = shell_path(path)?;
    Some(
        absolute
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_would_run_is_revealed_instead_of_opened() {
        for name in [
            "setup.exe",
            "script.BAT",
            "run.cmd",
            "tool.ps1",
            "install.msi",
            "atajo.lnk",
            "viejo.com",
            "fondo.scr",
        ] {
            assert!(runs_when_opened(Path::new(name)), "{name} se ejecuta");
        }
        for name in [
            "informe.pdf",
            "foto.png",
            "nota.txt",
            "sin-extension",
            "carpeta.d",
        ] {
            assert!(!runs_when_opened(Path::new(name)), "{name} se abre");
        }
    }

    #[test]
    fn a_path_that_is_not_there_is_neither_opened_nor_revealed() {
        let missing = Path::new(r"C:\no-existe-nada-de-nada\ni-esto.txt");
        assert!(!open(missing));
        assert!(!reveal(missing));
        assert_eq!(wide_of(missing), None);
    }

    #[test]
    fn the_wide_path_is_absolute_and_ends_in_a_terminator() {
        let wide = wide_of(Path::new(".")).expect("ruta");
        assert_eq!(wide.last(), Some(&0));
        let text = String::from_utf16_lossy(&wide[..wide.len() - 1]);
        assert!(text.contains(':'), "{text} no es absoluta");
        assert!(
            !text.starts_with(r"\\?\"),
            "{text} lleva el prefijo que el shell no entiende"
        );
    }
}
