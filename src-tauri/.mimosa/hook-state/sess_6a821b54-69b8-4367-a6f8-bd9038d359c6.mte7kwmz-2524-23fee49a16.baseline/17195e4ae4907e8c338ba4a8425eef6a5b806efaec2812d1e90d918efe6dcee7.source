use tauri::State;

use crate::Db;
use crate::models::project::{Project, ProjectFilter, ProjectInput};
use crate::services::project_service;

type CmdResult<T> = Result<T, String>;

fn with_conn<T>(db: &State<Db>, f: impl FnOnce(&rusqlite::Connection) -> CmdResult<T>) -> CmdResult<T> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    f(&conn)
}

#[tauri::command]
pub fn list_projects(db: State<Db>, filter: Option<ProjectFilter>) -> CmdResult<Vec<Project>> {
    with_conn(&db, |conn| {
        project_service::list(conn, filter.unwrap_or_default())
    })
}

#[tauri::command]
pub fn get_project(db: State<Db>, id: String) -> CmdResult<Project> {
    with_conn(&db, |conn| {
        crate::repository::project_repository::get(conn, &id)
            .map_err(|e| format!("数据库操作失败: {e}"))?
            .ok_or_else(|| "项目不存在".to_string())
    })
}

#[tauri::command]
pub fn create_project(db: State<Db>, input: ProjectInput) -> CmdResult<Project> {
    with_conn(&db, |conn| project_service::create(conn, input))
}

#[tauri::command]
pub fn update_project(db: State<Db>, id: String, input: ProjectInput) -> CmdResult<Project> {
    with_conn(&db, |conn| project_service::update(conn, &id, input))
}

#[tauri::command]
pub fn delete_project(db: State<Db>, id: String) -> CmdResult<()> {
    with_conn(&db, |conn| project_service::delete(conn, &id))
}

#[tauri::command]
pub fn set_favorite(db: State<Db>, id: String, favorite: bool) -> CmdResult<()> {
    with_conn(&db, |conn| {
        crate::repository::project_repository::set_favorite(conn, &id, favorite)
            .map_err(|e| format!("数据库操作失败: {e}"))?;
        Ok(())
    })
}

#[tauri::command]
pub fn mark_opened(db: State<Db>, id: String) -> CmdResult<()> {
    with_conn(&db, |conn| {
        crate::repository::project_repository::mark_opened(conn, &id)
            .map_err(|e| format!("数据库操作失败: {e}"))?;
        Ok(())
    })
}
