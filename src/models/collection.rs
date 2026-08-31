use serde::{Deserialize, Serialize};

/// 项目集合：人为组织项目（学习 / 工作 / 个人 / 创作…），与描述属性的 Tag 分离
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionInput {
    pub name: String,
}
