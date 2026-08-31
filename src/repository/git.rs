use rusqlite::{params, Connection};

use crate::error::AppResult;
use crate::models::git::GitInfo;

pub fn upsert(conn: &Connection, project_id: &str, info: &GitInfo) -> AppResult<usize> {
    Ok(conn.execute(
        "INSERT INTO git_info (project_id, remote_url, branch, commit_hash, commit_message, commit_time, is_dirty)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(project_id) DO UPDATE SET
            remote_url = excluded.remote_url,
            branch = excluded.branch,
            commit_hash = excluded.commit_hash,
            commit_message = excluded.commit_message,
            commit_time = excluded.commit_time,
            is_dirty = excluded.is_dirty",
        params![
            project_id,
            info.remote_url,
            info.branch,
            info.commit_hash,
            info.commit_message,
            info.commit_time,
            info.is_dirty.map(|b| b as i64),
        ],
    )?)
}

pub fn delete(conn: &Connection, project_id: &str) -> AppResult<usize> {
    Ok(conn.execute("DELETE FROM git_info WHERE project_id = ?1", params![project_id])?)
}
