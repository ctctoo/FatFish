use project_hub::database::{LATEST_SCHEMA_VERSION, migrate};
use rusqlite::Connection;

fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [name],
        |row| row.get(0),
    )
    .expect("table existence query should work")
}

#[test]
fn migrates_empty_database_to_latest_schema() {
    let mut conn = Connection::open_in_memory().expect("open in-memory database");

    migrate(&mut conn).expect("migrate empty database");

    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))
        .expect("schema version");
    assert_eq!(version, LATEST_SCHEMA_VERSION);

    for table in [
        "projects",
        "todos",
        "tags",
        "project_tags",
        "collections",
        "project_collections",
        "links",
        "attachments",
        "screenshots",
        "custom_fields",
        "project_custom_fields",
        "settings",
        "project_search",
    ] {
        assert!(table_exists(&conn, table), "missing table {table}");
    }
}

#[test]
fn upgrades_legacy_database_without_changing_user_data() {
    let mut conn = Connection::open_in_memory().expect("open in-memory database");
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'IN_PROGRESS',
            favorite INTEGER NOT NULL DEFAULT 0,
            github_url TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_opened_at TEXT
        );
        CREATE TABLE tags (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, color TEXT);
        CREATE TABLE project_tags (
            project_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (project_id, tag_id),
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );
        CREATE TABLE todos (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0,
            project_id TEXT,
            due_date TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL
        );
        INSERT INTO projects
            (id, name, path, description, status, favorite, github_url, created_at, updated_at)
        VALUES
            ('project-1', 'Thesis', 'D:\Research\Thesis', 'Research notes', 'DEVELOPING', 1,
             'https://github.com/example/thesis', '2026-01-01', '2026-02-01');
        INSERT INTO tags (id, name, color) VALUES ('tag-1', 'Research', '#61758a');
        INSERT INTO project_tags (project_id, tag_id) VALUES ('project-1', 'tag-1');
        INSERT INTO todos (id, title, project_id, created_at)
        VALUES ('todo-1', 'Finish chapter', 'project-1', '2026-02-02');
        "#,
    )
    .expect("create legacy schema");

    migrate(&mut conn).expect("upgrade legacy database");

    let project: (String, String, String, i64) = conn
        .query_row(
            "SELECT name, path, status, favorite FROM projects WHERE id = 'project-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("preserved project");
    assert_eq!(project.0, "Thesis");
    assert_eq!(project.1, r"D:\Research\Thesis");
    assert_eq!(project.2, "IN_PROGRESS");
    assert_eq!(project.3, 1);

    let tag_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM project_tags WHERE project_id = 'project-1' AND tag_id = 'tag-1'",
            [],
            |row| row.get(0),
        )
        .expect("preserved tag relation");
    assert_eq!(tag_count, 1);

    let todo_project: String = conn
        .query_row(
            "SELECT project_id FROM todos WHERE id = 'todo-1'",
            [],
            |row| row.get(0),
        )
        .expect("preserved todo relation");
    assert_eq!(todo_project, "project-1");

    let github_links: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM links WHERE project_id = 'project-1' AND link_type = 'github'",
            [],
            |row| row.get(0),
        )
        .expect("migrated github link");
    assert_eq!(github_links, 1);
}

#[test]
fn running_migrations_twice_is_idempotent() {
    let mut conn = Connection::open_in_memory().expect("open in-memory database");

    migrate(&mut conn).expect("first migration");
    conn.execute(
        "INSERT INTO projects
         (id, name, path, status, favorite, created_at, updated_at)
         VALUES ('project-1', 'Travel', 'D:\\Plans\\Travel', 'PLANNED', 0, '2026-01-01', '2026-01-01')",
        [],
    )
    .expect("insert project");
    migrate(&mut conn).expect("second migration");

    let project_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
        .expect("count projects");
    assert_eq!(project_count, 1);
}
