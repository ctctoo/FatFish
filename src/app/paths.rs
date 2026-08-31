use std::fs;
use std::path::{Path, PathBuf};

use directories::{BaseDirs, ProjectDirs};
use rusqlite::Connection;

use crate::database;
use crate::error::{AppError, AppResult};

/// 应用目录常量：与审计文档第 7 节的目标数据目录结构一致。
pub const APP_QUALIFIER: &str = "";
pub const APP_ORG: &str = "";
pub const APP_NAME: &str = "ProjectHub";
pub const DB_FILE_NAME: &str = "projecthub.db";

/// 旧版 Tauri 应用标识（tauri.conf.json 中的 identifier）。
const LEGACY_APP_DIR: &str = "com.fatfish.app";
const LEGACY_DB_FILE: &str = "fatfish.db";

/// Native 应用的数据目录服务。
///
/// 布局（Windows：`%APPDATA%\ProjectHub\...`）：
/// ```text
/// ProjectHub/
/// ├── data/
/// │   ├── projecthub.db
/// │   ├── attachments/
/// │   ├── screenshots/
/// │   └── backups/
/// ├── cache/
/// └── logs/
/// ```
#[derive(Debug, Clone)]
pub struct Paths {
    root: PathBuf,
}

impl Paths {
    /// 基于平台数据目录构造。失败表示系统数据目录不可用。
    pub fn new() -> AppResult<Self> {
        let dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .ok_or_else(|| AppError::DataDir("无法解析平台数据目录".into()))?;
        Ok(Self {
            root: dirs.data_dir().to_path_buf(),
        })
    }

    /// 测试/嵌入式使用：显式指定根目录（该目录可不存在）。
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn data_dir(&self) -> PathBuf {
        self.root.join("data")
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir().join(DB_FILE_NAME)
    }

    pub fn attachments_dir(&self) -> PathBuf {
        self.data_dir().join("attachments")
    }

    pub fn screenshots_dir(&self) -> PathBuf {
        self.data_dir().join("screenshots")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.data_dir().join("backups")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// 创建全部目录（幂等）。
    pub fn ensure_dirs(&self) -> AppResult<()> {
        for dir in [
            self.data_dir(),
            self.attachments_dir(),
            self.screenshots_dir(),
            self.backups_dir(),
            self.cache_dir(),
            self.logs_dir(),
        ] {
            fs::create_dir_all(&dir).map_err(|e| {
                AppError::DataDir(format!("创建目录失败: {} ({e})", dir.display()))
            })?;
        }
        Ok(())
    }

    /// 平台标准位置下可能存在的旧版 FatFish 数据库候选路径。
    ///
    /// Tauri 2 的 `app_data_dir()` 在 Windows 为 `%APPDATA%/<identifier>`，
    /// 在 macOS/Linux 为平台数据目录下的 `<identifier>`，identifier 为
    /// `com.fatfish.app`。逐个探测第一个存在的文件。
    pub fn legacy_candidates(&self) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Some(base) = BaseDirs::new() {
            candidates.push(base.data_dir().join(LEGACY_APP_DIR).join(LEGACY_DB_FILE));
        }
        candidates
    }

    /// 打开（必要时创建）Native 数据库。
    ///
    /// 流程：
    /// 1. Native 库已存在：直接打开（内部走版本化 migration）。
    /// 2. 不存在且探测到旧库：先在 `backups/` 生成一份只读快照，再把旧库
    ///    复制为 Native 库后打开迁移。**只复制、绝不移动旧库**，失败时旧库
    ///    与快照均保留。
    /// 3. 都不存在：创建空库。
    pub fn open_or_migrate(&self) -> AppResult<Connection> {
        self.open_or_migrate_from(&self.legacy_candidates())
    }

    /// 带自定义候选路径的打开/迁移入口，供测试注入虚拟旧库。
    pub fn open_or_migrate_from(&self, candidates: &[PathBuf]) -> AppResult<Connection> {
        self.ensure_dirs()?;
        let db_path = self.db_path();

        if db_path.exists() {
            return database::open(&db_path);
        }

        let Some(legacy) = candidates
            .iter()
            .find(|p| p.is_file())
            .cloned()
        else {
            return database::open(&db_path);
        };

        let backup_path = self.backups_dir().join(format!(
            "fatfish-legacy-backup-{}.db",
            crate::database::now().replace([':', '.'], "-")
        ));
        fs::copy(&legacy, &backup_path).map_err(|e| {
            AppError::Io(std::io::Error::new(
                e.kind(),
                format!("备份旧库失败: {} -> {} ({e})", legacy.display(), backup_path.display()),
            ))
        })?;

        // 只复制旧库文件；原文件保持原样（路径不移动保证）。
        fs::copy(&legacy, &db_path).map_err(|e| {
            AppError::Io(std::io::Error::new(
                e.kind(),
                format!("复制旧库失败: {} -> {} ({e})", legacy.display(), db_path.display()),
            ))
        })?;

        let conn = database::open(&db_path)?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at)
             VALUES ('migration.origin', ?1, ?2)",
            rusqlite::params![
                format!("{}|{}", legacy.display(), backup_path.display()),
                crate::database::now()
            ],
        )
        .map_err(AppError::from)?;
        tracing::info!(from = %legacy.display(), to = %db_path.display(), "legacy database adopted");
        Ok(conn)
    }
}
