//! Dónde viven los datos en macOS.

use std::path::PathBuf;

/// La carpeta de la aplicación, la misma que usa la 2.x.
///
/// Se comparte a propósito: así el usuario tiene una sola carpeta y la
/// migración manual puede leer el archivo viejo sin buscarlo. Lo que **no**
/// se comparte es el nombre de la base, para que instalar la 3.0 no pise el
/// historial de la 2.x.
pub fn data_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("CopyPaste"),
    )
}

/// La base de la 3.0. La 2.x usa `clipboard.db` en esta misma carpeta.
pub fn database() -> Option<PathBuf> {
    Some(data_dir()?.join("history.db"))
}

/// El archivo de la 2.x, para cuando haya importación manual.
pub fn legacy_database() -> Option<PathBuf> {
    Some(data_dir()?.join("clipboard.db"))
}

pub fn blobs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("blobs"))
}
