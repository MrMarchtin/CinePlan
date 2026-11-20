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

// ---------- équipe ----------

use crate::document::{DocumentFile, Folder, StoredFile};
use crate::member::TeamMember;
use crate::planning::{DayItem, ShootingDay};
use std::path::Path;

#[async_trait]
pub trait MemberRepository: Send + Sync {
    /// Membres d'un projet, triés par nom puis prénom.
    async fn list_members(&self, project_id: Uuid) -> CoreResult<Vec<TeamMember>>;
    async fn get_member(&self, id: Uuid) -> CoreResult<TeamMember>;
    async fn insert_member(&self, member: &TeamMember) -> CoreResult<()>;
    /// Même contrat de version que `ProjectRepository::update`.
    async fn update_member(&self, member: &TeamMember) -> CoreResult<()>;
    async fn delete_member(&self, id: Uuid) -> CoreResult<()>;
}

// ---------- planning ----------

#[async_trait]
pub trait PlanningRepository: Send + Sync {
    /// Journées d'un projet, triées par date (membres convoqués inclus).
    async fn list_days(&self, project_id: Uuid) -> CoreResult<Vec<ShootingDay>>;
    async fn get_day(&self, id: Uuid) -> CoreResult<ShootingDay>;
    async fn insert_day(&self, day: &ShootingDay) -> CoreResult<()>;
    /// Même contrat de version ; remplace aussi la liste des membres convoqués.
    async fn update_day(&self, day: &ShootingDay) -> CoreResult<()>;
    async fn delete_day(&self, id: Uuid) -> CoreResult<()>;

    /// Éléments d'une journée, dans l'ordre.
    async fn list_items(&self, day_id: Uuid) -> CoreResult<Vec<DayItem>>;
    async fn get_item(&self, id: Uuid) -> CoreResult<DayItem>;
    async fn insert_item(&self, item: &DayItem) -> CoreResult<()>;
    async fn update_item(&self, item: &DayItem) -> CoreResult<()>;
    async fn delete_item(&self, id: Uuid) -> CoreResult<()>;
    /// Réécrit les positions selon l'ordre donné.
    async fn set_item_order(&self, day_id: Uuid, ordered: &[Uuid]) -> CoreResult<()>;
}

// ---------- documents ----------

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn list_folders(&self, project_id: Uuid) -> CoreResult<Vec<Folder>>;
    async fn get_folder(&self, id: Uuid) -> CoreResult<Folder>;
    async fn insert_folder(&self, folder: &Folder) -> CoreResult<()>;
    async fn update_folder(&self, folder: &Folder) -> CoreResult<()>;
    async fn delete_folder(&self, id: Uuid) -> CoreResult<()>;

    async fn list_documents(&self, project_id: Uuid) -> CoreResult<Vec<DocumentFile>>;
    async fn get_document(&self, id: Uuid) -> CoreResult<DocumentFile>;
    async fn insert_document(&self, doc: &DocumentFile) -> CoreResult<()>;
    async fn update_document(&self, doc: &DocumentFile) -> CoreResult<()>;
    async fn delete_document(&self, id: Uuid) -> CoreResult<()>;
}

/// Stockage des fichiers eux-mêmes (disque local aujourd'hui, cloud demain).
pub trait FileStorage: Send + Sync {
    /// Vérifie le type et la taille, puis copie `source` dans le stockage du projet.
    fn import(&self, project_id: Uuid, source: &Path) -> CoreResult<StoredFile>;
    fn read(&self, project_id: Uuid, stored_name: &str) -> CoreResult<Vec<u8>>;
    fn export(&self, project_id: Uuid, stored_name: &str, dest: &Path) -> CoreResult<()>;
    fn remove(&self, project_id: Uuid, stored_name: &str) -> CoreResult<()>;
}
