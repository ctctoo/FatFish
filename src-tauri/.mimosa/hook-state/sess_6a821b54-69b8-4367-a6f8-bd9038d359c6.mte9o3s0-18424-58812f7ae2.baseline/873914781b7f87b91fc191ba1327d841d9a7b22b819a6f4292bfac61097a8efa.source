use rusqlite::{params, params_from_iter, Connection, Row};

use crate::models::git::GitInfo;
use crate::models::project::{Project, ProjectFilter, ProjectInput};
use crate::repository::{collection_repository, link_repository, tag_repository};

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn normalize_path(path: &str) -> String {
    path.trim().trim_end_matches(['\\', '/']).to_string()
}

/// 组装完整 Project（含标签 / 集合 / 链接 / Git 信息）
fn assemble_project(conn: &Connection, row: &Row) -> rusqlite::Result<Project> {
    let id: String = row.get("id")?;
    let favorite: i64 = row.get("favorite")?;
    let last_opened_at: Option<String> = row.get("last_opened_at")?;
    let git_info = load_git_info(conn, &id)?;
    Ok(Project {
        tags: tag_repository::list_for_project(conn, &id)?,
        collections: collection_repository::list_for_project(conn, &id)?,
        links: link_repository::list_for_project(conn, &id)?,
        git_info,
        id,
        name: row.get("name")?,
        path: row.get("path")?,
        description: row.get("description")?,
        status: row.get("status")?,
        favorite: favorite != 0,
        cover_emoji: row.get("cover_emoji")?,
        cover_color: row.get("cover_color")?,
        notes: row.get("notes")?,
        language: row.get("language")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        last_opened_at: last_opened_at.unwrap_or_default(),
    })
}

