mod age;
mod app;
mod dragging;
mod engine;
mod excuse;
mod face;
mod folder;
mod group;
mod here;
mod kept;
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
mod landing;
mod layout;
mod link;
mod measure;
mod media;
mod model;
mod note;
mod opening;
mod paired;
mod papers;
mod placing;
mod reaching;
mod say;
mod shape;
mod showing;
mod tags;
mod token;
mod view;
mod wave;
mod ways;

slint::include_modules!();

use std::path::PathBuf;

fn stand_aside() -> Result<(), slint::PlatformError> {
    let builder = i_slint_backend_winit::Backend::builder().with_renderer_name("skia-software");
    #[cfg(target_os = "macos")]
    let builder = {
        use i_slint_backend_winit::winit::platform::macos::{
            ActivationPolicy, EventLoopBuilderExtMacOS,
        };
        let mut quiet = i_slint_backend_winit::winit::event_loop::EventLoop::with_user_event();
        quiet
            .with_default_menu(false)
            .with_activation_policy(ActivationPolicy::Accessory)
            .with_activate_ignoring_other_apps(false);
        builder
            .with_default_menu_bar(false)
            .with_event_loop_builder(quiet)
    };
    slint::platform::set_platform(Box::new(builder.build()?))
        .map_err(|why| slint::PlatformError::Other(why.to_string()))
}

fn fall(why: &str) {
    note::note(why);
    std::process::abort();
}

fn main() {
    note::catch_panics();
    note::end_on_panic(fall);
    let options = match Options::from_args() {
        Ok(options) => options,
        Err(why) => {
            note::trouble(why);
            std::process::exit(1);
        }
    };
    note::note(&format!("starting over {}", options.db.display()));
    eprintln!(
        "anything worth noting is written to {}",
        note::where_to().display()
    );
    if std::env::var_os("SLINT_BACKEND").is_none()
        && let Err(why) = stand_aside()
    {
        note::note(&format!("the panel kept the default window system: {why}"));
    }
    let store = match cp_store::open_or_set_aside(&options.db, app::now_ms()) {
        Ok(opened) => {
            if let Some(kept) = opened.set_aside {
                note::trouble(&format!(
                    "the history was damaged and a new one was started; the old file was kept as {}",
                    kept.display()
                ));
            }
            opened.store
        }
        Err(why) => {
            note::trouble(&format!("the history could not be opened: {why}"));
            std::process::exit(1);
        }
    };
    let (panel, app) = match app::App::start(store, options.clone()) {
        Ok(started) => started,
        Err(why) => {
            note::trouble(&format!("the panel could not be drawn: {why}"));
            std::process::exit(1);
        }
    };
    if let Err(why) = app.run(&panel) {
        note::note(&format!("the panel closed with an error: {why}"));
        eprintln!("the panel closed with an error: {why}");
        std::process::exit(1);
    }
    note::note("the event loop is done");
    app.close();
    note::note("the panel closed cleanly");
}

#[derive(Debug, Clone)]
pub struct Options {
    pub db: PathBuf,
    pub measure: bool,
    pub serve: bool,
    pub signals: Option<PathBuf>,
    pub backdrop: String,
    pub flat: bool,
}

impl Options {
    fn from_args() -> Result<Self, &'static str> {
        let mut db = None;
        let mut measure = false;
        let mut serve = false;
        let mut flat = false;
        let mut backdrop = "none".to_owned();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--db" => db = args.next().map(PathBuf::from),
                "--measure" => measure = true,
                "--serve" => serve = true,
                "--flat" => flat = true,
                "--backdrop" => backdrop = args.next().unwrap_or_default(),
                other => eprintln!("argument ignored: {other}"),
            }
        }
        let db = match db {
            Some(db) => db,
            None if measure => seeded(),
            None => where_it_lives().ok_or("the folder the history lives in was not found")?,
        };
        Ok(Self {
            db,
            measure,
            serve,
            signals: std::env::var_os("CP_PANEL_SIGNALS").map(PathBuf::from),
            backdrop,
            flat,
        })
    }
}

fn seeded() -> PathBuf {
    std::env::temp_dir().join("cp-seed").join("history.db")
}

fn where_it_lives() -> Option<PathBuf> {
    here::data_dir().map(|dir| dir.join("history.db"))
}

#[cfg(test)]
#[path = "main_test.rs"]
mod tests;
