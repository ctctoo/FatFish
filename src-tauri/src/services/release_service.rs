use std::io::Read;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::db::settings;
use crate::models::release::ReleaseProgress;
use crate::repository::{activity_repository, project_repository, release_repository};
use crate::services::git_service;
use crate::services::github_auth_service;
use crate::services::version_file_service;

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_UPLOAD_BASE: &str = "https://uploads.github.com";
const USER_AGENT_VALUE: &str = "FatFish";

/// 发布参数（来自向导）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseParams {
    pub project_id: String,
    pub version: String,
    pub tag_name: String,
    pub changelog: String,
    /// 要上传的本地文件
    pub assets: Vec<String>,
    pub draft: bool,
    pub prerelease: bool,
    /// 是否同步版本号到项目文件
    pub sync_version: bool,
}

/// 发布执行结果（后台线程结束时发事件 / 写库）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseOutcome {
    pub release_id: String,
    pub success: bool,
    pub release_url: Option<String>,
    pub error: Option<String>,
    pub step: String,
}

/// 从项目本地发布记录里取上一个版本（用于建议版本号），没有则 None。
pub fn last_released_version(conn: &rusqlite::Connection, project_id: &str) -> Option<String> {
    release_repository::latest_for_project(conn, project_id)
        .ok()
        .flatten()
        .filter(|r| r.status == "published")
        .map(|r| r.version)
}

/// 依据最近 tag 建议下一个版本号：patch 位 +1；无 tag 时 0.1.0。
pub fn suggest_version(latest_tag: Option<&str>) -> String {
    let Some(tag) = latest_tag else {
        return "0.1.0".to_string();
    };
    let v = tag.trim().trim_start_matches(['v', 'V']);
    let mut parts: Vec<u64> = v.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    parts.resize(3, 0);
    format!("{}.{}.{}", parts[0], parts[1], parts[2] + 1)
}

fn emit_progress(app: &AppHandle, release_id: &str, step: &str, current_file: Option<String>, uploaded: Option<u64>, total: Option<u64>, message: Option<String>) {
    let _ = app.emit(
        "release://progress",
        ReleaseProgress {
            release_id: release_id.to_string(),
            step: step.to_string(),
            current_file,
            uploaded_bytes: uploaded,
            total_bytes: total,
            message,
        },
    );
}

/// 在后台线程中执行完整发布流程（blocking HTTP + git 子进程均不适合放在 async runtime 上）。
/// 每一步推进都更新 releases 表状态，并通过 `release://progress` / `release://outcome` 事件通知前端。
pub fn spawn_release(app: AppHandle, params: ReleaseParams, release_id: String) {
    std::thread::spawn(move || {
        let outcome = run_release_flow(&app, &params, &release_id);
        let _ = app.emit("release://outcome", &outcome);
        let db = app.state::<crate::Db>();
        let Ok(conn) = db.0.lock() else { return };
        if outcome.success {
            if let Some(url) = &outcome.release_url {
                activity_repository::log(&conn, &params.project_id, "release", &format!("发布 {}", params.tag_name));
                let _ = release_repository::mark_published(&conn, &release_id, url, Some(&params.changelog));
            }
        } else if let Some(err) = &outcome.error {
            let _ = release_repository::update_status(&conn, &release_id, "failed", Some(err));
        }
    });
}

