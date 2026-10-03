use crate::ole::Ole;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoTaskMemFree, IDataObject};
use windows::Win32::System::Ole::{DROPEFFECT_COPY, IDropSource};
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::{
    BHID_DataObject, SHCreateShellItemArrayFromIDLists, SHDoDragDrop, SHParseDisplayName,
};
use windows::core::PCWSTR;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dragged {
    Started,
    Nothing,
    Refused,
    Elsewhere,
}

fn pidl_of(path: &Path) -> Option<*mut ITEMIDLIST> {
    let said: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut pidl = std::ptr::null_mut();
    unsafe { SHParseDisplayName(PCWSTR(said.as_ptr()), None, &mut pidl, 0, None) }.ok()?;
    (!pidl.is_null()).then_some(pidl)
}

fn data_object_of(paths: &[&Path]) -> Option<IDataObject> {
    let pidls: Vec<*mut ITEMIDLIST> = paths.iter().filter_map(|one| pidl_of(one)).collect();
    if pidls.is_empty() {
        return None;
    }
    let read: Vec<*const ITEMIDLIST> = pidls.iter().map(|one| one.cast_const()).collect();
    let made = unsafe { SHCreateShellItemArrayFromIDLists(&read) };
    for one in pidls {
        unsafe { CoTaskMemFree(Some(one.cast())) };
    }
    unsafe { made.ok()?.BindToHandler(None, &BHID_DataObject) }.ok()
}

pub fn from_window(window: isize, paths: &[&Path]) -> Dragged {
    if paths.is_empty() {
        return Dragged::Nothing;
    }
    let _ole = Ole::enter();
    let Some(data) = data_object_of(paths) else {
        return Dragged::Nothing;
    };
    let hwnd = HWND(window as *mut std::ffi::c_void);
    match unsafe { SHDoDragDrop(Some(hwnd), &data, None::<&IDropSource>, DROPEFFECT_COPY) } {
        Ok(_) => Dragged::Started,
        Err(_) => Dragged::Refused,
    }
}

#[cfg(test)]
#[path = "dragging_test.rs"]
mod tests;
