use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

const SMALL: &[u8] = include_bytes!("../icons/tray/tray-32.png");
const LARGE: &[u8] = include_bytes!("../icons/tray/tray-64.png");

pub fn raise<R: Runtime>(app: &AppHandle<R>) -> Option<()> {
    let settings = MenuItem::with_id(app, "settings", "Ajustes…", true, None::<&str>).ok()?;
    let quit = MenuItem::with_id(app, "quit", "Salir de CopyPaste", true, None::<&str>).ok()?;
    let menu = Menu::with_items(app, &[&settings, &quit]).ok()?;
    let icon = art()?;

    TrayIconBuilder::with_id("copypaste")
        .icon(icon)
        .tooltip("CopyPaste")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => surface(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                surface(tray.app_handle());
            }
        })
        .build(app)
        .ok()?;

    Some(())
}

fn art() -> Option<Image<'static>> {
    let bytes = if cfg!(target_os = "macos") {
        LARGE
    } else {
        SMALL
    };
    Image::from_bytes(bytes).ok()
}

pub fn surface<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}
