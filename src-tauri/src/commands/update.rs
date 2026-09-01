use crate::services::update_service::{self, UpdateInfo};

type CmdResult<T> = Result<T, String>;

/// 检查 GitHub 上是否有比当前版本更新的 Release。
/// 无新版本返回 Ok(None)，网络失败返回 Err 供前端提示。
///
/// `update_service` 用的是阻塞式 reqwest，这里放进 spawn_blocking，
/// 避免同步网络请求卡住主线程（界面“正在检查…”才不会假死）。
#[tauri::command]
pub async fn check_for_update(current_version: String) -> CmdResult<Option<UpdateInfo>> {
    tauri::async_runtime::spawn_blocking(move || update_service::check_update(&current_version))
        .await
        .map_err(|e| format!("检查更新失败：{e}"))?
}
