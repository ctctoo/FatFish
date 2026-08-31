use serde::{Deserialize, Serialize};

/// 已登录的 GitHub 账号（含访问令牌）。
/// 令牌只存于本地 SQLite，绝不写入 localStorage 或日志。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubAccount {
    pub token: String,
    pub user: GithubUser,
    pub logged_in_at: String,
}

/// GitHub 用户公开信息（来自 GET /user）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub html_url: Option<String>,
    pub bio: Option<String>,
    pub public_repos: Option<u64>,
}

/// GitHub 仓库摘要（来自 GET /user/repos）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stargazers_count: Option<u64>,
    pub forks_count: Option<u64>,
    pub updated_at: Option<String>,
    pub private: bool,
    pub fork: bool,
    pub archived: bool,
    pub owner_avatar_url: Option<String>,
}
