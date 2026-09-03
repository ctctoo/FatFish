use tauri::{AppHandle, Manager};

use crate::models::github::{GithubAccount, GithubRepo};
use crate::services::github_service;
use crate::Db;

type CmdResult<T> = Result<T, String>;

/// 启动 GitHub Device Flow 登录，返回用户需要在浏览器输入的设备码。
/// 网络请求放进 spawn_blocking：Tauri 的同步命令跑在主线程上，
/// 阻塞 HTTP 会导致整个 UI 卡死（登录轮询期间界面假死）。
#[tauri::command]
pub async fn github_login_start(client_id: String) -> CmdResult<github_service::GithubDeviceCode> {
    tauri::async_runtime::spawn_blocking(move || github_service::request_device_code(&client_id))
        .await
        .map_err(|e| format!("任务执行失败：{e}"))?
}

/// 轮询一次 GitHub 授权结果；前端按 GitHub 返回的 interval 周期性调用。
#[tauri::command]
pub async fn github_login_poll(
    app: AppHandle,
    client_id: String,
    device_code: String,
) -> CmdResult<github_service::GithubLoginResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let result = github_service::poll_login(&client_id, &device_code)?;
        if result.status == "success" {
            if let Some(account) = &result.account {
                let db = app.state::<Db>();
                let conn = db
                    .0
                    .lock()
                    .map_err(|_| "数据库忙，请稍后重试".to_string())?;
                github_service::save_account(&conn, account)?;
            }
        }
        Ok(result)
    })
    .await
    .map_err(|e| format!("任务执行失败：{e}"))?
}

/// 查询本地是否已保存 GitHub 账号（纯本地读取，同步即可）。
#[tauri::command]
pub fn github_status(db: tauri::State<Db>) -> CmdResult<Option<GithubAccount>> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    github_service::load_account(&conn)
}

/// 退出登录（仅删除本地保存的令牌）。
#[tauri::command]
pub fn github_logout(db: tauri::State<Db>) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    github_service::logout(&conn)
}

/// 拉取已登录账号的仓库列表（网络请求，放 spawn_blocking）。
#[tauri::command]
pub async fn github_list_repos(app: AppHandle) -> CmdResult<Vec<GithubRepo>> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<Db>();
        let token = {
            let conn = db
                .0
                .lock()
                .map_err(|_| "数据库忙，请稍后重试".to_string())?;
            github_service::load_account(&conn)?.map(|a| a.token)
        };
        match token {
            Some(token) => github_service::list_repos(&token),
            None => Err("尚未登录 GitHub".to_string()),
        }
    })
    .await
    .map_err(|e| format!("任务执行失败：{e}"))?
}
