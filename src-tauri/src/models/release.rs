use serde::{Deserialize, Serialize};

/// 一次版本发布的记录（本地留存，与 GitHub Release 对应）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: String,
    pub project_id: String,
    /// 语义化版本号，如 "1.2.0"
    pub version: String,
    /// 打的 tag 名，如 "v1.2.0"
    pub tag_name: String,
    /// preparing | tag_pushed | published | failed
    pub status: String,
    /// 最终发布的 Markdown 说明
    pub changelog: Option<String>,
    /// GitHub Release 页面链接
    pub release_url: Option<String>,
    /// 失败原因
    pub error_message: Option<String>,
    pub released_at: Option<String>,
    pub created_at: String,
}

/// 发布流程的当前进度（前端进度事件 payload / 执行态快照）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseProgress {
    pub release_id: String,
    /// prepare | version | tag | release | assets | done
    pub step: String,
    /// 当前正在上传的文件名（仅 assets 阶段）
    pub current_file: Option<String>,
    /// 已上传字节数（仅 assets 阶段）
    pub uploaded_bytes: Option<u64>,
    /// 当前文件总字节数（仅 assets 阶段）
    pub total_bytes: Option<u64>,
    pub message: Option<String>,
}