fn load_git_info(conn: &Connection, project_id: &str) -> rusqlite::Result<Option<GitInfo>> {
    let mut stmt = conn.prepare(
        "SELECT remote_url, branch, commit_hash, commit_message, commit_time, is_dirty
         FROM git_info WHERE project_id = ?1",
    )?;
    let mut rows = stmt.query_map(params![project_id], |row| {
        let is_dirty: Option<i64> = row.get(5)?;
        Ok(GitInfo {
            remote_url: row.get(0)?,
            branch: row.get(1)?,
            commit_hash: row.get(2)?,
            commit_message: row.get(3)?,
            commit_time: row.get(4)?,
            is_dirty: is_dirty.map(|v| v != 0),
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

const PROJECT_COLUMNS: &str =
    "id, name, path, description, status, favorite, cover_emoji, cover_color, notes, language, created_at, updated_at, last_opened_at";

pub fn exists_with_path(conn: &Connection, path: &str) -> rusqlite::Result<bool> {
    let normalized = normalize_path(path);
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM projects WHERE lower(path) = lower(?1)",
        params![normalized],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn insert(conn: &Connection, input: &ProjectInput) -> rusqlite::Result<Project> {
    let id = new_id();
    let now = crate::db::sqlite::now();
    conn.execute(
        "INSERT INTO projects (id, name, path, description, status, favorite, cover_emoji, cover_color, notes, language, created_at, updated_at, last_opened_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8, NULL, ?9, ?9, NULL)",
        params![
            id,
            input.name.trim(),
            normalize_path(&input.path),
            input.description,
            input.status.as_deref().unwrap_or("IN_PROGRESS"),
            input.cover_emoji,
            input.cover_color,
            input.notes,
            now,
        ],
    )?;
    get(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update(conn: &Connection, id: &str, input: &ProjectInput) -> rusqlite::Result<Project> {
    let now = crate::db::sqlite::now();
    conn.execute(
        "UPDATE projects SET name = ?2, path = ?3, description = ?4, status = ?5, cover_emoji = ?6, cover_color = ?7, notes = ?8, updated_at = ?9
         WHERE id = ?1",
        params![
            id,
            input.name.trim(),
            normalize_path(&input.path),
            input.description,
            input.status.as_deref().unwrap_or("IN_PROGRESS"),
            input.cover_emoji,
            input.cover_color,
            input.notes,
            now,
        ],
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Project>> {
    let sql = format!("SELECT {PROJECT_COLUMNS} FROM projects WHERE id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map(params![id], |row| assemble_project(conn, row))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn set_favorite(conn: &Connection, id: &str, favorite: bool) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE projects SET favorite = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, favorite as i64, crate::db::sqlite::now()],
    )
}

pub fn mark_opened(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE projects SET last_opened_at = ?2 WHERE id = ?1",
        params![id, crate::db::sqlite::now()],
    )
}

/// 按组合条件查询：关键词 / 状态 / 收藏 / 标签 / 集合 / 最近打开 / 排序（简单 AND 组合）。
/// 关键词覆盖：名称、简介、备注、路径、标签、集合、链接标题与 URL、语言。
pub fn list(conn: &Connection, filter: &ProjectFilter) -> rusqlite::Result<Vec<Project>> {
    let mut conditions: Vec<String> = Vec::new();
    let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let next_param = |values: &mut Vec<Box<dyn rusqlite::ToSql>>, v: Box<dyn rusqlite::ToSql>| -> String {
        values.push(v);
        format!("?{}", values.len())
    };

    if let Some(query) = filter.query.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        let like = format!("%{}%", query.to_lowercase());
        let p = next_param(&mut bind_values, Box::new(like.clone()));
        conditions.push(format!(
            "(lower(name) LIKE {p} OR lower(coalesce(description, '')) LIKE {p} OR lower(coalesce(notes, '')) LIKE {p}
             OR lower(path) LIKE {p} OR lower(coalesce(language, '')) LIKE {p}
             OR EXISTS (SELECT 1 FROM project_tags pt JOIN tags t ON t.id = pt.tag_id
                        WHERE pt.project_id = projects.id AND lower(t.name) LIKE {p})
             OR EXISTS (SELECT 1 FROM project_collections pcc JOIN collections c ON c.id = pcc.collection_id
                        WHERE pcc.project_id = projects.id AND lower(c.name) LIKE {p})
             OR EXISTS (SELECT 1 FROM links l WHERE l.project_id = projects.id
                        AND (lower(l.title) LIKE {p} OR lower(l.url) LIKE {p})))"
        ));
    }

    if let Some(status) = filter.status.as_deref().filter(|s| !s.is_empty()) {
        let p = next_param(&mut bind_values, Box::new(status.to_string()));
        conditions.push(format!("status = {p}"));
    }

    if let Some(favorite) = filter.favorite {
        let p = next_param(&mut bind_values, Box::new(favorite as i64));
        conditions.push(format!("favorite = {p}"));
    }

    if let Some(tag_id) = filter.tag_id.as_deref().filter(|t| !t.is_empty()) {
        let p = next_param(&mut bind_values, Box::new(tag_id.to_string()));
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM project_tags pt WHERE pt.project_id = projects.id AND pt.tag_id = {p})"
        ));
    }

    if let Some(collection_id) = filter.collection_id.as_deref().filter(|c| !c.is_empty()) {
        let p = next_param(&mut bind_values, Box::new(collection_id.to_string()));
        conditions.push(format!(
            "EXISTS (SELECT 1 FROM project_collections pcc WHERE pcc.project_id = projects.id AND pcc.collection_id = {p})"
        ));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let order_clause = match filter.sort.as_deref() {
        Some("name") => "ORDER BY name COLLATE NOCASE",
        Some("opened") => "ORDER BY (last_opened_at IS NULL) ASC, last_opened_at DESC",
        Some("created") => "ORDER BY created_at DESC",
        _ => "ORDER BY (favorite = 1) DESC, updated_at DESC",
    };
    let limit_clause = if filter.recent.unwrap_or(false) {
        "LIMIT 8"
    } else {
        ""
    };

    let sql = format!("SELECT {PROJECT_COLUMNS} FROM projects {where_clause} {order_clause} {limit_clause}");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params_from_iter(bind_values.iter().map(|b| b.as_ref())),
        |row| assemble_project(conn, row),
    )?;
    rows.collect()
}
