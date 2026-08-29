use rusqlite::{params, Connection};

use crate::models::link::{Link, LinkInput};

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn list_for_project(conn: &Connection, project_id: &str) -> rusqlite::Result<Vec<Link>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, url, link_type FROM links WHERE project_id = ?1 ORDER BY rowid",
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
    rows.collect()
}

pub fn insert(conn: &Connection, project_id: &str, input: &LinkInput) -> rusqlite::Result<Link> {
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

pub fn update(conn: &Connection, id: &str, input: &LinkInput) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE links SET title = ?2, url = ?3, link_type = ?4 WHERE id = ?1",
        params![id, input.title.trim(), input.url.trim(), input.link_type],
    )
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM links WHERE id = ?1", params![id])
}
