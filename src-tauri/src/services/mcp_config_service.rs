use std::path::PathBuf;

use serde::Serialize;

/// 当前可执行文件路径（MCP 服务器命令）
fn exe_path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().trim_end_matches('"').to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub id: String,
    pub name: String,
    /// 配置文件是否存在
    pub config_found: bool,
    /// MCP 条目是否已配置
    pub configured: bool,
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .ok()
}

/// 各 agent 客户端的配置文件路径
fn config_path(id: &str) -> Option<PathBuf> {
    let home = home_dir()?;
    Some(match id {
        "claude_desktop" => home.join("AppData/Roaming/Claude/claude_desktop_config.json"),
        "cursor" => home.join(".cursor/mcp.json"),
        "windsurf" => home.join(".codeium/windsurf/mcp_config.json"),
        "claude_code" => home.join(".claude.json"),
        "codex" => home.join(".codex/config.toml"),
        "opencode" => home.join(".config/opencode/opencode.json"),
        "vscode" => home.join(".vscode/mcp.json"),
        _ => return None,
    })
}

const SERVER_NAME: &str = "fatfish";

/// 生成 JSON 配置中的 MCP 服务器条目
fn server_entry() -> serde_json::Value {
    let exe = exe_path().unwrap_or_default();
    serde_json::json!({ "command": exe, "args": ["--mcp"] })
}

/// 读取 JSON 配置文件（不存在时返回空对象）
fn read_json(path: &std::path::Path) -> serde_json::Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}

/// 判断 JSON 配置中是否已有 fatfish 条目。
/// 支持 mcpServers（多数客户端）与 projects 内嵌 mcpServers（opencode）。
fn json_configured(root: &serde_json::Value) -> bool {
    let has = |v: &serde_json::Value| {
        v.get("mcpServers")
            .and_then(|m| m.get(SERVER_NAME))
            .is_some()
    };
    if has(root) {
        return true;
    }
    root.get("projects")
        .and_then(|p| p.as_object())
        .map(|projects| projects.values().any(has))
        .unwrap_or(false)
}

/// 在 JSON 配置的 mcpServers 对象中写入 / 删除 fatfish 条目
fn edit_mcp_servers(
    root: &mut serde_json::Value,
    enable: bool,
) -> Result<(), String> {
    let obj = root.as_object_mut().ok_or("配置文件根节点不是 JSON 对象")?;
    let servers = obj
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));
    let servers_obj = servers.as_object_mut().ok_or("mcpServers 不是 JSON 对象")?;
    if enable {
        servers_obj.insert(SERVER_NAME.to_string(), server_entry());
    } else {
        servers_obj.remove(SERVER_NAME);
    }
    Ok(())
}

/// opencode：写在 projects.<path>.mcp 下，格式不同（command + enabled）
fn edit_opencode(root: &mut serde_json::Value, enable: bool, exe: &str) -> Result<(), String> {
    let obj = root.as_object_mut().ok_or("配置文件根节点不是 JSON 对象")?;
    if !enable {
        if let Some(projects) = obj.get_mut("projects").and_then(|p| p.as_object_mut()) {
            for project in projects.values_mut() {
                if let Some(mcp) = project.get_mut("mcp").and_then(|m| m.as_object_mut()) {
                    mcp.remove(SERVER_NAME);
                }
            }
        }
        return Ok(());
    }
    let projects = obj
        .entry("projects")
        .or_insert_with(|| serde_json::json!({}));
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let project = projects
        .as_object_mut()
        .ok_or("projects 不是 JSON 对象")?
        .entry(format!("local://{cwd}"))
        .or_insert_with(|| serde_json::json!({}));
    let mcp = project
        .as_object_mut()
        .ok_or("project 不是 JSON 对象")?
        .entry("mcp")
        .or_insert_with(|| serde_json::json!({}));
    mcp.as_object_mut()
        .ok_or("mcp 不是 JSON 对象")?
        .insert(
            SERVER_NAME.to_string(),
            serde_json::json!({ "type": "local", "command": [exe, "--mcp"], "enabled": true }),
        );
    Ok(())
}

/// codex：TOML 配置，[mcp_servers.fatfish] 段
fn is_codex_configured(path: &std::path::Path) -> bool {
    std::fs::read_to_string(path)
        .map(|s| s.contains("[mcp_servers.fatfish]"))
        .unwrap_or(false)
}

fn edit_codex(path: &std::path::Path, enable: bool) -> Result<bool, String> {
    let Some(exe) = exe_path() else { return Ok(false) };
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = existing
        .lines()
        .map(|l| l.to_string())
        .collect();
    // 移除旧段
    let mut cleaned: Vec<String> = Vec::with_capacity(lines.len());
    let mut skipping = false;
    for line in lines.drain(..) {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            skipping = trimmed.starts_with("[mcp_servers.fatfish]");
        }
        if !skipping {
            cleaned.push(line);
        }
    }
    if enable {
        while cleaned
            .last()
            .map(|l| l.trim().is_empty())
            .unwrap_or(false)
        {
            cleaned.pop();
        }
        if !cleaned.is_empty() {
            cleaned.push(String::new());
        }
        cleaned.push(format!("[mcp_servers.{SERVER_NAME}]"));
        cleaned.push(format!("command = \"{}\"", exe.replace('\\', "\\\\")));
        cleaned.push("args = [\"--mcp\"]".to_string());
        cleaned.push(String::new());
    }
    std::fs::create_dir_all(path.parent().ok_or("无法定位配置目录")?)
        .map_err(|e| format!("创建目录失败: {e}"))?;
    std::fs::write(path, cleaned.join("\n")).map_err(|e| format!("写入配置失败: {e}"))?;
    Ok(true)
}

/// 获取所有支持 agent 客户端的配置状态
pub fn agent_statuses() -> Vec<AgentStatus> {
    const AGENTS: [(&str, &str); 7] = [
        ("claude_desktop", "Claude Desktop"),
        ("cursor", "Cursor"),
        ("windsurf", "Windsurf"),
        ("claude_code", "Claude Code"),
        ("codex", "Codex CLI"),
        ("opencode", "opencode"),
        ("vscode", "VS Code"),
    ];
    let Some(exe) = exe_path() else { return Vec::new() };
    let _ = exe;
    AGENTS
        .iter()
        .filter_map(|(id, name)| {
            let path = config_path(id)?;
            let config_found = path.exists();
            let configured = if path.exists() {
                match *id {
                    "codex" => is_codex_configured(&path),
                    _ => {
                        let root = read_json(&path);
                        json_configured(&root)
                    }
                }
            } else {
                false
            };
            Some(AgentStatus {
                id: id.to_string(),
                name: name.to_string(),
                config_found,
                configured,
            })
        })
        .collect()
}

/// 为指定 agent 写入或移除 MCP 配置，返回是否实际写入。
pub fn configure_agent(id: &str, enable: bool) -> Result<bool, String> {
    let Some(exe) = exe_path() else {
        return Err("无法获取应用路径".into());
    };
    let Some(path) = config_path(id) else {
        return Err(format!("不支持的客户端: {id}"));
    };

    if id == "codex" {
        return edit_codex(&path, enable);
    }

    let mut root = if path.exists() {
        read_json(&path)
    } else {
        serde_json::json!({})
    };

    if id == "opencode" {
        edit_opencode(&mut root, enable, &exe)?;
    } else {
        edit_mcp_servers(&mut root, enable)?;
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let content = serde_json::to_string_pretty(&root).map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(&path, content).map_err(|e| format!("写入配置失败: {e}"))?;
    Ok(true)
}
