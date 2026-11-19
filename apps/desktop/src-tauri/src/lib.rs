mod commands;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Base SQLite dans le dossier de données de l'application.
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let state = tauri::async_runtime::block_on(AppState::open(&dir.join("cineplan.db")))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_projects,
            commands::get_project,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::list_scenes,
            commands::create_scene,
            commands::update_scene,
            commands::delete_scene,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de CinePlan");
}
