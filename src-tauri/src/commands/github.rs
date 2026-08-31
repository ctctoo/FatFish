use tauri::State;

use crate::models::github::{GithubAccount, GithubRepo};
use crate::services::github_service;
use crate::Db;

type CmdResult<T> = Result<T, String>;

/// 启动 GitHub Device Flow 登录，返回用户需要在浏览器输入的设备码。
#[tauri::command]
pub fn github_login_start(client_id: String) -> CmdResult<github_service::GithubDeviceCode> {
    github_service::request_device_code(&client_id)
}

/// 轮询一次 GitHub 授权结果；前端按 GitHub 返回的 interval 周期性调用。
#[tauri::command]
pub fn github_login_poll(
    db: State<Db>,
    client_id: String,
    device_code: String,
) -> CmdResult<github_service::GithubLoginResult> {
    let result = github_service::poll_login(&client_id, &device_code)?;
    if result.status == "success" {
        if let Some(account) = &result.account {
            let conn = db
                .0
                .lock()
                .map_err(|_| "数据库忙，请稍后重试".to_string())?;
            github_service::save_account(&conn, account)?;
        }
    }
    Ok(result)
}

/// 查询本地是否已保存 GitHub 账号。
#[tauri::command]
pub fn github_status(db: State<Db>) -> CmdResult<Option<GithubAccount>> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    github_service::load_account(&conn)
}

/// 退出登录（仅删除本地保存的令牌）。
#[tauri::command]
pub fn github_logout(db: State<Db>) -> CmdResult<()> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    github_service::logout(&conn)
}

/// 拉取已登录账号的仓库列表。
#[tauri::command]
pub fn github_list_repos(db: State<Db>) -> CmdResult<Vec<GithubRepo>> {
    let token = {
        let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
        github_service::load_account(&conn)?.map(|a| a.token)
    };
    match token {
        Some(token) => github_service::list_repos(&token),
        None => Err("尚未登录 GitHub".to_string()),
    }
}
