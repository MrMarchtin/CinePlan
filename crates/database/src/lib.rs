//! Implémentation SQLite des dépôts définis dans `cineplan-core`,
//! et stockage des fichiers sur le disque local.

mod documents;
mod files;
mod members;
mod planning;
mod store;

pub use files::LocalFileStorage;
pub use store::SqliteStore;
