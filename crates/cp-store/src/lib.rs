pub mod backup;
pub mod blobs;
pub mod enrich;
pub mod legacy;
pub mod query;
pub mod schema;
pub mod store;

pub use backup::{Brought, Made, Taken};
pub use blobs::Blobs;
pub use enrich::MetaByItem;
pub use query::{Clock, parse};
pub use schema::SCHEMA_VERSION;
pub use store::restrict;
pub use store::{
    A_DAY, AppCount, Broken, Cursor, Facet, Filter, FoundIn, Listed, More, Order, PREVIEW_CHARS,
    PREVIEW_UP_TO, Page, Policy, Restricted, Snippet, Store, Swept, Usage,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the database: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("«{format}» takes up {size} bytes and the blob store does not exist yet")]
    NeedsBlobStore { format: String, size: usize },
    #[error("the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("the database is at version {found} and this copy understands up to {supported}")]
    FromTheFuture { found: u32, supported: u32 },
    #[error("{size} bytes is more than an item can hold")]
    TooBig { size: usize },
    #[error("there is no item {id}")]
    NoSuchItem { id: i64 },
    #[error("the file is not a CopyPaste backup")]
    NotABackup,
    #[error("a copy cannot be written over the history it is copying")]
    OntoItself,
    #[error("the file is not CopyPaste 2's history")]
    NotTheFormerOne,
    #[error("the backup is at format {found} and this version understands up to {supported}")]
    BackupFromTheFuture { found: u32, supported: u32 },
    #[error("the cursor is for one order ({cursor}) and the list for another ({order})")]
    WrongCursor {
        cursor: &'static str,
        order: &'static str,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
