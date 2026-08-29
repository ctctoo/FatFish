use tauri::State;

use crate::Db;
use crate::models::project::Project;
use crate::services::project_service;

type CmdResult<T> = Result<T, String>;

#[tauri::command]
pub fn refresh_git_info(db: State<Db>, project_id: String) -> CmdResult<Project> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    project_service::refresh_git_info(&conn, &project_id)
}
