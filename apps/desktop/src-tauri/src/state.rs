use cineplan_core::{
    CoreResult, DocumentService, MemberService, PlanningService, ProjectService, SceneService,
};
use cineplan_database::{LocalFileStorage, SqliteStore};
use std::path::Path;
use std::sync::Arc;

/// État partagé géré par Tauri : les services métier branchés sur SQLite
/// et sur le stockage de fichiers local.
pub struct AppState {
    pub projects: ProjectService,
    pub scenes: SceneService,
    pub members: MemberService,
    pub planning: PlanningService,
    pub documents: DocumentService,
}

impl AppState {
    pub async fn open(db_path: &Path, files_root: &Path) -> CoreResult<Self> {
        let store = Arc::new(SqliteStore::connect(db_path).await?);
        let files = Arc::new(LocalFileStorage::new(files_root));
        Ok(Self {
            projects: ProjectService::new(store.clone()),
            scenes: SceneService::new(store.clone()),
            members: MemberService::new(store.clone()),
            planning: PlanningService::new(store.clone(), store.clone(), store.clone()),
            documents: DocumentService::new(store, files),
        })
    }
}
