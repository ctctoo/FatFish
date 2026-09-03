pub mod sqlite;

/// app_settings 表中的键名
pub mod settings {
    /// MCP 服务器开关
    pub const KEY_MCP_ENABLED: &str = "mcp_enabled";

    /// 发布：GitHub Personal Access Token（优先级低于已登录的 github_account）
    pub const KEY_GITHUB_PAT: &str = "github_pat";

    /// 发布：AI 润色配置（OpenAI 兼容）
    pub const KEY_AI_API_KEY: &str = "ai_api_key";
    pub const KEY_AI_BASE_URL: &str = "ai_base_url";
    pub const KEY_AI_MODEL: &str = "ai_model";

    /// AI 润色默认端点
    pub const DEFAULT_AI_BASE_URL: &str = "https://api.openai.com/v1";
    pub const DEFAULT_AI_MODEL: &str = "gpt-4o-mini";
}
