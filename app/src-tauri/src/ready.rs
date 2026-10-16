use tauri::{
    AppHandle, LogicalPosition, Manager, PhysicalRect, Runtime, WebviewUrl, WebviewWindowBuilder,
};

pub const LABEL: &str = "ready";

const WIDE: f64 = 360.0;
const TALL: f64 = 184.0;
const MARGIN: f64 = 12.0;

pub fn corner(area: PhysicalRect<i32, u32>, scale: f64, top: bool) -> LogicalPosition<f64> {
    let left = f64::from(area.position.x) / scale;
    let up = f64::from(area.position.y) / scale;
    let x = left + f64::from(area.size.width) / scale - WIDE - MARGIN;
    let y = if top {
        up + MARGIN
    } else {
        up + f64::from(area.size.height) / scale - TALL - MARGIN
    };
    LogicalPosition::new(x, y)
}

pub fn open<R: Runtime>(app: &AppHandle<R>) {
    if app.get_webview_window(LABEL).is_some() {
        return;
    }
    let Ok(Some(screen)) = app.primary_monitor() else {
        return;
    };
    let at = corner(
        *screen.work_area(),
        screen.scale_factor(),
        cfg!(target_os = "macos"),
    );
    let built = crate::looks::dressed(WebviewWindowBuilder::new(
        app,
        LABEL,
        WebviewUrl::App("index.html#ready".into()),
    ))
    .title("CoPanda")
    .inner_size(WIDE, TALL)
    .position(at.x, at.y)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(false)
    .accept_first_mouse(true)
    .build();
    if let Err(why) = built {
        crate::note::note(&format!("the ready card would not open: {why}"));
    }
}

pub fn close<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.close();
    }
}

#[tauri::command(async)]
pub fn glance(since: i64) -> Result<cp_store::Glance, String> {
    let Some(store) = crate::settings::history_if_any()? else {
        return Ok(cp_store::Glance::default());
    };
    store.glance(since).map_err(|why| why.to_string())
}

#[tauri::command]
pub async fn show_panel(app: AppHandle) {
    close(&app);
    crate::panel::show(&app);
}

#[cfg(test)]
#[path = "ready_test.rs"]
mod tests;
