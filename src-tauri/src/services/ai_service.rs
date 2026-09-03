use reqwest::blocking::Client;
use serde_json::{json, Value};

use crate::db::settings;

const TIMEOUT_SECS: u64 = 60;

/// 调用 OpenAI 兼容接口润色 Changelog。
/// 约束模型：保留事实、面向用户、不虚构。失败时返回 Err（润色永不阻塞发布，调用方回退原稿）。
pub fn polish_changelog(api_key: &str, base_url: &str, model: &str, markdown: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("未配置 AI API Key".to_string());
    }
    let base = if base_url.trim().is_empty() {
        settings::DEFAULT_AI_BASE_URL.to_string()
    } else {
        base_url.trim().trim_end_matches('/').to_string()
    };
    let model = if model.trim().is_empty() {
        settings::DEFAULT_AI_MODEL.to_string()
    } else {
        model.trim().to_string()
    };

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "你是软件发行说明编辑。把基于 Git 提交记录的 Changelog 润色为面向最终用户的发行说明。规则：1) 不虚构任何功能或修复；2) 保留 BREAKING 提示；3) 使用中文；4) 保持 Markdown 结构（版本标题与分组小节）；5) 语言简洁。"
            },
            {
                "role": "user",
                "content": markdown
            }
        ],
        "temperature": 0.3
    });

    let resp = Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("初始化 HTTP 客户端失败：{e}"))?
        .post(format!("{base}/chat/completions"))
        .bearer_auth(api_key.trim())
        .json(&body)
        .send()
        .map_err(|e| format!("无法连接 AI 服务：{e}"))?;

    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    if !status.is_success() {
        // 常见错误体里带 message 字段，尽量透出
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(msg) = v["error"]["message"].as_str() {
                return Err(format!("AI 服务返回错误（HTTP {status}）：{msg}"));
            }
        }
        return Err(format!("AI 服务返回错误（HTTP {status}）"));
    }

    let value: Value = serde_json::from_str(&text).map_err(|e| format!("AI 返回了无法解析的响应：{e}"))?;
    let content = value["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "AI 响应中缺少生成内容".to_string())?;
    if content.trim().is_empty() {
        return Err("AI 返回了空内容".to_string());
    }
    Ok(content.trim().to_string())
}
