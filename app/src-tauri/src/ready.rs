use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalRect, Runtime, WebviewUrl, WebviewWindowBuilder,
};

pub const LABEL: &str = "ready";

const WIDE: f64 = 360.0;
const TALL: f64 = 184.0;
const MARGIN: f64 = 12.0;

#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Glance {
    items: i64,
    today: i64,
    pinned: i64,
    bytes: i64,
}

pub fn corner(area: PhysicalRect<i32, u32>, scale: f64, top: bool) -> PhysicalPosition<i32> {
    let wide = (WIDE * scale).round() as i32;
    let tall = (TALL * scale).round() as i32;
    let margin = (MARGIN * scale).round() as i32;
    let right = area.position.x + area.size.width as i32;
    let x = right - wide - margin;
    let y = if top {
        area.position.y + margin
    } else {
        area.position.y + area.size.height as i32 - tall - margin
    };
    PhysicalPosition::new(x, y)
}

pub fn open<R: Runtime>(app: &AppHandle<R>) {
    if app.get_webview_window(LABEL).is_some() {
        return;
    }
    let built = crate::looks::dressed(WebviewWindowBuilder::new(
        app,
        LABEL,
        WebviewUrl::App("index.html#ready".into()),
    ))
    .title("CoPanda")
    .inner_size(WIDE, TALL)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(false)
    .visible(false)
    .build();
    let window = match built {
        Ok(window) => window,
        Err(why) => {
            crate::note::note(&format!("the ready card would not open: {why}"));
            return;
        }
    };
    if let Ok(Some(screen)) = app.primary_monitor() {
        let at = corner(
            *screen.work_area(),
            screen.scale_factor(),
            cfg!(target_os = "macos"),
        );
        let _ = window.set_position(at);
    }
    let _ = window.show();
}

pub fn close<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.close();
    }
}

#[tauri::command(async)]
pub fn glance(since: i64) -> Result<Glance, String> {
    let at = crate::settings::folder()
        .ok_or_else(crate::settings::nowhere)?
        .join("history.db");
    if !at.exists() {
        return Ok(Glance::default());
    }
    let store = cp_store::Store::open(&at).map_err(|why| why.to_string())?;
    let seen = store.glance(since).map_err(|why| why.to_string())?;
    Ok(Glance {
        items: seen.items,
        today: seen.today,
        pinned: seen.pinned,
        bytes: seen.bytes,
    })
}

#[tauri::command]
pub async fn show_panel(app: AppHandle) {
    close(&app);
    crate::panel::show(&app);
}

#[cfg(test)]
#[path = "ready_test.rs"]
mod tests;
