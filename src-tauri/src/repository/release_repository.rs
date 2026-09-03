use rusqlite::{params, Connection, OptionalExtension};

use crate::models::release::Release;

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn row_to_release(row: &rusqlite::Row) -> rusqlite::Result<Release> {
    Ok(Release {
        id: row.get(0)?,
        project_id: row.get(1)?,
        version: row.get(2)?,
        tag_name: row.get(3)?,
        status: row.get(4)?,
        changelog: row.get(5)?,
        release_url: row.get(6)?,
        error_message: row.get(7)?,
        released_at: row.get(8)?,
        created_at: row.get(9)?,
    })
}

const COLUMNS: &str = "id, project_id, version, tag_name, status, changelog, release_url, error_message, released_at, created_at";

/// 创建一条发布记录（初始状态 preparing）。
pub fn insert(
    conn: &Connection,
    project_id: &str,
    version: &str,
    tag_name: &str,
    changelog: Option<&str>,
) -> rusqlite::Result<String> {
    let id = new_id();
    conn.execute(
        &format!(
            "INSERT INTO releases (id, project_id, version, tag_name, status, changelog, created_at)
             VALUES (?1, ?2, ?3, ?4, 'preparing', ?5, ?6)"
        ),
        params![id, project_id, version, tag_name, changelog, crate::db::sqlite::now()],
    )?;
    Ok(id)
}

/// 更新发布状态。
pub fn update_status(
    conn: &Connection,
    id: &str,
    status: &str,
    error_message: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE releases SET status = ?2, error_message = ?3 WHERE id = ?1",
        params![id, status, error_message],
    )?;
    Ok(())
}

/// 发布成功：落定终态信息。
pub fn mark_published(
    conn: &Connection,
    id: &str,
    release_url: &str,
    changelog: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE releases SET status = 'published', release_url = ?2, changelog = COALESCE(?3, changelog), error_message = NULL, released_at = ?4 WHERE id = ?1",
        params![id, release_url, changelog, crate::db::sqlite::now()],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Release>> {
    let sql = format!("SELECT {COLUMNS} FROM releases WHERE id = ?1");
    conn.query_row(&sql, params![id], row_to_release).optional()
}

pub fn list_for_project(conn: &Connection, project_id: &str, limit: i64) -> rusqlite::Result<Vec<Release>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM releases WHERE project_id = ?1 ORDER BY created_at DESC LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id, limit], row_to_release)?;
    rows.collect()
}

/// 项目最近一次发布记录（用于向导建议版本号）。
pub fn latest_for_project(conn: &Connection, project_id: &str) -> rusqlite::Result<Option<Release>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM releases WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 1"
    );
    conn.query_row(&sql, params![project_id], row_to_release)
        .optional()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE projects (id TEXT PRIMARY KEY, name TEXT NOT NULL);
             CREATE TABLE releases (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                version TEXT NOT NULL,
                tag_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'preparing',
                changelog TEXT,
                release_url TEXT,
                error_message TEXT,
                released_at TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
             );",
        )
        .unwrap();
        conn.execute("INSERT INTO projects VALUES ('p1', 'demo')", []).unwrap();
        conn
    }

    #[test]
    fn insert_update_list_roundtrip() {
        let conn = setup();
        let id = insert(&conn, "p1", "1.0.0", "v1.0.0", Some("changelog")).unwrap();

        let rec = get(&conn, &id).unwrap().unwrap();
        assert_eq!(rec.status, "preparing");
        assert_eq!(rec.version, "1.0.0");

        update_status(&conn, &id, "tag_pushed", None).unwrap();
        mark_published(&conn, &id, "https://github.com/a/b/releases/v1.0.0", None).unwrap();

        let list = list_for_project(&conn, "p1", 10).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].status, "published");
        assert!(list[0].release_url.as_deref().unwrap().starts_with("https://"));
        assert_eq!(list[0].changelog.as_deref(), Some("changelog"));

        let latest = latest_for_project(&conn, "p1").unwrap().unwrap();
        assert_eq!(latest.id, id);
        assert!(latest_for_project(&conn, "missing").unwrap().is_none());
    }

    #[test]
    fn failure_keeps_error_message() {
        let conn = setup();
        let id = insert(&conn, "p1", "0.2.0", "v0.2.0", None).unwrap();
        update_status(&conn, &id, "failed", Some("push rejected")).unwrap();
        let rec = get(&conn, &id).unwrap().unwrap();
        assert_eq!(rec.status, "failed");
        assert_eq!(rec.error_message.as_deref(), Some("push rejected"));
    }
}
