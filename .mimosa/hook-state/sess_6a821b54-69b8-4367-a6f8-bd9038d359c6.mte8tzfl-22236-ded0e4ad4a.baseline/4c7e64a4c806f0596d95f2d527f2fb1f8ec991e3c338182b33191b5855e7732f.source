use serde::{Deserialize, Serialize};

/// 只读 Git 信息。任何字段读取失败都应为 None，不能让项目加载失败。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInfo {
    pub remote_url: Option<String>,
    pub branch: Option<String>,
    pub commit_hash: Option<String>,
    pub commit_message: Option<String>,
    pub commit_time: Option<String>,
    pub is_dirty: Option<bool>,
}
