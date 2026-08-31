#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Native 桌面入口：把首页 UI 接到核心层（数据目录、迁移、repository）。
//!
//! 窗口内容由 `ui/app.slint` 定义；这里只负责：
//! 1. 打开（必要时迁移）数据库；
//! 2. 用 repository 查询结果填充首页模型；
//! 3. 把 UI 回调映射为 repository 写操作后刷新模型。

use std::sync::{Arc, Mutex};

use chrono::Timelike;
use project_hub::app::paths::Paths;
use project_hub::error::{AppError, AppResult};
use project_hub::models::project::{Project, ProjectFilter};
use project_hub::repository::{project as project_repo, todo as todo_repo};

slint::include_modules!();

fn main() -> AppResult<()> {
    let _log_guard = init_logging();

    let paths = Paths::new()?;
    let conn = Arc::new(Mutex::new(paths.open_or_migrate()?));

    let app = AppWindow::new()
        .map_err(|e| AppError::InvalidInput(format!("创建主窗口失败: {e}")))?;

    {
        let conn = conn.lock().expect("database mutex poisoned");
        refresh_home(&app, &conn)?;
    }

    // 待办勾选：直接写入数据库后刷新列表。
    let ui = app.as_weak();
    let conn_handle = Arc::clone(&conn);
    app.on_toggle_todo(move |id, done| {
        let Some(ui) = ui.upgrade() else { return };
        let conn = conn_handle.lock().expect("database mutex poisoned");
        if let Err(e) = todo_repo::set_done(&conn, id.as_str(), done) {
            tracing::error!(error = %e, id = %id, "toggle todo failed");
            return;
        }
        if let Err(e) = refresh_home(&ui, &conn) {
            tracing::error!(error = %e, "refresh home after todo toggle failed");
        }
    });

    // 收藏切换：写入数据库后刷新（排序与封面都会变化）。
    let ui = app.as_weak();
    let conn_handle = Arc::clone(&conn);
    app.on_toggle_favorite(move |id, favorite| {
        let Some(ui) = ui.upgrade() else { return };
        let conn = conn_handle.lock().expect("database mutex poisoned");
        if let Err(e) = project_repo::set_favorite(&conn, id.as_str(), favorite) {
            tracing::error!(error = %e, id = %id, "toggle favorite failed");
            return;
        }
        if let Err(e) = refresh_home(&ui, &conn) {
            tracing::error!(error = %e, "refresh home after favorite toggle failed");
        }
    });

    app.run()
        .map_err(|e| AppError::InvalidInput(format!("运行主窗口失败: {e}")))?;
    Ok(())
}

/// 用数据库最新状态刷新首页模型（最近项目 + 待办 + 问候语）。
fn refresh_home(app: &AppWindow, conn: &rusqlite::Connection) -> AppResult<()> {
    let recent = project_repo::list(conn, &ProjectFilter { recent: Some(true), ..Default::default() })?;
    let todos = todo_repo::list(conn)?;

    app.set_has_projects(!recent.is_empty());
    app.set_projects(model_rc(
        recent.iter().map(to_project_row).collect::<Vec<_>>(),
    ));
    app.set_todos(model_rc(
        todos
            .iter()
            .map(|t| TodoRow {
                id: t.id.clone().into(),
                title: t.title.clone().into(),
                done: t.done,
                project_name: t.project_name.clone().unwrap_or_default().into(),
            })
            .collect::<Vec<_>>(),
    ));
    app.set_greeting(greeting().into());
    Ok(())
}

/// 把 Rust Vec 包装成 Slint 共享模型。
fn model_rc<T: Clone + 'static>(items: Vec<T>) -> slint::ModelRc<T> {
    slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(items)))
}

fn to_project_row(p: &Project) -> ProjectRow {
    ProjectRow {
        id: p.id.clone().into(),
        name: p.name.clone().into(),
        status: p.status.clone().into(),
        description: p.description.clone().unwrap_or_default().into(),
        cover_emoji: p.cover_emoji.clone().unwrap_or_else(|| "📁".into()).into(),
        cover_color: cover_brush(p.cover_color.as_deref()),
        favorite: p.favorite,
    }
}

/// 把数据库里的 `#rrggbb` 转成 Slint Brush；缺省或非法时用品牌蓝兜底。
fn cover_brush(hex: Option<&str>) -> slint::Brush {
    if let Some(hex) = hex.and_then(parse_hex_rgb) {
        return slint::Brush::from(slint::Color::from_rgb_u8(hex[0], hex[1], hex[2]));
    }
    slint::Brush::from(slint::Color::from_rgb_u8(0x2f, 0x6f, 0xed))
}

fn parse_hex_rgb(s: &str) -> Option<[u8; 3]> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some([((v >> 16) & 0xff) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8])
}

/// 按时段生成问候语（后续接 profile 后可在前面加称呼）。
fn greeting() -> String {
    match chrono::Local::now().hour() {
        5..=10 => "早上好，开始新的一天吧".into(),
        11..=13 => "中午好，注意休息".into(),
        14..=17 => "下午好".into(),
        18..=23 => "晚上好".into(),
        _ => "夜深了，注意休息".into(),
    }
}

/// 初始化日志：写入 `logs/` 目录下的滚动文件；目录不可用时退回临时目录。
fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = Paths::new()
        .map(|p| p.logs_dir())
        .unwrap_or_else(|_| std::env::temp_dir());
    let _ = std::fs::create_dir_all(&log_dir);

    let (writer, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(&log_dir, "project-hub.log"));
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "project_hub=info".into()),
        )
        .with_ansi(false)
        .with_writer(writer)
        .init();
    guard
}
