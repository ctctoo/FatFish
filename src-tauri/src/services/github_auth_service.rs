use rusqlite::Connection;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use std::path::Path;
use std::process::Command;

use crate::db::settings;
use crate::services::git_service::normalize_remote_url;

const GITHUB_API_BASE: &str = "https://api.github.com";
const USER_AGENT_VALUE: &str = "FatFish";

/// Token 的来源，用于 UI 展示
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenSource {
    /// 应用内已登录的 GitHub 账号（github_account 表）
    Account,
    /// 设置页填写的 PAT（app_settings.github_pat）
    PersonalAccessToken,
    /// 本机 gh CLI 已登录
    GhCli,
}

impl TokenSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenSource::Account => "account",
            TokenSource::PersonalAccessToken => "pat",
            TokenSource::GhCli => "gh_cli",
        }
    }
}

/// 解析到的可用凭证
#[derive(Debug, Clone)]
pub struct ResolvedToken {
    pub token: String,
    pub source: TokenSource,
}

/// 按优先级解析 GitHub 凭证：设置页 PAT -> 已登录账号 -> gh CLI。
/// PAT 优先：应用内登录走的是 GitHub App Token，其 API 权限由 App 配置决定
/// （可能缺少 Contents 写权限），而 PAT 是用户为发布显式提供的凭证。
pub fn resolve_token(conn: &Connection) -> Option<ResolvedToken> {
    // 1. 设置页 PAT（发布专用，权限最明确）
    if let Ok(Some(pat)) = crate::repository::settings_repository::get(conn, settings::KEY_GITHUB_PAT) {
        if !pat.trim().is_empty() {
            return Some(ResolvedToken {
                token: pat.trim().to_string(),
                source: TokenSource::PersonalAccessToken,
            });
        }
    }
    // 2. 应用内已登录账号
    if let Ok(Some(account)) = crate::services::github_service::load_account(conn) {
        if !account.token.is_empty() {
            return Some(ResolvedToken {
                token: account.token,
                source: TokenSource::Account,
            });
        }
    }
    // 3. gh CLI
    if let Some(token) = read_gh_cli_token() {
        return Some(ResolvedToken {
            token,
            source: TokenSource::GhCli,
        });
    }
    None
}

/// 检测本机 gh CLI 是否已登录（`gh auth token` 成功输出即视为可用）。
pub fn read_gh_cli_token() -> Option<String> {
    let output = Command::new("gh").args(["auth", "token"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn client() -> Client {
    Client::new()
}

/// 验证 token 有效性，返回 login。
pub fn verify_token(token: &str) -> Result<String, String> {
    let resp = client()
        .get(format!("{GITHUB_API_BASE}/user"))
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, USER_AGENT_VALUE)
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    match resp.status().as_u16() {
        200 => {
            #[derive(serde::Deserialize)]
            struct Raw {
                login: String,
            }
            let raw: Raw = resp
                .json()
                .map_err(|e| format!("GitHub 返回了无法解析的用户信息：{e}"))?;
            Ok(raw.login)
        }
        401 => Err("Token 无效或已过期，请重新配置".to_string()),
        status => Err(format!("验证 Token 失败（HTTP {status}）")),
    }
}

/// 仓库访问检查结果
#[derive(Debug, Clone)]
pub struct RepoAccess {
    pub full_name: String,
    pub can_push: bool,
    pub private: bool,
}

/// 检查对 owner/repo 的访问权限与推送权限。
pub fn check_repo_access(token: &str, owner: &str, repo: &str) -> Result<RepoAccess, String> {
    let resp = client()
        .get(format!("{GITHUB_API_BASE}/repos/{owner}/{repo}"))
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, USER_AGENT_VALUE)
        .send()
        .map_err(|e| format!("无法连接 GitHub：{e}"))?;
    match resp.status().as_u16() {
        200 => {
            #[derive(serde::Deserialize)]
            struct Permissions {
                #[serde(default)]
                push: bool,
            }
            #[derive(serde::Deserialize)]
            struct Raw {
                full_name: String,
                private: bool,
                permissions: Permissions,
            }
            let raw: Raw = resp
                .json()
                .map_err(|e| format!("GitHub 返回了无法解析的仓库信息：{e}"))?;
            Ok(RepoAccess {
                full_name: raw.full_name,
                can_push: raw.permissions.push,
                private: raw.private,
            })
        }
        401 => Err("Token 无效或已过期，请重新配置".to_string()),
        403 => Err("当前账号没有该仓库的访问权限".to_string()),
        404 => Err("仓库不存在，或当前账号无权访问（owner/repo 是否正确？）".to_string()),
        status => Err(format!("检查仓库权限失败（HTTP {status}）")),
    }
}

/// 从归一化后的 remote URL 解析 owner/repo。
/// 仅支持 github.com；其他平台返回 None。
pub fn parse_owner_repo(remote_url: &str) -> Option<(String, String)> {
    let url = normalize_remote_url(remote_url);
    let rest = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))?;
    let mut parts = rest.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// 读取项目 remote 并解析 owner/repo。没有 remote 或不是 GitHub 时返回结构化错误。
pub fn resolve_github_repo(conn: &Connection, project_path: &str) -> Result<(String, String), String> {
    let info = crate::services::git_service::collect_git_info(project_path);
    let Some(remote) = info.remote_url else {
        return Err("该项目没有配置 origin remote，请先在项目目录执行 git remote add origin <url>".to_string());
    };
    parse_owner_repo(&remote).ok_or_else(|| {
        format!("remote 不是 GitHub 仓库（{remote}），当前版本发布仅支持 github.com")
    })
}

/// 校验 token 对 owner/repo 的推送权限，返回仓库信息。
pub fn ensure_can_publish(conn: &Connection, project_path: &str) -> Result<(ResolvedToken, RepoAccess), String> {
    let Some(token) = resolve_token(conn) else {
        return Err("尚未配置 GitHub 凭证：请在应用内登录 GitHub，或在设置页填写 Token，或安装并登录 gh CLI".to_string());
    };
    let (owner, repo) = resolve_github_repo(conn, project_path)?;
    let access = check_repo_access(&token.token, &owner, &repo)?;
    if !access.can_push {
        return Err(format!("当前账号对 {} 没有推送权限，无法发布", access.full_name));
    }
    Ok((token, access))
}

#[cfg(test)]
mod tests {
    use super::parse_owner_repo;

    #[test]
    fn parses_https_and_ssh_urls() {
        assert_eq!(
            parse_owner_repo("https://github.com/a/b.git"),
            Some(("a".to_string(), "b".to_string()))
        );
        assert_eq!(
            parse_owner_repo("git@github.com:a/b.git"),
            Some(("a".to_string(), "b".to_string()))
        );
        assert_eq!(
            parse_owner_repo("https://github.com/a/b/"),
            Some(("a".to_string(), "b".to_string()))
        );
    }

    #[test]
    fn rejects_non_github() {
        assert_eq!(parse_owner_repo("https://gitee.com/a/b.git"), None);
        assert_eq!(parse_owner_repo("https://gitlab.com/a/b.git"), None);
        assert_eq!(parse_owner_repo(""), None);
    }
}
