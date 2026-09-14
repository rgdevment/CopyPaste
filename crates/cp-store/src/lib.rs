pub mod blobs;
pub mod schema;
pub mod store;

pub use blobs::Blobs;
pub use schema::SCHEMA_VERSION;
pub use store::Store;

/// Lo que puede salir mal en el almacén.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("la base de datos: {0}")]
    Db(#[from] rusqlite::Error),
    /// Un formato que no cabe en la fila y necesita el almacén de blobs, que
    /// todavía no existe. Se rechaza en vez de guardar la fila sin sus bytes.
    #[error("«{format}» ocupa {size} bytes y el almacén de blobs no existe todavía")]
    NeedsBlobStore { format: String, size: usize },
    #[error("el archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("la base es de la versión {found} y esta copia entiende hasta la {supported}")]
    FromTheFuture { found: u32, supported: u32 },
}

pub type Result<T> = std::result::Result<T, Error>;
