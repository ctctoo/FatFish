use rusqlite::{params, Connection};

use crate::models::tag::{Tag, TagInput};

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name COLLATE NOCASE")?;
    let rows = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn list_for_project(conn: &Connection, project_id: &str) -> rusqlite::Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color
         FROM tags t
         JOIN project_tags pt ON pt.tag_id = t.id
         WHERE pt.project_id = ?1
         ORDER BY t.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn insert(conn: &Connection, input: &TagInput) -> rusqlite::Result<Tag> {
    let id = new_id();
    conn.execute(
        "INSERT INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
        params![id, input.name.trim(), input.color],
    )?;
    Ok(Tag {
        id,
        name: input.name.trim().to_string(),
        color: input.color.clone(),
    })
}

pub fn update(conn: &Connection, id: &str, input: &TagInput) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE tags SET name = ?2, color = ?3 WHERE id = ?1",
        params![id, input.name.trim(), input.color],
    )
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
}

/// 全量替换项目的标签集合
pub fn set_project_tags(conn: &Connection, project_id: &str, tag_ids: &[String]) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM project_tags WHERE project_id = ?1", params![project_id])?;
    for tag_id in tag_ids {
        tx.execute(
            "INSERT OR IGNORE INTO project_tags (project_id, tag_id) VALUES (?1, ?2)",
            params![project_id, tag_id],
        )?;
    }
    tx.commit()
}
