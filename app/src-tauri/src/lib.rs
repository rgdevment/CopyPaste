mod settings;
mod tray;
mod waking;

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tray::surface(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            settings::settings,
            settings::keep,
            settings::where_it_lives,
            settings::former,
            waking::waking,
            waking::wake,
            relabel
        ])
        .setup(|app| {
            let kept = settings::settings().ok();
            let spanish = tray::spanish(kept.as_ref().and_then(|one| one.locale.as_deref()));
            if tray::raise(app.handle(), spanish).is_none() {
                tray::surface(app.handle());
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("CopyPaste no pudo arrancar");

    app.run(|_app, event| {
        if let tauri::RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}

#[tauri::command]
fn relabel(app: tauri::AppHandle, locale: Option<String>) {
    tray::reword(&app, tray::spanish(locale.as_deref()));
}
