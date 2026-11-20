mod commands;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Base SQLite et documents dans le dossier de données de l'application.
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let state = tauri::async_runtime::block_on(AppState::open(
                &dir.join("cineplan.db"),
                &dir.join("documents"),
            ))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // projets
            commands::list_projects,
            commands::get_project,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            // scènes
            commands::list_scenes,
            commands::create_scene,
            commands::update_scene,
            commands::delete_scene,
            // équipe
            commands::list_members,
            commands::create_member,
            commands::update_member,
            commands::delete_member,
            // planning
            commands::list_days,
            commands::get_day_view,
            commands::create_day,
            commands::update_day,
            commands::delete_day,
            commands::add_scene_item,
            commands::add_break_item,
            commands::update_item,
            commands::reorder_items,
            commands::remove_item,
            // documents
            commands::list_folders,
            commands::create_folder,
            commands::rename_folder,
            commands::delete_folder,
            commands::list_documents,
            commands::import_document,
            commands::update_document,
            commands::delete_document,
            commands::read_document,
            commands::export_document,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de CinePlan");
}
