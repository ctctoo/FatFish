use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::settings;
use crate::repository::{project_repository, release_repository};
use crate::services::{changelog_service, git_service, github_auth_service, release_service};
use crate::Db;

type CmdResult<T> = Result<T, String>;

/// 向导第一步所需的全部上下文
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseContext {
    pub checks: Vec<ReleaseCheck>,
    pub latest_tag: Option<String>,
    pub suggested_version: String,
    pub suggested_tag: String,
    pub commits: Vec<CommitSummary>,
    pub draft_changelog: String,
    /// 账号登录名（token 有效时）
    pub login: Option<String>,
    /// token 来源：account / pat / gh_cli / none
    pub token_source: String,
    pub has_version_files: bool,
    pub is_dirty: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseCheck {
    pub key: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitSummary {
    pub hash: String,
    pub subject: String,
}

/// 网络调用（token 验证 / 仓库权限检查）与 git 子进程都是阻塞操作，
/// 统一放进 spawn_blocking：Tauri 同步命令跑在主线程，会卡死整个 UI。
fn get_release_context_inner(conn: &rusqlite::Connection, project_id: &str) -> CmdResult<ReleaseContext> {
    let project = project_repository::get(conn, project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "项目不存在".to_string())?;
    let path = &project.path;

    let git_info = git_service::collect_git_info(path);
    let mut checks: Vec<ReleaseCheck> = Vec::new();

    // git 可用性
    let git_ok = git_info.branch.is_some() || git_info.commit_hash.is_some();
    checks.push(ReleaseCheck {
        key: "git".into(),
        passed: git_ok,
        message: if git_ok { "Git 可用".into() } else { "Git 不可用或目录不是 Git 仓库".into() },
    });

    // remote + GitHub
    let owner_repo = github_auth_service::resolve_github_repo(conn, path);
    match &owner_repo {
        Ok((owner, repo)) => checks.push(ReleaseCheck {
            key: "remote".into(),
            passed: true,
            message: format!("GitHub 仓库：{owner}/{repo}"),
        }),
        Err(e) => checks.push(ReleaseCheck {
            key: "remote".into(),
            passed: false,
            message: e.clone(),
        }),
    }

    // 鉴权与仓库推送权限
    let mut login = None;
    let mut token_source = "none".to_string();
    let auth_check = match github_auth_service::resolve_token(conn) {
        Some(token) => match github_auth_service::verify_token(&token.token) {
            Ok(user_login) => {
                login = Some(user_login.clone());
                token_source = token.source.as_str().to_string();
                match &owner_repo {
                    Ok((owner, repo)) => match github_auth_service::check_repo_access(&token.token, owner, repo) {
                        Ok(access) if access.can_push => ReleaseCheck {
                            key: "auth".into(),
                            passed: true,
                            message: format!("已认证：{user_login}（对 {} 有推送权限）", access.full_name),
                        },
                        Ok(_) => ReleaseCheck {
                            key: "auth".into(),
                            passed: false,
                            message: format!("账号 {user_login} 对该仓库没有推送权限"),
                        },
                        Err(e) => ReleaseCheck { key: "auth".into(), passed: false, message: e },
                    },
                    Err(_) => ReleaseCheck {
                        key: "auth".into(),
                        passed: true,
                        message: format!("已认证：{user_login}"),
                    },
                }
            }
            Err(e) => ReleaseCheck { key: "auth".into(), passed: false, message: e },
        },
        None => ReleaseCheck {
            key: "auth".into(),
            passed: false,
            message: "尚未配置 GitHub 凭证：请登录 GitHub / 填写 Token / 或登录 gh CLI".into(),
        },
    };
    checks.push(auth_check);

    // tag 与提交
    let latest_tag = git_service::latest_version_tag(path);
    let commits = git_service::log_since_tag(path, latest_tag.as_deref()).unwrap_or_default();
    let summaries: Vec<CommitSummary> = commits
        .iter()
        .map(|c| CommitSummary {
            hash: c.hash.clone(),
            subject: c.subject.clone(),
        })
        .collect();
    let draft = changelog_service::generate_changelog(&commits, "NEW");

    let suggested_version = release_service::suggest_version(latest_tag.as_deref());
    let suggested_tag = format!("v{suggested_version}");

    checks.push(ReleaseCheck {
        key: "commits".into(),
        passed: !summaries.is_empty(),
        message: if summaries.is_empty() {
            "没有可发布的提交".into()
        } else {
            format!("自上个 tag 以来有 {} 个提交", summaries.len())
        },
    });

    let is_dirty = git_info.is_dirty.unwrap_or(false);
    let has_version_files = !crate::services::version_file_service::detect_version_files(path).is_empty();

    Ok(ReleaseContext {
        checks,
        latest_tag,
        suggested_version,
        suggested_tag,
        commits: summaries,
        draft_changelog: draft,
        login,
        token_source,
        has_version_files,
        is_dirty,
    })
}

#[tauri::command]
pub async fn get_release_context(app: AppHandle, project_id: String) -> CmdResult<ReleaseContext> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Db>();
        let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
        get_release_context_inner(&conn, &project_id)
    })
    .await
    .map_err(|e| format!("任务执行失败：{e}"))?
}

