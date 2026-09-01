use std::path::Path;

use rusqlite::Connection;

use crate::models::link::LinkInput;
use crate::models::project::{Project, ProjectInput, ProjectFilter};
use crate::models::scanned::ScannedProject;
use crate::repository::{activity_repository, git_repository, link_repository, project_repository};
use crate::services::{git_service, scanner_service};

fn status_label(status: &str) -> String {
    match status {
        "IN_PROGRESS" => "进行中".to_string(),
        "PLANNED" => "计划中".to_string(),
        "PAUSED" => "已暂停".to_string(),
        "COMPLETED" => "已完成".to_string(),
        "ARCHIVED" => "已归档".to_string(),
        other => other.to_string(),
    }
}

fn validate(input: &ProjectInput) -> Result<(), String> {
    if input.name.trim().is_empty() {
        return Err("项目名称不能为空".into());
    }
    if input.path.trim().is_empty() {
        return Err("项目路径不能为空".into());
    }
    Ok(())
}

/// 补全可选的开发者信息：语言自动识别 + 只读 Git 信息同步。
/// Git / 语言失败不影响项目本身。
fn enrich_and_persist(conn: &Connection, project: &Project) -> rusqlite::Result<Project> {
    let mut language = project.language.clone();
    if language.is_none() {
        language = scanner_service::detect_language(Path::new(&project.path));
        if language.is_some() {
            conn.execute(
                "UPDATE projects SET language = ?2 WHERE id = ?1",
                rusqlite::params![project.id, language],
            )?;
        }
    }

    let git_info = git_service::collect_git_info(&project.path);
    git_repository::upsert(conn, &project.id, &git_info)?;

    // GitHub remote 自动生成一条链接（已有 GitHub 链接或同 URL 链接时跳过）
    if let Some(remote) = &git_info.remote_url {
        let already = project.links.iter().any(|l| {
            l.link_type.as_deref() == Some("github") || l.url.eq_ignore_ascii_case(remote)
        });
        if !already && remote.contains("github.com") {
            if link_repository::insert(
                conn,
                &project.id,
                &LinkInput {
                    title: "GitHub".into(),
                    url: remote.clone(),
                    link_type: Some("github".into()),
                },
            )
            .is_ok()
            {
                activity_repository::log(conn, &project.id, "link", "自动识别 GitHub 链接");
            }
        }
    }

    project_repository::get(conn, &project.id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn create(conn: &Connection, input: ProjectInput) -> Result<Project, String> {
    validate(&input)?;
    if project_repository::exists_with_path(conn, &input.path).map_err(db_err)? {
        return Err("该路径的项目已存在".into());
    }
    let project = project_repository::insert(conn, &input).map_err(db_err)?;
    activity_repository::log(conn, &project.id, "created", "项目创建");
    enrich_and_persist(conn, &project).map_err(db_err)
}

pub fn update(conn: &Connection, id: &str, input: ProjectInput) -> Result<Project, String> {
    validate(&input)?;
    let existing = project_repository::get(conn, id).map_err(db_err)?.ok_or("项目不存在")?;
    let normalized = input.path.trim().trim_end_matches(['\\', '/']).to_string();
    if !normalized.eq_ignore_ascii_case(&existing.path)
        && project_repository::exists_with_path(conn, &input.path).map_err(db_err)?
    {
        return Err("该路径的项目已存在".into());
    }
    let project = project_repository::update(conn, id, &input).map_err(db_err)?;

    // 记录变更动态（描述 / 备注 / 状态 / 名称）
    if existing.name != project.name {
        activity_repository::log(conn, id, "updated", "重命名项目");
    }
    match (&existing.description, &project.description) {
        (None, Some(_)) => activity_repository::log(conn, id, "updated", "添加项目描述"),
        (Some(a), Some(b)) if a != b => activity_repository::log(conn, id, "updated", "更新项目描述"),
        _ => {}
    }
    match (&existing.notes, &project.notes) {
        (None, Some(_)) => activity_repository::log(conn, id, "updated", "添加备注"),
        (Some(a), Some(b)) if a != b => activity_repository::log(conn, id, "updated", "更新备注"),
        _ => {}
    }
    if existing.status != project.status {
        activity_repository::log(
            conn,
            id,
            "updated",
            &format!("状态改为 {}", status_label(&project.status)),
        );
    }

    enrich_and_persist(conn, &project).map_err(db_err)
}

pub fn refresh_git_info(conn: &Connection, id: &str) -> Result<Project, String> {
    let project = project_repository::get(conn, id).map_err(db_err)?.ok_or("项目不存在")?;
    let info = git_service::collect_git_info(&project.path);
    git_repository::upsert(conn, id, &info).map_err(db_err)?;
    project_repository::get(conn, id).map_err(db_err)?.ok_or_else(|| "项目不存在".into())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), String> {
    // links / project_tags / project_collections / git_info 依赖 ON DELETE CASCADE
    git_repository::delete(conn, id).map_err(db_err)?;
    project_repository::delete(conn, id).map_err(db_err)?;
    Ok(())
}

pub fn list(conn: &Connection, filter: ProjectFilter) -> Result<Vec<Project>, String> {
    project_repository::list(conn, &filter).map_err(db_err)
}

/// 扫描目录并标记哪些已经导入过
pub fn scan_directory(conn: &Connection, root: &str) -> Result<Vec<ScannedProject>, String> {
    let mut scanned = scanner_service::scan_directory(root)?;
    for item in scanned.iter_mut() {
        item.already_imported =
            project_repository::exists_with_path(conn, &item.path).map_err(db_err)?;
    }
    Ok(scanned)
}

/// 批量导入扫描结果（不限于代码项目，普通文件夹也可导入）：
/// 重复路径跳过，单个失败不影响其它项。
/// 传入 collection_id 时，导入的项目自动加入该集合。
pub fn import_scanned(
    conn: &Connection,
    paths: &[String],
    collection_id: Option<&str>,
) -> Result<Vec<ScannedProject>, String> {
    let mut imported = Vec::new();
    for path in paths {
        let path = path.trim();
        if path.is_empty() || project_repository::exists_with_path(conn, path).map_err(db_err)? {
            continue;
        }
        let dir = Path::new(path);
        if !dir.is_dir() {
            continue;
        }
        let input = ProjectInput {
            name: scanner_service::project_name_from_path(path),
            path: path.to_string(),
            description: None,
            status: Some("IN_PROGRESS".into()),
            cover_emoji: None,
            cover_color: None,
            notes: None,
        };
        if let Ok(project) = create(conn, input) {
            if let Some(cid) = collection_id {
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO project_collections (project_id, collection_id) VALUES (?1, ?2)",
                    rusqlite::params![project.id, cid],
                );
                activity_repository::log(conn, &project.id, "collection", "加入集合");
            }
            imported.push(ScannedProject {
                name: project.name,
                path: project.path,
                is_project: project.language.is_some(),
                language: project.language,
                already_imported: true,
            });
        }
    }
    Ok(imported)
}

fn db_err(err: rusqlite::Error) -> String {
    format!("数据库操作失败: {err}")
}
