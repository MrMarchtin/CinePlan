//! Commandes Tauri : fines couches entre l'interface Svelte et les services
//! métier. Aucune règle métier ici.

use crate::state::AppState;
use cineplan_core::{
    CoreError, NewProject, NewScene, Project, ProjectUpdate, Scene, SceneUpdate,
};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

/// Erreur renvoyée à l'interface : `kind` permet de réagir (ex. `conflict`),
/// `message` est affichable tel quel.
#[derive(Debug, Serialize)]
pub struct CommandError {
    kind: &'static str,
    message: String,
}

impl From<CoreError> for CommandError {
    fn from(e: CoreError) -> Self {
        Self {
            kind: e.kind(),
            message: e.to_string(),
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

// ---------- projets ----------

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> CommandResult<Vec<Project>> {
    Ok(state.projects.list().await?)
}

#[tauri::command]
pub async fn get_project(state: State<'_, AppState>, id: Uuid) -> CommandResult<Project> {
    Ok(state.projects.get(id).await?)
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    input: NewProject,
) -> CommandResult<Project> {
    Ok(state.projects.create(input).await?)
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    input: ProjectUpdate,
) -> CommandResult<Project> {
    Ok(state.projects.update(input).await?)
}

#[tauri::command]
pub async fn delete_project(state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    Ok(state.projects.delete(id).await?)
}

// ---------- scènes ----------

#[tauri::command]
pub async fn list_scenes(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> CommandResult<Vec<Scene>> {
    Ok(state.scenes.list(project_id).await?)
}

#[tauri::command]
pub async fn create_scene(state: State<'_, AppState>, input: NewScene) -> CommandResult<Scene> {
    Ok(state.scenes.create(input).await?)
}

#[tauri::command]
pub async fn update_scene(
    state: State<'_, AppState>,
    input: SceneUpdate,
) -> CommandResult<Scene> {
    Ok(state.scenes.update(input).await?)
}

#[tauri::command]
pub async fn delete_scene(state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    Ok(state.scenes.delete(id).await?)
}
