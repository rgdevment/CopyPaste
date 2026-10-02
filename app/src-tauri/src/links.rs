fn opened(url: &str) -> bool {
    #[cfg(windows)]
    {
        cp_win_sys::files::open_link(url)
    }
    #[cfg(target_os = "macos")]
    {
        cp_mac_sys::files::open_link(url)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = url;
        false
    }
}

#[tauri::command(async)]
pub fn open_web(url: String) -> Result<(), String> {
    if !cp_core::linkunbound::opens_the_web(&url) {
        return Err(url);
    }
    if opened(&url) {
        return Ok(());
    }
    Err(url)
}

#[tauri::command(async)]
pub fn linkunbound_here() -> bool {
    #[cfg(windows)]
    {
        cp_win_sys::files::linkunbound_here()
    }
    #[cfg(target_os = "macos")]
    {
        cp_mac_sys::files::linkunbound_here()
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        false
    }
}
