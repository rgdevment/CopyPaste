pub mod blobs;
pub mod query;
pub mod schema;
pub mod store;

pub use blobs::Blobs;
pub use query::{Clock, parse};
pub use schema::SCHEMA_VERSION;
pub use store::{
    AppCount, Broken, Cursor, Facet, Filter, Listed, Order, Page, Policy, Restricted, Snippet,
    Store, Swept, Usage, Where,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("la base de datos: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("«{format}» ocupa {size} bytes y el almacén de blobs no existe todavía")]
    NeedsBlobStore { format: String, size: usize },
    #[error("el archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("la base es de la versión {found} y esta copia entiende hasta la {supported}")]
    FromTheFuture { found: u32, supported: u32 },
    #[error("{size} bytes es más de lo que un ítem puede guardar")]
    TooBig { size: usize },
    #[error("no hay ningún ítem {id}")]
    NoSuchItem { id: i64 },
}

pub type Result<T> = std::result::Result<T, Error>;
