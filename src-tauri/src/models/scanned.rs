use serde::{Deserialize, Serialize};

/// 扫描发现的项目（尚未入库）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedProject {
    pub name: String,
    pub path: String,
    /// 检测到项目特征文件（package.json / Cargo.toml 等）
    pub is_project: bool,
    pub language: Option<String>,
    /// 数据库中已存在同路径项目
    pub already_imported: bool,
}
