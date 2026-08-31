use rusqlite::Connection;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::models::github::{GithubAccount, GithubRepo, GithubUser};
use crate::repository::github_repository;

const GITHUB_API_BASE: &str = "https://api.github.com";
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_AGENT_VALUE: &str = "FatFish";

/// 内置的 GitHub OAuth App Client ID（开发者配置一次即可，用户无需自行创建 OAuth App）。
/// 留空时可回退到前端设置里的自定义 Client ID。
const DEFAULT_GITHUB_CLIENT_ID: &str = "";

/// 解析客户端实际使用的 Client ID：优先使用前端传入的自定义值，否则使用内置默认值。
fn resolve_client_id(candidate: &str) -> Result<String, String> {
    let candidate = candidate.trim();
    if !candidate.is_empty() {
        return Ok(candidate.to_string());
    }
    let default = DEFAULT_GITHUB_CLIENT_ID.trim();
    if !default.is_empty() {
        return Ok(default.to_string());
    }
    Err("未配置 GitHub OAuth App 的 Client ID，请联系应用开发者".to_string())
}

/// Device Flow 第一步：申请设备码。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubDeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval: u64,
    pub expires_in: u64,
}

/// 单次轮询登录结果。status 取值：success / pending / slow_down / expired / denied / failed。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubLoginResult {
    pub status: String,
    pub account: Option<GithubAccount>,
    pub interval: Option<u64>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct DeviceCodeResponse {
    #[serde(default)]
    device_code: Option<String>,
    #[serde(default)]
    user_code: Option<String>,
    #[serde(default)]
    verification_uri: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    interval: Option<u64>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct TokenResponse {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
    #[serde(default)]
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawUser {
    id: u64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
    html_url: Option<String>,
    bio: Option<String>,
    public_repos: Option<u64>,
}

impl From<RawUser> for GithubUser {
    fn from(raw: RawUser) -> Self {
        GithubUser {
            id: raw.id,
            login: raw.login,
            name: raw.name,
            avatar_url: raw.avatar_url,
            html_url: raw.html_url,
            bio: raw.bio,
            public_repos: raw.public_repos,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawOwner {
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RawRepo {
    id: u64,
    name: String,
    full_name: String,
    html_url: String,
    description: Option<String>,
    language: Option<String>,
    stargazers_count: Option<u64>,
    forks_count: Option<u64>,
    updated_at: Option<String>,
    private: bool,
    fork: bool,
    archived: bool,
    owner: RawOwner,
}

impl From<RawRepo> for GithubRepo {
    fn from(raw: RawRepo) -> Self {
        GithubRepo {
            id: raw.id,
            name: raw.name,
            full_name: raw.full_name,
            html_url: raw.html_url,
            description: raw.description,
            language: raw.language,
            stargazers_count: raw.stargazers_count,
            forks_count: raw.forks_count,
            updated_at: raw.updated_at,
            private: raw.private,
            fork: raw.fork,
            archived: raw.archived,
            owner_avatar_url: raw.owner.avatar_url,
        }
    }
}

fn client() -> Client {
    Client::new()
}

/// 发起 Device Flow：POST /login/device/code，返回设备码与验证码。
pub fn request_device_code(client_id: &str) -> Result<GithubDeviceCode, String> {
    let client_id = resolve_client_id(client_id)?;
    let resp = client()
        .post(DEVICE_CODE_URL)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, USER_AGENT_VALUE)
        .form(&[("client_id", client_id.as_str()), ("scope", "repo")])
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    let status = resp.status();
    let body: DeviceCodeResponse = resp
        .json()
        .map_err(|e| format!("GitHub 返回了无法解析的响应：{e}"))?;

    if let Some(error) = body.error {
        let detail = body.error_description.unwrap_or_default();
        return Err(format!("GitHub 拒绝设备授权：{error}（{detail}）"));
    }
    if !status.is_success() {
        return Err(format!("GitHub 返回异常状态码：{status}"));
    }

    let device_code = body.device_code.unwrap_or_default();
    let user_code = body.user_code.unwrap_or_default();
    let verification_uri = body.verification_uri.unwrap_or_default();
    if device_code.is_empty() || user_code.is_empty() || verification_uri.is_empty() {
        return Err("GitHub 设备码响应缺少必要字段".to_string());
    }

    Ok(GithubDeviceCode {
        device_code,
        user_code,
        verification_uri,
        interval: body.interval.unwrap_or(5).max(1),
        expires_in: body.expires_in.unwrap_or(900),
    })
}

/// 轮询一次授权状态。前端按 GitHub 返回的 interval 周期性调用本函数。
pub fn poll_login(client_id: &str, device_code: &str) -> Result<GithubLoginResult, String> {
    let client_id = resolve_client_id(client_id)?;
    let resp = client()
        .post(ACCESS_TOKEN_URL)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, USER_AGENT_VALUE)
        .form(&[
            ("client_id", client_id.as_str()),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    let body: TokenResponse = resp
        .json()
        .map_err(|e| format!("GitHub 返回了无法解析的响应：{e}"))?;

    if let Some(token) = body.access_token {
        if token.is_empty() {
            return Err("GitHub 返回了空的访问令牌".to_string());
        }
        let account = fetch_account(&token)?;
        return Ok(GithubLoginResult {
            status: "success".to_string(),
            account: Some(account),
            interval: None,
            message: None,
        });
    }

    let status = match body.error.as_deref() {
        Some("authorization_pending") => "pending",
        Some("slow_down") => "slow_down",
        Some("expired_token") => "expired",
        Some("access_denied") => "denied",
        _ => "failed",
    };
    Ok(GithubLoginResult {
        status: status.to_string(),
        account: None,
        interval: body.interval,
        message: body.error_description,
    })
}

/// 用令牌拉取当前用户信息，组装成可落库的 GithubAccount。
fn fetch_account(token: &str) -> Result<GithubAccount, String> {
    let resp = client()
        .get(format!("{GITHUB_API_BASE}/user"))
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, USER_AGENT_VALUE)
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("获取 GitHub 用户信息失败（HTTP {}）", resp.status()));
    }
    let raw: RawUser = resp
        .json()
        .map_err(|e| format!("GitHub 返回了无法解析的用户信息：{e}"))?;
    Ok(GithubAccount {
        token: token.to_string(),
        user: raw.into(),
        logged_in_at: crate::db::sqlite::now(),
    })
}

/// 拉取已登录账号可见的仓库列表（owner / collaborator / organization_member）。
pub fn list_repos(token: &str) -> Result<Vec<GithubRepo>, String> {
    let resp = client()
        .get(format!("{GITHUB_API_BASE}/user/repos"))
        .query(&[
            ("per_page", "100"),
            ("affiliation", "owner,collaborator,organization_member"),
            ("sort", "updated"),
            ("visibility", "all"),
        ])
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, USER_AGENT_VALUE)
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("获取仓库列表失败（HTTP {}）", resp.status()));
    }
    let raw: Vec<RawRepo> = resp
        .json()
        .map_err(|e| format!("GitHub 返回了无法解析的仓库数据：{e}"))?;
    Ok(raw.into_iter().map(GithubRepo::from).collect())
}

