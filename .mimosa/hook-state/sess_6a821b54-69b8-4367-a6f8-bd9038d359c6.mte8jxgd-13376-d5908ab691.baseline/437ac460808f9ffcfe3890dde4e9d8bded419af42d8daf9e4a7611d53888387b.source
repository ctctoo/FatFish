use tauri::State;

use crate::models::todo::{Todo, TodoInput};
use crate::repository::todo_repository;
use crate::Db;

type CmdResult<T> = Result<T, String>;

fn validate(input: &TodoInput) -> Result<(), String> {
    if input.title.trim().is_empty() {
        return Err("任务内容不能为空".into());
    }
    Ok(())
}

#[tauri::command]
pub fn list_todos(db: State<Db>) -> CmdResult<Vec<Todo>> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    todo_repository::list(&conn).map_err(|e| format!("数据库操作失败: {e}"))
}

#[tauri::command]
pub fn create_todo(db: State<Db>, input: TodoInput) -> CmdResult<Todo> {
    validate(&input)?;
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    todo_repository::insert(&conn, &input).map_err(|e| format!("数据库操作失败: {e}"))
}

#[tauri::command]
pub fn update_todo(db: State<Db>, id: String, input: TodoInput) -> CmdResult<()> {
    validate(&input)?;
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    todo_repository::update(&conn, &id, &input).map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn toggle_todo(db: State<Db>, id: String, done: bool) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    todo_repository::set_done(&conn, &id, done).map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_todo(db: State<Db>, id: String) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    todo_repository::delete(&conn, &id).map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(())
}
