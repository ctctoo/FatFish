use serde::{Deserialize, Serialize};

/// 扫描发现的项目（尚未入库）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedProject {
    pub name: String,
    pub path: String,
    pub language: Option<String>,
    /// 数据库中已存在同路径项目
    pub already_imported: bool,
}
