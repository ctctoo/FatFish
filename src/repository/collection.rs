use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::models::collection::{Collection, CollectionInput};
use crate::repository::new_id;

pub fn list(conn: &Connection) -> AppResult<Vec<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name,
                (SELECT COUNT(*) FROM project_collections pc WHERE pc.collection_id = c.id) AS cnt
         FROM collections c ORDER BY c.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |row| {
        let _count: i64 = row.get(2)?;
        Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_for_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name
         FROM collections c
         JOIN project_collections pc ON pc.collection_id = c.id
         WHERE pc.project_id = ?1
         ORDER BY c.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn insert(conn: &Connection, input: &CollectionInput) -> AppResult<Collection> {
    let id = new_id();
    conn.execute(
        "INSERT INTO collections (id, name) VALUES (?1, ?2)",
        params![id, input.name.trim()],
    )?;
    Ok(Collection {
        id,
        name: input.name.trim().to_string(),
    })
}

pub fn update(conn: &Connection, id: &str, input: &CollectionInput) -> AppResult<usize> {
    let n = conn.execute(
        "UPDATE collections SET name = ?2 WHERE id = ?1",
        params![id, input.name.trim()],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(format!("collection {id}")));
    }
    Ok(n)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<usize> {
    Ok(conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?)
}

/// 全量替换项目的集合归属
pub fn set_project_collections(
    conn: &Connection,
    project_id: &str,
    collection_ids: &[String],
) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM project_collections WHERE project_id = ?1",
        params![project_id],
    )?;
    for collection_id in collection_ids {
        tx.execute(
            "INSERT OR IGNORE INTO project_collections (project_id, collection_id) VALUES (?1, ?2)",
            params![project_id, collection_id],
        )?;
    }
    tx.commit()?;
    Ok(())
}
