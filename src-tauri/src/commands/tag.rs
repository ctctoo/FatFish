use tauri::State;

use crate::Db;
use crate::models::tag::{Tag, TagInput};
use crate::repository::tag_repository;

type CmdResult<T> = Result<T, String>;

fn validate(input: &TagInput) -> Result<(), String> {
    if input.name.trim().is_empty() {
        return Err("标签名称不能为空".into());
    }
    Ok(())
}

#[tauri::command]
pub fn list_tags(db: State<Db>) -> CmdResult<Vec<Tag>> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    tag_repository::list(&conn).map_err(|e| format!("数据库操作失败: {e}"))
}

#[tauri::command]
pub fn create_tag(db: State<Db>, input: TagInput) -> CmdResult<Tag> {
    validate(&input)?;
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    tag_repository::insert(&conn, &input).map_err(|e| format!("数据库操作失败: {e}"))
}

#[tauri::command]
pub fn update_tag(db: State<Db>, id: String, input: TagInput) -> CmdResult<()> {
    validate(&input)?;
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    tag_repository::update(&conn, &id, &input).map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_tag(db: State<Db>, id: String) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    tag_repository::delete(&conn, &id).map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn set_project_tags(db: State<Db>, project_id: String, tag_ids: Vec<String>) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    tag_repository::set_project_tags(&conn, &project_id, &tag_ids)
        .map_err(|e| format!("数据库操作失败: {e}"))
}
