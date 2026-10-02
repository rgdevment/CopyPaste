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

pub fn open_link(url: &str) -> bool {
    let wide: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
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
#[path = "files_test.rs"]
mod tests;
