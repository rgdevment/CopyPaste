mod settings;
mod tray;

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
            settings::former
        ])
        .setup(|app| {
            tray::raise(app.handle());
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