fn run_release_flow(app: &AppHandle, params: &ReleaseParams, release_id: &str) -> ReleaseOutcome {
    let fail = |step: &str, err: String| ReleaseOutcome {
        release_id: release_id.to_string(),
        success: false,
        release_url: None,
        error: Some(err),
        step: step.to_string(),
    };

    let db = app.state::<crate::Db>();
    let conn = match db.0.lock() {
        Ok(c) => c,
        Err(_) => return fail("prepare", "数据库忙，请稍后重试".to_string()),
    };

    // ---- 前置检查 ----
    let project = match project_repository::get(&conn, &params.project_id) {
        Ok(Some(p)) => p,
        _ => return fail("prepare", "项目不存在".to_string()),
    };
    let project_path = project.path.clone();

    let (token, access) = match github_auth_service::ensure_can_publish(&conn, &project_path) {
        Ok(v) => v,
        Err(e) => return fail("prepare", e),
    };
    let (owner, repo) = match github_auth_service::parse_owner_repo(
        &git_service::collect_git_info(&project_path).remote_url.unwrap_or_default(),
    ) {
        Some(v) => v,
        None => return fail("prepare", "无法从 remote 解析 owner/repo".to_string()),
    };

    // ---- 可选：同步版本号并提交 ----
    if params.sync_version {
        emit_progress(app, release_id, "version", None, None, None, Some("同步版本号到项目文件".to_string()));
        match version_file_service::bump_version(&project_path, &params.version) {
            Ok(files) if !files.is_empty() => {
                let msg = format!("chore(release): bump version to {}", params.version);
                if let Err(e) = git_service::commit_files(&project_path, &files, &msg) {
                    return fail("version", format!("提交版本号变更失败：{e}"));
                }
            }
            Ok(_) => {} // 没有文件变化
            Err(e) => return fail("version", e),
        }
    }

    // ---- tag & push ----
    emit_progress(app, release_id, "tag", None, None, None, Some(format!("创建并推送 tag {}", params.tag_name)));
    if let Err(e) = git_service::tag_and_push(&project_path, &params.tag_name) {
        return fail("tag", e);
    }
    let _ = release_repository::update_status(&conn, release_id, "tag_pushed", None);

    // ---- 创建 GitHub Release ----
    emit_progress(app, release_id, "release", None, None, None, Some("创建 GitHub Release".to_string()));
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
    {
        Ok(c) => c,
        Err(e) => return fail("release", format!("初始化 HTTP 客户端失败：{e}")),
    };
    let create_body = serde_json::json!({
        "tag_name": params.tag_name,
        "name": params.tag_name,
        "body": params.changelog,
        "draft": params.draft,
        "prerelease": params.prerelease,
    });
    let resp = match client
        .post(format!("{GITHUB_API_BASE}/repos/{owner}/{repo}/releases"))
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token.token))
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header(reqwest::header::USER_AGENT, USER_AGENT_VALUE)
        .json(&create_body)
        .send()
    {
        Ok(r) => r,
        Err(e) => return fail("release", format!("无法连接 GitHub：{e}")),
    };
    let status = resp.status().as_u16();
    let text = resp.text().unwrap_or_default();
    if !(200..300).contains(&status) {
        let hint = match status {
            401 => "Token 无效或已过期".to_string(),
            403 => "没有创建 Release 的权限".to_string(),
            422 => "GitHub 拒绝了请求：tag 已存在或字段非法".to_string(),
            s => format!("HTTP {s}"),
        };
        return fail("release", format!("创建 Release 失败（{hint}）：{text}"));
    }
    let created: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return fail("release", format!("解析 Release 响应失败：{e}")),
    };
    let gh_release_id = created["id"].as_u64().unwrap_or(0);
    let html_url = created["html_url"].as_str().unwrap_or_default().to_string();
    if gh_release_id == 0 {
        return fail("release", "GitHub 响应缺少 Release id".to_string());
    }

    // ---- 上传 assets ----
    let total_assets = params.assets.len();
    for (idx, path_str) in params.assets.iter().enumerate() {
        let path = PathBuf::from(path_str);
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("asset-{idx}"));
        let meta = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => return fail("assets", format!("无法读取产物 {file_name}：{e}")),
        };
        let total = meta.len();
        emit_progress(
            app,
            release_id,
            "assets",
            Some(file_name.clone()),
            Some(0),
            Some(total),
            Some(format!("上传产物 {}/{}：{}", idx + 1, total_assets, file_name)),
        );

        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => return fail("assets", format!("无法打开产物 {file_name}：{e}")),
        };
        let app_for_reader = app.clone();
        let release_id_for_reader = release_id.to_string();
        let file_name_for_reader = file_name.clone();
        let mut reader = ProgressReader {
            inner: file,
            pos: 0,
            total,
            on_progress: Box::new(move |uploaded| {
                emit_progress(
                    &app_for_reader,
                    &release_id_for_reader,
                    "assets",
                    Some(file_name_for_reader.clone()),
                    Some(uploaded),
                    Some(total),
                    None,
                );
            }),
            last_reported: 0,
        };

        let url = format!("{GITHUB_UPLOAD_BASE}/repos/{owner}/{repo}/releases/{gh_release_id}/assets?name={file_name}");
        let resp = client
            .post(&url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token.token))
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .header(reqwest::header::USER_AGENT, USER_AGENT_VALUE)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .header(reqwest::header::CONTENT_LENGTH, total)
            .body(reqwest::blocking::Body::new(reader))
            .send();
        match resp {
            Ok(r) if r.status().is_success() => {}
            Ok(r) => {
                let s = r.status();
                let t = r.text().unwrap_or_default();
                return fail("assets", format!("上传 {file_name} 失败（HTTP {s}）：{t}"));
            }
            Err(e) => return fail("assets", format!("上传 {file_name} 失败：{e}")),
        }
    }

    // ---- 完成 ----
    emit_progress(app, release_id, "done", None, None, None, Some("发布完成".to_string()));
    let url = if html_url.is_empty() { None } else { Some(html_url) };
    ReleaseOutcome {
        release_id: release_id.to_string(),
        success: true,
        release_url: url,
        error: None,
        step: "done".to_string(),
    }
}

