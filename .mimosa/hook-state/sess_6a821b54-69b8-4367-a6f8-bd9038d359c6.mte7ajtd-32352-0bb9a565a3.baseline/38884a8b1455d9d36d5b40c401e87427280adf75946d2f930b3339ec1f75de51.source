use serde::{Deserialize, Serialize};

/// 项目链接：GitHub / 官网 / 文档 / Figma / 网盘… 项目类型不被限制
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub url: String,
    /// github | website | docs | design | demo | paper | cloud | other
    pub link_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkInput {
    pub title: String,
    pub url: String,
    pub link_type: Option<String>,
}
