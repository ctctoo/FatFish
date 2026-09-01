mod commands;
mod db;
mod models;
mod repository;
mod services;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

pub struct Db(pub Mutex<Connection>);

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&dir)?;
            let conn = db::sqlite::init_connection(&dir.join("fatfish.db"))?;
            app.manage(Db(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::list_projects,
            commands::project::get_project,
            commands::project::create_project,
            commands::project::update_project,
            commands::project::delete_project,
            commands::project::set_favorite,
            commands::project::mark_opened,
            commands::tag::list_tags,
            commands::tag::create_tag,
            commands::tag::update_tag,
            commands::tag::delete_tag,
            commands::tag::set_project_tags,
            commands::collection::list_collections,
            commands::collection::create_collection,
            commands::collection::update_collection,
            commands::collection::delete_collection,
            commands::collection::set_project_collections,
            commands::link::add_link,
            commands::link::update_link,
            commands::link::delete_link,
            commands::todo::list_todos,
            commands::todo::create_todo,
            commands::todo::update_todo,
            commands::todo::toggle_todo,
            commands::todo::delete_todo,
            commands::activity::list_activities,
            commands::scanner::scan_directory,
            commands::scanner::import_projects,
            commands::git::refresh_git_info,
            commands::github::github_login_start,
            commands::github::github_login_poll,
            commands::github::github_status,
            commands::github::github_logout,
            commands::github::github_list_repos,
            commands::system::open_folder,
            commands::system::open_terminal,
            commands::update::check_for_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
