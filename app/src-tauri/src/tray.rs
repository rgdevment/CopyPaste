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

fn worded(spanish: bool) -> [&'static str; 2] {
    if spanish {
        ["Ajustes…", "Salir de CopyPaste"]
    } else {
        ["Settings…", "Quit CopyPaste"]
    }
}

#[cfg(target_os = "macos")]
const BAR: &[u8] = include_bytes!("../icons/tray/macos@2x.png");
#[cfg(not(target_os = "macos"))]
const BAR: &[u8] = include_bytes!("../icons/tray/windows-32.png");

pub fn raise<R: Runtime>(app: &AppHandle<R>, spanish: bool) -> Option<()> {
    let [asks, leaves] = worded(spanish);
    let settings = MenuItem::with_id(app, "settings", asks, true, None::<&str>).ok()?;
    let quit = MenuItem::with_id(app, "quit", leaves, true, None::<&str>).ok()?;
    let menu = Menu::with_items(app, &[&settings, &quit]).ok()?;

    let tray = TrayIconBuilder::with_id("copypaste")
        .icon(Image::from_bytes(BAR).ok()?)
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

    #[cfg(target_os = "macos")]
    let _ = tray.set_icon_as_template(true);
    #[cfg(not(target_os = "macos"))]
    let _ = &tray;

    app.manage(Said(Mutex::new(vec![settings, quit])));

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
mod tests {
    use super::spanish;
    use tauri::image::Image;

    const WINDOWS: &[u8] = include_bytes!("../icons/tray/windows-32.png");
    const MACOS: &[u8] = include_bytes!("../icons/tray/macos@2x.png");

    fn ink(png: &[u8]) -> (u32, usize) {
        let art = Image::from_bytes(png).expect("el icono es un png");
        let pixels = art.rgba();
        let (sum, seen) =
            pixels
                .as_chunks::<4>()
                .0
                .iter()
                .fold((0u64, 0usize), |(sum, seen), px| {
                    if px[3] < 128 {
                        return (sum, seen);
                    }
                    let grey =
                        (px[0] as u64 * 299 + px[1] as u64 * 587 + px[2] as u64 * 114) / 1000;
                    (sum + grey, seen + 1)
                });
        assert!(seen > 0, "el icono es transparente entero");
        ((sum / seen as u64) as u32, seen)
    }

    #[test]
    fn the_bar_icon_on_macos_is_pale_because_the_system_paints_it_itself() {
        let (grey, _) = ink(MACOS);
        assert!(grey > 200, "la plantilla de macOS debe ser clara: {grey}");
    }

    #[test]
    fn only_a_locale_that_starts_with_en_gets_english() {
        assert!(!spanish(Some("en")));
        assert!(!spanish(Some("en-GB")));
        assert!(!spanish(Some("EN-us")));
        assert!(spanish(Some("es")));
        assert!(spanish(Some("es-CL")));
        assert!(spanish(Some("pt-BR")));
    }

    #[test]
    fn the_windows_icon_carries_its_own_colour() {
        let (_, seen) = ink(WINDOWS);
        assert!(seen > 64, "el icono de Windows apenas tiene tinta: {seen}");
    }
}
