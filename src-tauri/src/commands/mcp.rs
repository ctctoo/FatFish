use tauri::State;

use crate::db::settings as settings_store;
use crate::repository::settings_repository;
use crate::services::mcp_config_service::{agent_statuses, configure_agent};
use crate::Db;

type CmdResult<T> = Result<T, String>;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpStatus {
    pub enabled: bool,
    pub agents: Vec<crate::services::mcp_config_service::AgentStatus>,
}

#[tauri::command]
pub fn mcp_status(db: State<Db>) -> CmdResult<McpStatus> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    let enabled = settings_repository::get_bool(&conn, settings_store::KEY_MCP_ENABLED)
        .map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(McpStatus { enabled, agents: agent_statuses() })
}

/// 开启 / 关闭 MCP。enable 为 true 时同步为已检测到的 agent 客户端写入配置。
#[tauri::command]
pub fn set_mcp_enabled(db: State<Db>, enable: bool) -> CmdResult<McpStatus> {
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    settings_repository::set_bool(&conn, settings_store::KEY_MCP_ENABLED, enable)
        .map_err(|e| format!("数据库操作失败: {e}"))?;

    let mut results: Vec<(String, Result<bool, String>)> = Vec::new();
    for agent in agent_statuses() {
        // 只为已存在配置文件的客户端写入；关闭时对所有已配置的客户端移除
        if enable && !agent.config_found {
            continue;
        }
        if !enable && !agent.configured {
            continue;
        }
        let r = configure_agent(&agent.id, enable);
        results.push((agent.id.clone(), r));
    }

    let failed: Vec<String> = results
        .into_iter()
        .filter_map(|(id, r)| r.err().map(|e| format!("{id}: {e}")))
        .collect();
    if !failed.is_empty() {
        return Err(format!("部分客户端配置失败：{}", failed.join("；")));
    }

    let enabled = settings_repository::get_bool(&conn, settings_store::KEY_MCP_ENABLED)
        .map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(McpStatus { enabled, agents: agent_statuses() })
}

/// 为单个 agent 客户端写入 / 移除 MCP 配置（用户按需选择客户端）。
#[tauri::command]
pub fn configure_mcp_agent(db: State<Db>, agent_id: String, enable: bool) -> CmdResult<McpStatus> {
    configure_agent(&agent_id, enable)?;
    let conn = db.0.lock().map_err(|_| "数据库忙，请稍后重试".to_string())?;
    let enabled = settings_repository::get_bool(&conn, settings_store::KEY_MCP_ENABLED)
        .map_err(|e| format!("数据库操作失败: {e}"))?;
    Ok(McpStatus { enabled, agents: agent_statuses() })
}
