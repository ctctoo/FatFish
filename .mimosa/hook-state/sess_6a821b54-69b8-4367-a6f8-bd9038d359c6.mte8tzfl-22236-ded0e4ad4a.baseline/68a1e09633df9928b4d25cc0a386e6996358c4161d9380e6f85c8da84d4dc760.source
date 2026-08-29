use serde::{Deserialize, Serialize};

/// 任务：可关联项目、可设截止日期
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub done: bool,
    pub project_id: Option<String>,
    /// 关联项目名称（列表联查，便于展示）
    pub project_name: Option<String>,
    /// 项目状态色（用于小圆点）
    pub project_status: Option<String>,
    /// 截止日期，YYYY-MM-DD
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoInput {
    pub title: String,
    pub project_id: Option<String>,
    pub due_date: Option<String>,
}