/// 带进度回调的文件读取器：每上报 256KB 触发一次进度事件。
struct ProgressReader<F: FnMut(u64)> {
    inner: std::fs::File,
    pos: u64,
    total: u64,
    on_progress: F,
    last_reported: u64,
}

impl<F: FnMut(u64)> Read for ProgressReader<F> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.pos += n as u64;
        if self.pos - self.last_reported >= 256 * 1024 || self.pos == self.total {
            self.last_reported = self.pos;
            (self.on_progress)(self.pos);
        }
        Ok(n)
    }
}

/// 读取发布相关设置（token 掩码返回，绝不回传明文）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishSettings {
    pub has_logged_in_account: bool,
    pub gh_cli_available: bool,
    pub has_pat: bool,
    pub pat_masked: Option<String>,
    pub ai_base_url: String,
    pub ai_model: String,
    pub has_ai_key: bool,
}

pub fn load_publish_settings(conn: &rusqlite::Connection) -> PublishSettings {
    let has_account = crate::services::github_service::load_account(conn)
        .ok()
        .flatten()
        .is_some();
    let gh_available = github_auth_service::read_gh_cli_token().is_some();
    let pat = crate::repository::settings_repository::get(conn, settings::KEY_GITHUB_PAT)
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty());
    let pat_masked = pat.as_ref().map(|v| {
        let v = v.trim();
        if v.len() <= 8 {
            format!("{}****", &v[..2.min(v.len())])
        } else {
            format!("{}****{}", &v[..4], &v[v.len() - 4..])
        }
    });
    let ai_base_url = crate::repository::settings_repository::get(conn, settings::KEY_AI_BASE_URL)
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| settings::DEFAULT_AI_BASE_URL.to_string());
    let ai_model = crate::repository::settings_repository::get(conn, settings::KEY_AI_MODEL)
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| settings::DEFAULT_AI_MODEL.to_string());
    let has_ai_key = crate::repository::settings_repository::get(conn, settings::KEY_AI_API_KEY)
        .ok()
        .flatten()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    PublishSettings {
        has_logged_in_account: has_account,
        gh_cli_available: gh_available,
        has_pat: pat.is_some(),
        pat_masked,
        ai_base_url,
        ai_model,
        has_ai_key,
    }
}

#[cfg(test)]
mod tests {
    use super::suggest_version;

    #[test]
    fn suggests_patch_bump() {
        assert_eq!(suggest_version(Some("v1.2.3")), "1.2.4");
        assert_eq!(suggest_version(Some("1.2")), "1.2.1");
        assert_eq!(suggest_version(Some("2.0.0")), "2.0.1");
    }

    #[test]
    fn suggests_default_without_tag() {
        assert_eq!(suggest_version(None), "0.1.0");
    }
}
