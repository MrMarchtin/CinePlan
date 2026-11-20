//! Commandes Tauri : fines couches entre l'interface Svelte et les services
//! métier. Aucune règle métier ici.

use crate::state::AppState;
use cineplan_core::{
    CoreError, DayItem, DayUpdate, DayView, DocumentFile, DocumentUpdate, Folder, FolderRename,
    ImportDocument, ItemUpdate, MemberUpdate, NewBreakItem, NewDay, NewFolder, NewMember,
    NewProject, NewScene, NewSceneItem, Project, ProjectUpdate, Scene, SceneUpdate, ShootingDay,
    TeamMember,
};
use serde::Serialize;
use tauri::ipc::Response;
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

// ---------- équipe ----------

#[tauri::command]
pub async fn list_members(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> CommandResult<Vec<TeamMember>> {
    Ok(state.members.list(project_id).await?)
}

#[tauri::command]
pub async fn create_member(
    state: State<'_, AppState>,
    input: NewMember,
) -> CommandResult<TeamMember> {
    Ok(state.members.create(input).await?)
}

#[tauri::command]
pub async fn update_member(
    state: State<'_, AppState>,
    input: MemberUpdate,
) -> CommandResult<TeamMember> {
    Ok(state.members.update(input).await?)
}

#[tauri::command]
pub async fn delete_member(state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    Ok(state.members.delete(id).await?)
}

// ---------- planning ----------

#[tauri::command]
pub async fn list_days(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> CommandResult<Vec<ShootingDay>> {
    Ok(state.planning.list_days(project_id).await?)
}

#[tauri::command]
pub async fn get_day_view(state: State<'_, AppState>, id: Uuid) -> CommandResult<DayView> {
    Ok(state.planning.view(id).await?)
}

#[tauri::command]
pub async fn create_day(state: State<'_, AppState>, input: NewDay) -> CommandResult<ShootingDay> {
    Ok(state.planning.create_day(input).await?)
}

#[tauri::command]
pub async fn update_day(
    state: State<'_, AppState>,
    input: DayUpdate,
) -> CommandResult<ShootingDay> {
    Ok(state.planning.update_day(input).await?)
}

#[tauri::command]
pub async fn delete_day(state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    Ok(state.planning.delete_day(id).await?)
}

#[tauri::command]
pub async fn add_scene_item(
    state: State<'_, AppState>,
    input: NewSceneItem,
) -> CommandResult<DayItem> {
    Ok(state.planning.add_scene(input).await?)
}

#[tauri::command]
pub async fn add_break_item(
    state: State<'_, AppState>,
    input: NewBreakItem,
) -> CommandResult<DayItem> {
    Ok(state.planning.add_break(input).await?)
}

#[tauri::command]
pub async fn update_item(
    state: State<'_, AppState>,
    input: ItemUpdate,
) -> CommandResult<DayItem> {
    Ok(state.planning.update_item(input).await?)
}

#[tauri::command]
pub async fn reorder_items(
    state: State<'_, AppState>,
    day_id: Uuid,
    ordered: Vec<Uuid>,
) -> CommandResult<()> {
    Ok(state.planning.reorder(day_id, ordered).await?)
}

#[tauri::command]
pub async fn remove_item(state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    Ok(state.planning.remove_item(id).await?)
}

// ---------- documents ----------

#[tauri::command]
pub async fn list_folders(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> CommandResult<Vec<Folder>> {
    Ok(state.documents.list_folders(project_id).await?)
}

#[tauri::command]
pub async fn create_folder(state: State<'_, AppState>, input: NewFolder) -> CommandResult<Folder> {
    Ok(state.documents.create_folder(input).await?)
}

#[tauri::command]
pub async fn rename_folder(
    state: State<'_, AppState>,
    input: FolderRename,
) -> CommandResult<Folder> {
    Ok(state.documents.rename_folder(input).await?)
}

#[tauri::command]
pub async fn delete_folder(state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    Ok(state.documents.delete_folder(id).await?)
}

#[tauri::command]
pub async fn list_documents(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> CommandResult<Vec<DocumentFile>> {
    Ok(state.documents.list_documents(project_id).await?)
}

#[tauri::command]
pub async fn import_document(
    state: State<'_, AppState>,
    input: ImportDocument,
) -> CommandResult<DocumentFile> {
    Ok(state.documents.import(input).await?)
}

#[tauri::command]
pub async fn update_document(
    state: State<'_, AppState>,
    input: DocumentUpdate,
) -> CommandResult<DocumentFile> {
    Ok(state.documents.update_document(input).await?)
}

#[tauri::command]
pub async fn delete_document(state: State<'_, AppState>, id: Uuid) -> CommandResult<()> {
    Ok(state.documents.delete_document(id).await?)
}

/// Renvoie le contenu brut du fichier (ArrayBuffer côté interface) pour la visionneuse.
#[tauri::command]
pub async fn read_document(state: State<'_, AppState>, id: Uuid) -> CommandResult<Response> {
    let bytes = state.documents.read(id).await?;
    Ok(Response::new(bytes))
}

#[tauri::command]
pub async fn export_document(
    state: State<'_, AppState>,
    id: Uuid,
    dest: String,
) -> CommandResult<()> {
    Ok(state.documents.export(id, &dest).await?)
}
