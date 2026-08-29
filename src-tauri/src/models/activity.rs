use serde::{Deserialize, Serialize};

/// 项目动态：由系统动作自动产生，用于详情页 Timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub project_id: String,
    /// created | updated | link | todo
    pub kind: String,
    pub message: String,
    pub created_at: String,
}
