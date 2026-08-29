use rusqlite::{params, Connection};

use crate::models::todo::{Todo, TodoInput};

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Todo>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.done, t.project_id, t.due_date,
                p.name, p.status
         FROM todos t
         LEFT JOIN projects p ON p.id = t.project_id
         ORDER BY t.done ASC, t.sort_order ASC, t.created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let done: i64 = row.get(2)?;
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            done: done != 0,
            project_id: row.get(3)?,
            due_date: row.get(4)?,
            project_name: row.get(5)?,
            project_status: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn insert(conn: &Connection, input: &TodoInput) -> rusqlite::Result<Todo> {
    let id = new_id();
    let order: i64 = conn.query_row(
        "SELECT COALESCE(MIN(sort_order), 0) - 1 FROM todos",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO todos (id, title, done, project_id, due_date, sort_order, created_at)
         VALUES (?1, ?2, 0, ?3, ?4, ?5, ?6)",
        params![
            id,
            input.title.trim(),
            input.project_id,
            input.due_date,
            order,
            crate::db::sqlite::now()
        ],
    )?;
    Ok(Todo {
        id,
        title: input.title.trim().to_string(),
        done: false,
        project_id: input.project_id.clone(),
        due_date: input.due_date.clone(),
        project_name: None,
        project_status: None,
    })
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &TodoInput,
) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE todos SET title = ?2, project_id = ?3, due_date = ?4 WHERE id = ?1",
        params![id, input.title.trim(), input.project_id, input.due_date],
    )
}

pub fn set_done(conn: &Connection, id: &str, done: bool) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE todos SET done = ?2 WHERE id = ?1",
        params![id, done as i64],
    )
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM todos WHERE id = ?1", params![id])
}
