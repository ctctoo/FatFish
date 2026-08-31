use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::models::activity::Activity;
use crate::repository::new_id;

/// 记录一条项目动态。失败静默（动态属附加信息，不能影响主流程）。
pub fn log(conn: &Connection, project_id: &str, kind: &str, message: &str) {
    let _ = conn.execute(
        "INSERT INTO activities (id, project_id, kind, message, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![new_id(), project_id, kind, message, crate::database::now()],
    );
}

pub fn list_for_project(conn: &Connection, project_id: &str, limit: i64) -> AppResult<Vec<Activity>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, kind, message, created_at
         FROM activities WHERE project_id = ?1
         ORDER BY created_at DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![project_id, limit], |row| {
        Ok(Activity {
            id: row.get(0)?,
            project_id: row.get(1)?,
            kind: row.get(2)?,
            message: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
