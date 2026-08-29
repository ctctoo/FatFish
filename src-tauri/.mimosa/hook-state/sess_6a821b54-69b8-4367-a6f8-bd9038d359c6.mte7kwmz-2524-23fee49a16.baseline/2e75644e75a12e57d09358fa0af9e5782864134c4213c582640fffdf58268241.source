use tauri::State;

use crate::Db;
use crate::models::collection::{Collection, CollectionInput};
use crate::repository::collection_repository;

type CmdResult<T> = Result<T, String>;

#[tauri::command]
pub fn list_collections(db: State<Db>) -> CmdResult<Vec<Collection>> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    collection_repository::list(&conn).map_err(|e| format!("数据库操作失败: {e}"))
}

#[tauri::command]
pub fn create_collection(db: State<Db>, input: CollectionInput) -> CmdResult<Collection> {
    if input.name.trim().is_empty() {
        return Err("集合名称不能为空".into());
    }
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    collection_repository::insert(&conn, &input).map_err(|e| format!("数据库操作失败: {e}"))
}

#[tauri::command]
pub fn update_collection(db: State<Db>, id: String, input: CollectionInput) -> CmdResult<()> {
    if input.name.trim().is_empty() {
        return Err("集合名称不能为空".into());
    }
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    collection_repository::update(&conn, &id, &input).map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_collection(db: State<Db>, id: String) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    collection_repository::delete(&conn, &id).map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn set_project_collections(
    db: State<Db>,
    project_id: String,
    collection_ids: Vec<String>,
) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    collection_repository::set_project_collections(&conn, &project_id, &collection_ids)
        .map_err(|e| format!("数据库操作失败: {e}"))
}
