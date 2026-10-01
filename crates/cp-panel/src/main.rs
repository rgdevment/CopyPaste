mod age;
mod app;
mod engine;
mod here;
mod layout;
mod measure;
mod media;
mod model;
mod note;
mod say;
mod shape;
mod view;

slint::include_modules!();

use std::path::PathBuf;

fn main() {
    note::catch_panics();
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
    if std::env::var_os("SLINT_BACKEND").is_none() {
        let _ = slint::BackendSelector::new()
            .backend_name("winit".into())
            .renderer_name("skia-software".into())
            .select();
    }
    let store = match cp_store::Store::open(&options.db) {
        Ok(store) => store,
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
        eprintln!("the panel closed with an error: {why}");
        std::process::exit(1);
    }
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
