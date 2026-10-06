use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

pub struct Said<R: Runtime>(Mutex<Vec<MenuItem<R>>>);

pub fn spanish(locale: Option<&str>) -> bool {
    let asked = locale
        .map(str::to_owned)
        .or_else(sys_locale::get_locale)
        .unwrap_or_default();
    !asked.to_lowercase().starts_with("en")
}

fn worded(spanish: bool) -> [&'static str; 4] {
    if spanish {
        [
            "Mostrar el panel",
            "Ajustes…",
            "Reiniciar el panel",
            "Salir de CopyPaste",
        ]
    } else {
        [
            "Show the panel",
            "Settings…",
            "Restart the panel",
            "Quit CopyPaste",
        ]
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

fn revive<R: Runtime>(app: &AppHandle<R>) {
    if app
        .try_state::<crate::backup::Crossing>()
        .is_some_and(|crossing| crossing.underway())
    {
        surface(app);
        return;
    }
    crate::panel::quit(app);
    if let Err(why) = crate::panel::relight(app) {
        crate::note::note(&format!("the panel would not come back: {why}"));
    }
}

pub fn raise<R: Runtime>(app: &AppHandle<R>, spanish: bool) -> Option<()> {
    let [shows, asks, again, leaves] = worded(spanish);
    let panel = MenuItem::with_id(app, "panel", shows, true, None::<&str>).ok()?;
    let settings = MenuItem::with_id(app, "settings", asks, true, None::<&str>).ok()?;
    let restart = MenuItem::with_id(app, "restart", again, true, None::<&str>).ok()?;
    let quit = MenuItem::with_id(app, "quit", leaves, true, None::<&str>).ok()?;
    let menu = Menu::with_items(app, &[&panel, &settings, &restart, &quit]).ok()?;

    let tray = TrayIconBuilder::with_id("copypaste")
        .icon(Image::from_bytes(BAR).ok()?)
        .tooltip("CopyPaste")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "panel" => crate::panel::show(app),
            "settings" => surface(app),
            "restart" => revive(app),
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

    app.manage(Said(Mutex::new(vec![panel, settings, restart, quit])));

    Some(())
}

pub fn tell<R: Runtime>(app: &AppHandle<R>, trouble: Option<&str>) {
    let said = trouble.map_or_else(
        || "CopyPaste".to_owned(),
        |what| format!("CopyPaste: {what}"),
    );
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(tray) = handle.tray_by_id("copypaste") {
            let _ = tray.set_tooltip(Some(said));
        }
    });
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
    surface_at(app, None);
}

pub fn surface_at<R: Runtime>(app: &AppHandle<R>, rail: Option<&str>) {
    crate::panel::hide(app);
    if let Some(window) = app.get_webview_window("main") {
        if let Some(rail) = rail {
            let _ = app.emit("rail", rail);
        }
        let _ = window.show();
        let _ = window.unminimize();
        ahead(&window);
        return;
    }

    let page = rail.map_or(WebviewUrl::default(), |rail| {
        WebviewUrl::App(format!("index.html#{rail}").into())
    });
    let built = framed(
        WebviewWindowBuilder::new(app, "main", page)
            .title("CopyPaste")
            .inner_size(780.0, 580.0)
            .min_inner_size(620.0, 460.0)
            .center(),
    )
    .build();
    if let Ok(window) = built {
        unzoomed(&window);
        ahead(&window);
    }
}

#[cfg(target_os = "macos")]
fn unzoomed<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    let _ = window.with_webview(|webview| {
        if let Some(ns_window) = std::ptr::NonNull::new(webview.ns_window()) {
            cp_mac_sys::titlebar::without_zoom(ns_window);
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn unzoomed<R: Runtime>(_window: &tauri::WebviewWindow<R>) {}

#[cfg(target_os = "macos")]
fn framed<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M> {
    builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        .maximizable(false)
        .traffic_light_position(tauri::LogicalPosition::new(14.0, 17.0))
}

#[cfg(not(target_os = "macos"))]
fn framed<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M> {
    builder.decorations(false)
}

fn ahead<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    let was = window.is_always_on_top().unwrap_or(false);
    let _ = window.set_always_on_top(true);
    let _ = window.set_focus();
    if !was {
        let _ = window.set_always_on_top(false);
    }
}

#[cfg(test)]
#[path = "tray_test.rs"]
mod tests;
