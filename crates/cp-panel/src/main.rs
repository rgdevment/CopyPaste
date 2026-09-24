mod age;
mod app;
mod engine;
mod measure;
mod model;
mod note;
mod view;

slint::include_modules!();

use std::path::PathBuf;

fn main() {
    let options = Options::from_args();
    note::catch_panics();
    note::note(&format!("arranca sobre {}", options.db.display()));
    eprintln!(
        "las incidencias se anotan en {}",
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
            note::note(&format!("no se pudo abrir {}: {why}", options.db.display()));
            eprintln!("no se pudo abrir {}: {why}", options.db.display());
            std::process::exit(1);
        }
    };
    let (panel, app) = match app::App::start(store, options.clone()) {
        Ok(started) => started,
        Err(why) => {
            note::note(&format!("el panel no arrancó: {why}"));
            eprintln!("el panel no arrancó: {why}");
            std::process::exit(1);
        }
    };
    if let Err(why) = app.run(&panel) {
        eprintln!("el panel se cerró con error: {why}");
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
    fn from_args() -> Self {
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
                other => eprintln!("argumento ignorado: {other}"),
            }
        }
        Self {
            db: db.unwrap_or_else(where_it_lives),
            measure,
            serve,
            signals: std::env::var_os("CP_PANEL_SIGNALS").map(PathBuf::from),
            backdrop,
            flat,
        }
    }
}

fn where_it_lives() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(path) = cp_win_sys::paths::database() {
        return path;
    }
    std::env::temp_dir().join("cp-seed").join("history.db")
}
