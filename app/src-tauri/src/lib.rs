mod backup;
mod keys;
mod links;
mod looks;
mod note;
mod panel;
mod reviving;
mod settings;
mod tray;
mod trust;
mod update;
mod waking;
mod welcome;

pub fn run() {
    note::catch_panics();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            welcome::reopened(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            settings::settings,
            settings::keep,
            settings::where_it_lives,
            settings::former,
            settings::notices,
            settings::licences,
            links::open_web,
            links::linkunbound_here,
            waking::waking,
            waking::wake,
            keys::keys,
            keys::spare,
            trust::trust,
            trust::ask_trust,
            backup::save_backup,
            backup::peek_backup,
            backup::load_backup,
            backup::bring_former,
            backup::drop_former,
            update::update_ready,
            update::update_install,
            empty,
            trouble,
            relabel,
            welcome::greeting,
            welcome::tour,
            welcome::open_settings
        ])
        .manage(update::Installing::default())
        .manage(backup::Crossing::default())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let fresh = settings::settle();
            let kept = settings::settings().ok();
            let spanish = tray::spanish(kept.as_ref().and_then(|one| one.locale.as_deref()));
            #[cfg(target_os = "macos")]
            if let Err(why) = tray::settle_menu(app.handle()) {
                note::note(&format!("the menu kept its quit key: {why}"));
            }
            if tray::raise(app.handle(), spanish).is_none() {
                tray::surface(app.handle());
            }
            panel::raise(app.handle());
            let wanted = kept
                .as_ref()
                .map_or(cp_config::SHORTCUT, |one| one.shortcut.as_str());
            keys::raise(app.handle(), wanted);
            welcome::raise(app.handle(), fresh);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("CopyPaste could not start");

    app.run(|app, event| match event {
        tauri::RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        tauri::RunEvent::Exit => panel::quit(app),
        tauri::RunEvent::WindowEvent {
            event: tauri::WindowEvent::ThemeChanged(_),
            ..
        } => looks::wear(app),
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } => welcome::reopened(app),
        _ => {}
    });
}

#[tauri::command]
fn empty(app: tauri::AppHandle) -> Result<(), String> {
    panel::empty(&app)
}

#[tauri::command]
fn trouble(app: tauri::AppHandle) -> Option<String> {
    panel::trouble(&app)
}

#[tauri::command]
fn relabel(app: tauri::AppHandle, locale: Option<String>) {
    tray::reword(&app, tray::spanish(locale.as_deref()));
}
