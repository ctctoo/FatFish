use rusqlite::Connection;

use crate::models::project::{Project, ProjectFilter};
use crate::repository::project_repository;

/// 搜索服务：MVP 阶段直接委托 repository 的组合条件查询
/// （关键词覆盖名称 / 简介 / 路径 / GitHub / 标签 / 技术栈）
pub fn search(conn: &Connection, filter: ProjectFilter) -> Result<Vec<Project>, String> {
    project_repository::list(conn, &filter).map_err(|e| format!("搜索失败: {e}"))
}
