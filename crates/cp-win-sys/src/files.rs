use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::UI::Shell::{
    ASSOCF_NONE, ASSOCSTR_EXECUTABLE, AssocQueryStringW, ILCreateFromPathW, ILFree,
    SHOpenFolderAndSelectItems, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, PWSTR, w};

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

pub fn linkunbound_here() -> bool {
    scheme_here(cp_core::linkunbound::SCHEME)
}

fn scheme_here(scheme: &str) -> bool {
    let scheme: Vec<u16> = scheme.encode_utf16().chain(std::iter::once(0)).collect();
    let mut room: u32 = 0;
    let asked = unsafe {
        AssocQueryStringW(
            ASSOCF_NONE,
            ASSOCSTR_EXECUTABLE,
            PCWSTR(scheme.as_ptr()),
            PCWSTR::null(),
            None,
            &raw mut room,
        )
    };
    if asked.is_err() || room == 0 {
        return false;
    }
    let mut said = vec![0u16; room as usize];
    let read = unsafe {
        AssocQueryStringW(
            ASSOCF_NONE,
            ASSOCSTR_EXECUTABLE,
            PCWSTR(scheme.as_ptr()),
            PCWSTR::null(),
            Some(PWSTR(said.as_mut_ptr())),
            &raw mut room,
        )
    };
    read.is_ok() && said.first().is_some_and(|first| *first != 0)
}

pub fn open_link(url: &str) -> bool {
    cp_core::linkunbound::opened_by(url, linkunbound_here(), shown)
}

fn shown(url: &str) -> bool {
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