fn polish_changelog_inner(conn: &rusqlite::Connection, markdown: &str) -> CmdResult<String> {
    let api_key = crate::repository::settings_repository::get(conn, settings::KEY_AI_API_KEY)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let base_url = crate::repository::settings_repository::get(conn, settings::KEY_AI_BASE_URL)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let model = crate::repository::settings_repository::get(conn, settings::KEY_AI_MODEL)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    crate::services::ai_service::polish_changelog(&api_key, &base_url, &model, markdown)
}

#[tauri::command]
pub async fn polish_changelog(app: AppHandle, markdown: String) -> CmdResult<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Db>();
        let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
        polish_changelog_inner(&conn, &markdown)
    })
    .await
    .map_err(|e| format!("任务执行失败：{e}"))?
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartReleaseArgs {
    pub project_id: String,
    pub version: String,
    pub tag_name: String,
    pub changelog: String,
    pub assets: Vec<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub sync_version: bool,
}

/// 启动发布：先落库 preparing 记录，然后后台线程执行全流程并推送事件。
/// 本命令本身只做快速落库（不阻塞），重活在 release_service 的线程里。
#[tauri::command]
pub fn start_release(app: AppHandle, db: State<Db>, args: StartReleaseArgs) -> CmdResult<String> {
    if args.version.trim().is_empty() || args.tag_name.trim().is_empty() {
        return Err("版本号和 tag 不能为空".to_string());
    }
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    let release_id = release_repository::insert(
        &conn,
        &args.project_id,
        args.version.trim(),
        args.tag_name.trim(),
        Some(&args.changelog),
    )
    .map_err(|e| e.to_string())?;
    drop(conn);

    let params = release_service::ReleaseParams {
        project_id: args.project_id,
        version: args.version.trim().to_string(),
        tag_name: args.tag_name.trim().to_string(),
        changelog: args.changelog.clone(),
        assets: args.assets,
        draft: args.draft,
        prerelease: args.prerelease,
        sync_version: args.sync_version,
    };
    release_service::spawn_release(app, params, release_id.clone());
    Ok(release_id)
}

#[tauri::command]
pub fn list_releases(db: State<Db>, project_id: String, limit: Option<i64>) -> CmdResult<Vec<crate::models::release::Release>> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    release_repository::list_for_project(&conn, &project_id, limit.unwrap_or(20))
        .map_err(|e| e.to_string())
}

/// 读取发布设置（token 掩码返回；含 gh CLI 子进程探测，放 spawn_blocking）。
#[tauri::command]
pub async fn get_publish_settings(app: AppHandle) -> CmdResult<release_service::PublishSettings> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Db>();
        let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
        Ok(release_service::load_publish_settings(&conn))
    })
    .await
    .map_err(|e| format!("任务执行失败：{e}"))?
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPublishSettingsArgs {
    pub github_pat: Option<String>,
    pub ai_api_key: Option<String>,
    pub ai_base_url: Option<String>,
    pub ai_model: Option<String>,
}

/// 写入发布设置。空字符串表示清除该配置（纯本地写，同步即可）。
#[tauri::command]
pub fn set_publish_settings(db: State<Db>, args: SetPublishSettingsArgs) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    let mut updates: Vec<(&str, &str)> = Vec::new();
    if let Some(v) = &args.github_pat {
        updates.push((settings::KEY_GITHUB_PAT, v.trim()));
    }
    if let Some(v) = &args.ai_api_key {
        updates.push((settings::KEY_AI_API_KEY, v.trim()));
    }
    if let Some(v) = &args.ai_base_url {
        updates.push((settings::KEY_AI_BASE_URL, v.trim()));
    }
    if let Some(v) = &args.ai_model {
        updates.push((settings::KEY_AI_MODEL, v.trim()));
    }
    for (key, value) in updates {
        crate::repository::settings_repository::set(&conn, key, value).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 手动验证 PAT（设置页「验证」按钮；网络请求，放 spawn_blocking）。
#[tauri::command]
pub async fn verify_github_pat(token: String) -> CmdResult<String> {
    tauri::async_runtime::spawn_blocking(move || github_auth_service::verify_token(token.trim()))
        .await
        .map_err(|e| format!("任务执行失败：{e}"))?
}

/// 发布流程事件到达前端的通知入口（progress 已由 service 直接 emit，这里预留统一通道）
pub fn emit_outcome(app: &AppHandle, payload: &release_service::ReleaseOutcome) {
    let _ = app.emit("release://outcome", payload);
}
