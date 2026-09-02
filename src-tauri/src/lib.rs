mod commands;
mod db;
mod mcp;
mod models;
mod repository;
mod services;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

pub struct Db(pub Mutex<Connection>);

/// 应用数据目录（MCP 模式下无法通过 app handle 获取，这里独立解析）。
/// 与 Tauri 的 `app_data_dir()` 保持一致（`dirs::data_dir()/<bundle_identifier>`），
/// 否则 MCP 模式会读到与 GUI 不同的数据库。
fn mcp_data_dir() -> Option<std::path::PathBuf> {
    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let identifier = context.config().identifier.clone();
    directories::BaseDirs::new().map(|dirs| dirs.data_dir().join(identifier))
}

/// MCP 模式：`fatfish --mcp` 时不启动 GUI，仅在 stdio 上运行 MCP 服务，
/// 供外部 Agent 作为子进程拉起。MCP 模式会尊重设置中的开关。
fn run_mcp_mode() {
    let Some(dir) = mcp_data_dir() else { return };
    let db_path = dir.join("fatfish.db");
    if !db_path.exists() {
        eprintln!("FatFish 数据库不存在，请先启动 FatFish 应用完成初始化。");
        return;
    }
    // 开关关闭时不启动服务（stdio 立即关闭，客户端得到空服务）
    let enabled = db::sqlite::init_connection(&db_path)
        .and_then(|conn| crate::repository::settings_repository::get_bool(&conn, "mcp_enabled"))
        .unwrap_or(false);
    if !enabled {
        eprintln!("FatFish MCP 未启用，请在 FatFish 设置中开启。");
        return;
    }
    mcp::run_mcp_server(db_path);
}

pub fn run() {
    // `--mcp` 参数：作为 MCP 服务器运行（无 GUI）
    if std::env::args().any(|a| a == "--mcp") {
        run_mcp_mode();
        return;
    }

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
            commands::mcp::mcp_status,
            commands::mcp::set_mcp_enabled,
            commands::mcp::configure_mcp_agent,
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
