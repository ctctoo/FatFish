use std::fs;

use project_hub::app::paths::Paths;
use project_hub::database::LATEST_SCHEMA_VERSION;
use rusqlite::Connection;

fn legacy_db(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let conn = Connection::open(path).unwrap();
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
        CREATE TABLE collections (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE);
        CREATE TABLE project_collections (
            project_id TEXT NOT NULL,
            collection_id TEXT NOT NULL,
            PRIMARY KEY (project_id, collection_id),
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY(collection_id) REFERENCES collections(id) ON DELETE CASCADE
        );
        CREATE TABLE links (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            link_type TEXT,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
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
        CREATE TABLE activities (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );
        CREATE TABLE git_info (
            project_id TEXT PRIMARY KEY,
            remote_url TEXT,
            branch TEXT,
            commit_hash TEXT,
            commit_message TEXT,
            commit_time TEXT,
            is_dirty INTEGER DEFAULT 0,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
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
        INSERT INTO git_info (project_id, branch, is_dirty)
        VALUES ('project-1', 'main', 1);
        "#,
    )
    .unwrap();
}

#[test]
fn paths_layout_matches_plan() {
    let root = std::env::temp_dir().join(format!("ph-paths-{}", uuid::Uuid::new_v4()));
    let paths = Paths::with_root(root.clone());
    assert_eq!(paths.data_dir(), root.join("data"));
    assert_eq!(paths.db_path(), root.join("data").join("projecthub.db"));
    assert_eq!(paths.attachments_dir(), root.join("data").join("attachments"));
    assert_eq!(paths.screenshots_dir(), root.join("data").join("screenshots"));
    assert_eq!(paths.backups_dir(), root.join("data").join("backups"));
    assert_eq!(paths.cache_dir(), root.join("cache"));
    assert_eq!(paths.logs_dir(), root.join("logs"));
    paths.ensure_dirs().unwrap();
    for d in [paths.data_dir(), paths.attachments_dir(), paths.screenshots_dir(), paths.backups_dir(), paths.cache_dir(), paths.logs_dir()] {
        assert!(d.is_dir(), "missing {}", d.display());
    }
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn opens_fresh_db_when_no_legacy_exists() {
    let root = std::env::temp_dir().join(format!("ph-fresh-{}", uuid::Uuid::new_v4()));
    let paths = Paths::with_root(root.clone());

    let conn = paths.open_or_migrate_from(&[]).unwrap();
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, LATEST_SCHEMA_VERSION);
    assert!(paths.db_path().is_file());

    // Windows 上必须关闭连接（含 WAL 句柄）后才能删除目录
    drop(conn);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn adopts_legacy_db_without_moving_or_losing_data() {
    let root = std::env::temp_dir().join(format!("ph-adopt-{}", uuid::Uuid::new_v4()));
    let legacy = root.join("legacy").join("com.fatfish.app").join("fatfish.db");
    legacy_db(&legacy);
    let before = fs::read(&legacy).unwrap();
    let paths = Paths::with_root(root.join("app"));

    let conn = paths.open_or_migrate_from(std::slice::from_ref(&legacy)).unwrap();

    // 新库已创建并迁移到最新版本
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, LATEST_SCHEMA_VERSION);

    // 数据保留：项目、状态归一、GitHub 链接迁移、Tag/Todo/Git 关系
    let project: (String, String, String) = conn
        .query_row(
            "SELECT name, path, status FROM projects WHERE id = 'project-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(project.0, "Thesis");
    assert_eq!(project.1, r"D:\Research\Thesis");
    assert_eq!(project.2, "IN_PROGRESS");
    let tag_rel: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM project_tags WHERE project_id = 'project-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tag_rel, 1);
    let todo_rel: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM todos WHERE project_id = 'project-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(todo_rel, 1);
    let git: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM git_info WHERE project_id = 'project-1' AND branch = 'main'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(git, 1);
    let github: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM links WHERE project_id = 'project-1' AND link_type = 'github'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(github, 1);

    // 迁移来源标记已写入 settings
    let origin: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'migration.origin'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(origin.contains("fatfish.db"));

    // 旧库未被移动或改动（路径不移动 + 字节不变）
    assert!(legacy.is_file());
    assert_eq!(fs::read(&legacy).unwrap(), before);

    // 迁移前快照已生成
    let backups: Vec<_> = fs::read_dir(paths.backups_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("fatfish-legacy-backup-") && n.ends_with(".db"))
        .collect();
    assert_eq!(backups.len(), 1);

    // 再次打开幂等：不重复复制/迁移
    let conn2 = paths.open_or_migrate_from(std::slice::from_ref(&legacy)).unwrap();
    let count: i64 = conn2
        .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);

    drop(conn);
    drop(conn2);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn legacy_candidates_point_to_old_app_dir() {
    let paths = Paths::with_root(std::env::temp_dir());
    let candidates = paths.legacy_candidates();
    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|p| {
        let s = p.to_string_lossy();
        s.contains("com.fatfish.app") && s.ends_with("fatfish.db")
    }));
}
