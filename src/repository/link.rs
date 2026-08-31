use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::models::link::{Link, LinkInput};
use crate::repository::new_id;

pub fn list_for_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Link>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, url, link_type FROM links WHERE project_id = ?1 ORDER BY sort_order ASC, rowid",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(Link {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            url: row.get(3)?,
            link_type: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn insert(conn: &Connection, project_id: &str, input: &LinkInput) -> AppResult<Link> {
    let id = new_id();
    conn.execute(
        "INSERT INTO links (id, project_id, title, url, link_type) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, project_id, input.title.trim(), input.url.trim(), input.link_type],
    )?;
    Ok(Link {
        id,
        project_id: project_id.to_string(),
        title: input.title.trim().to_string(),
        url: input.url.trim().to_string(),
        link_type: input.link_type.clone(),
    })
}

pub fn update(conn: &Connection, id: &str, input: &LinkInput) -> AppResult<usize> {
    let n = conn.execute(
        "UPDATE links SET title = ?2, url = ?3, link_type = ?4 WHERE id = ?1",
        params![id, input.title.trim(), input.url.trim(), input.link_type],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(format!("link {id}")));
    }
    Ok(n)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<usize> {
    Ok(conn.execute("DELETE FROM links WHERE id = ?1", params![id])?)
}
