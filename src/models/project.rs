use serde::{Deserialize, Serialize};

use super::collection::Collection;
use super::git::GitInfo;
use super::link::Link;
use super::tag::Tag;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub status: String,
    pub favorite: bool,
    pub tags: Vec<Tag>,
    pub collections: Vec<Collection>,
    pub links: Vec<Link>,
    /// 封面：emoji 图标 / 自动生成颜色，均可为空（用名称自动生成）
    pub cover_emoji: Option<String>,
    pub cover_color: Option<String>,
    /// Markdown 备注
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: String,
    // ---- 可选开发者模块：本地路径与 Git 信息 ----
    pub language: Option<String>,
    pub git_info: Option<GitInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub cover_emoji: Option<String>,
    pub cover_color: Option<String>,
    pub notes: Option<String>,
}

/// 组合筛选条件：关键词 + 状态 + 收藏 + 标签 + 集合 + 最近打开 + 排序
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectFilter {
    pub query: Option<String>,
    pub status: Option<String>,
    pub favorite: Option<bool>,
    pub tag_id: Option<String>,
    pub collection_id: Option<String>,
    pub recent: Option<bool>,
    /// updated | name | opened | created
    pub sort: Option<String>,
}

pub const STATUS_IN_PROGRESS: &str = "IN_PROGRESS";
pub const STATUS_PLANNED: &str = "PLANNED";
pub const STATUS_PAUSED: &str = "PAUSED";
pub const STATUS_COMPLETED: &str = "COMPLETED";
pub const STATUS_ARCHIVED: &str = "ARCHIVED";

pub const ALL_STATUSES: [&str; 5] = [
    STATUS_IN_PROGRESS,
    STATUS_PLANNED,
    STATUS_PAUSED,
    STATUS_COMPLETED,
    STATUS_ARCHIVED,
];

pub fn is_valid_status(s: &str) -> bool {
    ALL_STATUSES.contains(&s)
}
