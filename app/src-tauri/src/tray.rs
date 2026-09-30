use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

pub struct Said<R: Runtime>(Mutex<Vec<MenuItem<R>>>);

pub fn spanish(locale: Option<&str>) -> bool {
    let asked = locale
        .map(str::to_owned)
        .or_else(sys_locale::get_locale)
        .unwrap_or_default();
    !asked.to_lowercase().starts_with("en")
}

fn worded(spanish: bool) -> [&'static str; 3] {
    if spanish {
        ["Mostrar el panel", "Ajustes…", "Salir de CopyPaste"]
    } else {
        ["Show the panel", "Settings…", "Quit CopyPaste"]
    }
}

#[cfg(target_os = "macos")]
const BAR: &[u8] = include_bytes!("../icons/tray/macos@2x.png");
#[cfg(not(target_os = "macos"))]
const BAR: &[u8] = include_bytes!("../icons/tray/windows-32.png");

fn leave<R: Runtime>(app: &AppHandle<R>) {
    if app
        .try_state::<crate::backup::Crossing>()
        .is_some_and(|crossing| crossing.underway())
    {
        surface(app);
        return;
    }
    app.exit(0);
}

pub fn raise<R: Runtime>(app: &AppHandle<R>, spanish: bool) -> Option<()> {
    let [shows, asks, leaves] = worded(spanish);
    let panel = MenuItem::with_id(app, "panel", shows, true, None::<&str>).ok()?;
    let settings = MenuItem::with_id(app, "settings", asks, true, None::<&str>).ok()?;
    let quit = MenuItem::with_id(app, "quit", leaves, true, None::<&str>).ok()?;
    let menu = Menu::with_items(app, &[&panel, &settings, &quit]).ok()?;

    let tray = TrayIconBuilder::with_id("copypaste")
        .icon(Image::from_bytes(BAR).ok()?)
        .tooltip("CopyPaste")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "panel" => crate::panel::show(app),
            "settings" => surface(app),
            "quit" => leave(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::panel::show(tray.app_handle());
            }
        })
        .build(app)
        .ok()?;

    #[cfg(target_os = "macos")]
    let _ = tray.set_icon_as_template(true);
    #[cfg(not(target_os = "macos"))]
    let _ = &tray;

    app.manage(Said(Mutex::new(vec![panel, settings, quit])));

    Some(())
}

pub fn reword<R: Runtime>(app: &AppHandle<R>, spanish: bool) {
    let Some(items) = app.try_state::<Said<R>>() else {
        return;
    };
    let Ok(items) = items.0.lock() else {
        return;
    };
    for (item, said) in items.iter().zip(worded(spanish)) {
        let _ = item.set_text(said);
    }
}

pub fn surface<R: Runtime>(app: &AppHandle<R>) {
    crate::panel::hide(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("CopyPaste")
        .inner_size(780.0, 580.0)
        .min_inner_size(620.0, 460.0)
        .decorations(false)
        .center()
        .build();
}

#[cfg(test)]
#[path = "tray_test.rs"]
mod tests;
