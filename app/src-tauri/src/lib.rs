mod keys;
mod note;
mod panel;
mod settings;
mod tray;
mod waking;

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tray::surface(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            settings::settings,
            settings::keep,
            settings::where_it_lives,
            settings::former,
            waking::waking,
            waking::wake,
            keys::keys,
            empty,
            relabel
        ])
        .setup(|app| {
            let kept = settings::settings().ok();
            let spanish = tray::spanish(kept.as_ref().and_then(|one| one.locale.as_deref()));
            if tray::raise(app.handle(), spanish).is_none() {
                tray::surface(app.handle());
            }
            panel::raise(app.handle());
            let wanted = kept
                .as_ref()
                .map_or(cp_config::SHORTCUT, |one| one.shortcut.as_str());
            keys::raise(app.handle(), wanted);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("CopyPaste no pudo arrancar");

    app.run(|app, event| match event {
        tauri::RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        tauri::RunEvent::Exit => panel::quit(app),
        _ => {}
    });
}

#[tauri::command]
fn empty(app: tauri::AppHandle) -> Result<(), String> {
    panel::empty(&app)
}

#[tauri::command]
fn relabel(app: tauri::AppHandle, locale: Option<String>) {
    tray::reword(&app, tray::spanish(locale.as_deref()));
}
