use tauri::window::Color;
use tauri::{AppHandle, Manager, Runtime, Theme, WebviewWindowBuilder};

const LIGHT: Color = Color(0xF7, 0xF8, 0xFB, 0xFF);
const DARK: Color = Color(0x1A, 0x1D, 0x2B, 0xFF);
const WINDOWS: [&str; 2] = ["main", "welcome"];

fn chosen() -> Option<Theme> {
    match crate::settings::settings().map(|kept| kept.theme) {
        Ok(cp_config::Theme::Light) => Some(Theme::Light),
        Ok(cp_config::Theme::Dark) => Some(Theme::Dark),
        _ => None,
    }
}

fn system_is_light() -> bool {
    #[cfg(target_os = "macos")]
    return cp_mac_sys::theme::wants_light().unwrap_or(true);
    #[cfg(target_os = "windows")]
    return cp_win_sys::theme::wants_light().unwrap_or(true);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    true
}

fn ground(chosen: Option<Theme>) -> Color {
    match chosen {
        Some(Theme::Dark) => DARK,
        Some(_) => LIGHT,
        None if system_is_light() => LIGHT,
        None => DARK,
    }
}

pub fn dressed<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M> {
    let chosen = chosen();
    builder.theme(chosen).background_color(ground(chosen))
}

pub fn wear<R: Runtime>(app: &AppHandle<R>) {
    let chosen = chosen();
    for label in WINDOWS {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.set_theme(chosen);
            let _ = window.set_background_color(Some(ground(chosen)));
        }
    }
}
