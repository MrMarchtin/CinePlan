use crate::error::CoreResult;
use crate::project::Project;
use crate::scene::Scene;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Stockage des projets. Implémenté par `cineplan-database` (SQLite aujourd'hui,
/// PostgreSQL ou client API demain).
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Projets non supprimés, du plus récemment modifié au plus ancien.
    async fn list(&self) -> CoreResult<Vec<Project>>;
    async fn get(&self, id: Uuid) -> CoreResult<Project>;
    async fn insert(&self, project: &Project) -> CoreResult<()>;
    /// Enregistre `project` dont `version` a déjà été incrémentée : l'écriture
    /// n'a lieu que si la version stockée vaut `project.version - 1`,
    /// sinon `CoreError::Conflict`.
    async fn update(&self, project: &Project) -> CoreResult<()>;
    /// Suppression douce (corbeille) : le projet disparaît des listes.
    async fn soft_delete(&self, id: Uuid, at: DateTime<Utc>) -> CoreResult<()>;
}

#[async_trait]
pub trait SceneRepository: Send + Sync {
    /// Scènes d'un projet, triées par numéro.
    async fn list_by_project(&self, project_id: Uuid) -> CoreResult<Vec<Scene>>;
    async fn get(&self, id: Uuid) -> CoreResult<Scene>;
    async fn insert(&self, scene: &Scene) -> CoreResult<()>;
    /// Même contrat de version que `ProjectRepository::update`.
    async fn update(&self, scene: &Scene) -> CoreResult<()>;
    async fn delete(&self, id: Uuid) -> CoreResult<()>;
}
