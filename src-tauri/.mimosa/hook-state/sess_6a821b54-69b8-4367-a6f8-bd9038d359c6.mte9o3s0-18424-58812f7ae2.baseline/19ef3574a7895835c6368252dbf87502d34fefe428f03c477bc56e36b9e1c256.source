use tauri::State;

use crate::models::activity::Activity;
use crate::repository::activity_repository;
use crate::Db;

type CmdResult<T> = Result<T, String>;

#[tauri::command]
pub fn list_activities(db: State<Db>, project_id: String) -> CmdResult<Vec<Activity>> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    activity_repository::list_for_project(&conn, &project_id, 50)
        .map_err(|e| format!("数据库操作失败: {e}"))
}
