/// Native core 的统一错误类型。
///
/// 所有层级（数据库、文件系统、数据目录、业务校验）都收敛到这里，
/// 最终由 UI 层映射为稳定的本地化文案，不向用户暴露底层堆栈。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库操作失败: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("文件系统操作失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("记录不存在: {0}")]
    NotFound(String),

    #[error("输入不合法: {0}")]
    InvalidInput(String),

    #[error("旧数据库探测失败: {0}")]
    Legacy(String),

    #[error("数据目录不可用: {0}")]
    DataDir(String),
}

impl AppError {
    /// 稳定的错误代码，供 UI 做文案映射与埋点。
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database",
            AppError::Io(_) => "io",
            AppError::NotFound(_) => "not_found",
            AppError::InvalidInput(_) => "invalid_input",
            AppError::Legacy(_) => "legacy_discovery",
            AppError::DataDir(_) => "data_dir",
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// 帮助定位问题：数据目录相关错误附带实际路径。
pub fn data_dir_err(action: &str, path: &std::path::Path, source: &std::io::Error) -> AppError {
    AppError::DataDir(format!("{action} 失败: {} ({})", path.display(), source))
}

pub fn legacy_err(action: &str, path: &std::path::Path) -> AppError {
    AppError::Legacy(format!("{action}: {}", path.display()))
}
