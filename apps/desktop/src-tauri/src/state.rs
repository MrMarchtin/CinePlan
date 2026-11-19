use cineplan_core::{CoreResult, ProjectService, SceneService};
use cineplan_database::SqliteStore;
use std::path::Path;
use std::sync::Arc;

/// État partagé géré par Tauri : les services métier branchés sur SQLite.
pub struct AppState {
    pub projects: ProjectService,
    pub scenes: SceneService,
}

impl AppState {
    pub async fn open(db_path: &Path) -> CoreResult<Self> {
        let store = Arc::new(SqliteStore::connect(db_path).await?);
        Ok(Self {
            projects: ProjectService::new(store.clone()),
            scenes: SceneService::new(store),
        })
    }
}