// ---- 本地账号持久化（服务层组装，保持 repository 只做纯 SQL） ----

pub fn save_account(conn: &Connection, account: &GithubAccount) -> Result<(), String> {
    let user_json = serde_json::to_string(&account.user)
        .map_err(|e| format!("序列化 GitHub 用户信息失败：{e}"))?;
    github_repository::save(
        conn,
        &account.user.login,
        &account.token,
        &user_json,
        &account.logged_in_at,
    )
    .map(|_| ())
    .map_err(|e| format!("保存 GitHub 账号失败：{e}"))
}

pub fn load_account(conn: &Connection) -> Result<Option<GithubAccount>, String> {
    let Some((_login, token, user_json, logged_in_at)) = github_repository::load(conn)
        .map_err(|e| format!("读取 GitHub 账号失败：{e}"))?
    else {
        return Ok(None);
    };
    let user: GithubUser = serde_json::from_str(&user_json)
        .map_err(|e| format!("解析 GitHub 用户信息失败：{e}"))?;
    Ok(Some(GithubAccount {
        token,
        user,
        logged_in_at,
    }))
}

pub fn logout(conn: &Connection) -> Result<(), String> {
    github_repository::delete(conn)
        .map(|_| ())
        .map_err(|e| format!("退出 GitHub 失败：{e}"))
}
