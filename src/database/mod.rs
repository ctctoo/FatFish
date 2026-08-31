use std::path::Path;

use rusqlite::{Connection, OptionalExtension};

use crate::error::{AppError, AppResult};

/// 当前 schema 版本。任何结构变化都必须通过 migrations 推进版本。
pub const LATEST_SCHEMA_VERSION: i64 = 5;

/// 打开（必要时创建）数据库：确保父目录存在、启用 WAL 与外键，
/// 并执行版本化 migration 到最新版本。
pub fn open(db_path: &Path) -> AppResult<Connection> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::DataDir(format!(
            "创建数据库目录失败: {} ({e})",
            parent.display()
        )))?;
    }
    let mut conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    let version = migrate(&mut conn)?;
    tracing::debug!(schema_version = version, path = %db_path.display(), "database opened");
    Ok(conn)
}

/// 打开数据库并迁移到最新版本。
///
/// - 空数据库：从 v0 开始按 migrations 顺序执行。
/// - 旧版 FatFish 数据库（无 schema_version 表）：先执行 v1 的基础表，
///   再补充旧代码里运行时 ALTER TABLE 的列和状态归一化。
/// - 已有新版本数据库：仅执行缺少的 migration，保证幂等。
///
/// 整个迁移在单个事务中执行：任何一步失败都会回滚，避免留下半迁移状态。
pub fn migrate(conn: &mut Connection) -> rusqlite::Result<i64> {
    // foreign_keys 必须在事务开始前设置（SQLite 规定该 pragma 在事务内不生效）。
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let tx = conn.transaction()?;
    tx.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
        "#,
    )?;

    let current = current_version(&tx)?;
    if current >= LATEST_SCHEMA_VERSION {
        tx.commit()?;
        return Ok(current);
    }

    if current == 0 {
        run_migration(&tx, 1, include_str!("../../migrations/0001_initial.sql"))?;
    }

    // 兼容旧版 FatFish：旧库可能已有 projects/tags 等表，但没有 schema_version。
    // 此时需要补上缺失列、旧状态值和 github_url 链接迁移。
    migrate_legacy_columns(&tx)?;

    if current <= 1 {
        run_migration(&tx, 2, include_str!("../../migrations/0002_enhancements.sql"))?;
    }
    if current <= 2 {
        run_migration(&tx, 3, include_str!("../../migrations/0003_custom_fields.sql"))?;
    }
    if current <= 3 {
        run_migration(&tx, 4, include_str!("../../migrations/0004_media_tables.sql"))?;
    }
    if current <= 4 {
        run_migration(&tx, 5, include_str!("../../migrations/0005_search_index.sql"))?;
    }

    let version = current_version(&tx)?;
    tx.commit()?;
    Ok(version)
}

fn current_version(conn: &Connection) -> rusqlite::Result<i64> {
    // MAX 在空表上返回一行 NULL，用 COALESCE 兜底为 0。
    conn.query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |row| {
        row.get(0)
    })
}

fn run_migration(conn: &Connection, version: i64, script: &str) -> rusqlite::Result<()> {
    conn.execute_batch(script)?;
    conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
        rusqlite::params![version, now()],
    )?;
    Ok(())
}

fn migrate_legacy_columns(conn: &Connection) -> rusqlite::Result<()> {
    let has_projects = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'projects')",
            [],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(false);
    if !has_projects {
        return Ok(());
    }

    add_column_if_missing(conn, "projects", "cover_emoji", "TEXT")?;
    add_column_if_missing(conn, "projects", "cover_color", "TEXT")?;
    add_column_if_missing(conn, "projects", "notes", "TEXT")?;
    add_column_if_missing(conn, "projects", "language", "TEXT")?;

    // 旧状态 DEVELOPING / MAINTAINING 归一为 IN_PROGRESS
    conn.execute(
        "UPDATE projects SET status = 'IN_PROGRESS' WHERE status IN ('DEVELOPING', 'MAINTAINING')",
        [],
    )?;

    // 旧 github_url 迁移为 links 记录；新库的 projects 没有该列，先探测再迁移。
    // 重复运行不会产生重复记录。
    if has_column(conn, "projects", "github_url")? {
        conn.execute(
            r#"INSERT INTO links (id, project_id, title, url, link_type)
               SELECT lower(hex(randomblob(16))), p.id, 'GitHub', p.github_url, 'github'
               FROM projects p
               WHERE p.github_url IS NOT NULL AND p.github_url != ''
                 AND NOT EXISTS (
                    SELECT 1 FROM links l WHERE l.project_id = p.id AND l.link_type = 'github'
                 )"#,
            [],
        )?;
    }

    Ok(())
}

fn has_column(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    Ok(conn
        .prepare(&format!("PRAGMA table_info({table})"))?
        .query_map([], |row| {
            let name: String = row.get(1)?;
            Ok(name)
        })?
        .filter_map(Result::ok)
        .any(|name| name == column))
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    decl: &str,
) -> rusqlite::Result<()> {
    if !has_column(conn, table, column)? {
        conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl};"))?;
    }
    Ok(())
}

pub fn now() -> String {
    chrono::Local::now().to_rfc3339()
}
