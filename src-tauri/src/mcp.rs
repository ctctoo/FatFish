use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt, handler::server::wrapper::Parameters,
    model::*, schemars, tool, tool_handler, tool_router, transport::stdio,
};
use serde_json::json;

use crate::models::project::{ProjectFilter, ProjectInput};
use crate::models::todo::TodoInput;
use crate::repository::{activity_repository, git_repository, project_repository, todo_repository};
use crate::services::{git_service, project_service};

/// 传给 MCP 工具的数据库句柄。
/// Tauri 的 `Db` state 不便直接共享，这里用独立连接按需打开。
#[derive(Clone)]
pub struct McpDb {
    pub db_path: Arc<std::path::PathBuf>,
}

pub struct FatFishMcp {
    db: McpDb,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProjectNameArgs {
    #[schemars(description = "项目名称（不填则返回所有项目概览）")]
    pub project: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateProjectArgs {
    #[schemars(description = "项目名称，用于定位要修改的项目")]
    pub project: String,
    #[schemars(description = "新的项目描述")]
    pub description: Option<String>,
    #[schemars(description = "要添加到项目备注（Markdown）中的计划内容，追加到已有备注末尾")]
    pub plan: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddTodoArgs {
    #[schemars(description = "任务内容")]
    pub title: String,
    #[schemars(description = "关联的项目名称（可选）")]
    pub project: Option<String>,
    #[schemars(description = "截止日期，格式 YYYY-MM-DD（可选）")]
    pub due_date: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GitLogArgs {
    #[schemars(description = "项目名称")]
    pub project: String,
    #[schemars(description = "返回的提交数量，默认 10，最大 50")]
    pub limit: Option<u32>,
}

#[tool_router]
impl FatFishMcp {
    pub fn new(db: McpDb) -> Self {
        Self { db, tool_router: Self::tool_router() }
    }

    fn with_conn<T>(
        &self,
        f: impl FnOnce(&rusqlite::Connection) -> Result<T, String>,
    ) -> Result<T, McpError> {
        let conn = rusqlite::Connection::open(self.db.db_path.as_path())
            .map_err(|e| McpError::internal_error(format!("打开数据库失败: {e}"), None))?;
        f(&conn).map_err(|e| McpError::internal_error(e, None))
    }

    fn resolve_project(&self, name: &str) -> Result<crate::models::project::Project, McpError> {
        let query = name.trim();
        if query.is_empty() {
            return Err(McpError::invalid_params("项目名称不能为空", None));
        }
        let project = self.with_conn(|conn| {
            let all = project_service::list(
                conn,
                ProjectFilter { query: Some(query.to_string()), ..Default::default() },
            )?;
            match all.iter().find(|p| p.name.eq_ignore_ascii_case(query)) {
                Some(p) => Ok(p.clone()),
                None => all.first().cloned().ok_or_else(|| format!("未找到项目: {query}")),
            }
        })?;
        Ok(project)
    }

    /// 工具 1：已有项目概览（名称、描述、地址）
    #[tool(description = "获取 FatFish 中已有项目的概览：项目名称、项目描述和项目地址（本地路径与 Git 远程地址）。可指定项目名称查询单个项目，或返回全部项目。")]
    fn project_overview(
        &self,
        Parameters(args): Parameters<ProjectNameArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = match args.project.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(name) => {
                let p = self.resolve_project(name)?;
                json!({
                    "name": p.name,
                    "description": p.description,
                    "path": p.path,
                    "remote_url": p.git_info.as_ref().and_then(|g| g.remote_url.clone()),
                    "status": p.status,
                    "favorite": p.favorite,
                })
            }
            None => {
                let projects = self.with_conn(|conn| {
                    project_service::list(conn, ProjectFilter::default())
                })?;
                json!(projects
                    .iter()
                    .map(|p| json!({
                        "name": p.name,
                        "description": p.description,
                        "path": p.path,
                        "remote_url": p.git_info.as_ref().and_then(|g| g.remote_url.clone()),
                        "status": p.status,
                        "favorite": p.favorite,
                    }))
                    .collect::<Vec<_>>())
            }
        };
        Ok(CallToolResult::success(vec![Content::text(text.to_string())]))
    }

    /// 工具 2：修改项目（描述 / 添加计划）
    #[tool(description = "修改指定项目的描述，或向项目备注添加计划内容。项目名称不匹配时返回错误。")]
    fn update_project(
        &self,
        Parameters(args): Parameters<UpdateProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        let project = self.resolve_project(&args.project)?;
        let description = args.description.clone();
        let plan = args.plan.clone();
        if description.is_none() && plan.is_none() {
            return Err(McpError::invalid_params(
                "至少提供 description 或 plan 之一",
                None,
            ));
        }
        let updated = self.with_conn(|conn| {
            let mut input = ProjectInput {
                name: project.name.clone(),
                path: project.path.clone(),
                description: description.clone(),
                status: Some(project.status.clone()),
                cover_emoji: project.cover_emoji.clone(),
                cover_color: project.cover_color.clone(),
                notes: project.notes.clone(),
            };
            if input.description.is_none() {
                input.description = project.description.clone();
            }
            if let Some(plan) = &plan {
                let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M");
                let block = format!("\n\n## 计划（{stamp}，由 Agent 添加）\n\n{plan}");
                input.notes = Some(match &project.notes {
                    Some(existing) => format!("{existing}{block}"),
                    None => block.trim_start().to_string(),
                });
            }
            project_service::update(conn, &project.id, input)
        })?;
        Ok(CallToolResult::success(vec![Content::text(json!({
            "success": true,
            "name": updated.name,
            "description": updated.description,
            "notes": updated.notes,
        }).to_string())]))
    }

    /// 工具 3：添加 Todo
    #[tool(description = "在 FatFish 中添加一条 Todo 记录，可关联项目和设置截止日期。")]
    fn add_todo(
        &self,
        Parameters(args): Parameters<AddTodoArgs>,
    ) -> Result<CallToolResult, McpError> {
        let title = args.title.trim().to_string();
        if title.is_empty() {
            return Err(McpError::invalid_params("任务内容不能为空", None));
        }
        let project_id = match args.project.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(name) => Some(self.resolve_project(name)?.id),
            None => None,
        };
        let todo = self.with_conn(|conn| {
            let todo = todo_repository::insert(conn, &TodoInput {
                title,
                project_id,
                due_date: args.due_date.clone(),
            })
            .map_err(|e| format!("数据库操作失败: {e}"))?;
            if let Some(project_id) = &todo.project_id {
                activity_repository::log(
                    conn,
                    project_id,
                    "todo",
                    &format!("创建 Todo：{}", todo.title),
                );
            }
            Ok(todo)
        })?;
        Ok(CallToolResult::success(vec![Content::text(json!({
            "success": true,
            "id": todo.id,
            "title": todo.title,
            "project": args.project,
            "due_date": todo.due_date,
        }).to_string())]))
    }

    /// 工具 4：读取项目 Git 记录
    #[tool(description = "读取指定项目的 Git 提交记录（提交哈希、作者、时间、说明）以及当前分支和最新提交摘要。")]
    fn git_log(
        &self,
        Parameters(args): Parameters<GitLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        let project = self.resolve_project(&args.project)?;
        let limit = args.limit.unwrap_or(10).clamp(1, 50);
        let path = std::path::Path::new(&project.path);
        if !path.join(".git").exists() {
            return Ok(CallToolResult::success(vec![Content::text(json!({
                "project": project.name,
                "git": "该目录不是 Git 仓库",
            }).to_string())]));
        }

        let run_git = |args: &[&str]| -> Option<String> {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(path)
                .output()
                .ok()?;
            out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        };

        let log = run_git(&[
            "log",
            &format!("-{limit}"),
            "--pretty=format:%H%x1f%an%x1f%aI%x1f%s",
        ])
        .unwrap_or_default();
        let commits: Vec<serde_json::Value> = log
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| {
                let parts: Vec<&str> = l.split('\x1f').collect();
                json!({
                    "hash": parts.first().copied().unwrap_or(""),
                    "author": parts.get(1).copied().unwrap_or(""),
                    "time": parts.get(2).copied().unwrap_or(""),
                    "message": parts.get(3).copied().unwrap_or(""),
                })
            })
            .collect();

        let summary = self.with_conn(|conn| {
            let info = git_service::collect_git_info(&project.path);
            git_repository::upsert(conn, &project.id, &info)
                .map_err(|e| format!("数据库操作失败: {e}"))?;
            project_repository::get(conn, &project.id)
                .map_err(|e| format!("数据库操作失败: {e}"))?
                .ok_or_else(|| "项目不存在".to_string())
                .map(|p| p.git_info)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json!({
            "project": project.name,
            "branch": summary.as_ref().and_then(|g| g.branch.clone()),
            "latest_commit": summary.as_ref().map(|g| json!({
                "hash": g.commit_hash,
                "message": g.commit_message,
                "time": g.commit_time,
            })),
            "is_dirty": summary.as_ref().and_then(|g| g.is_dirty),
            "remote_url": summary.as_ref().and_then(|g| g.remote_url.clone()),
            "commits": commits,
        }).to_string())]))
    }

    /// 工具 5：最近项目 + 收藏项目
    #[tool(description = "读取 FatFish 中最近打开的项目和收藏的项目列表。")]
    fn recent_and_favorite_projects(&self) -> Result<CallToolResult, McpError> {
        let recent = self.with_conn(|conn| {
            project_service::list(
                conn,
                ProjectFilter { recent: Some(true), ..Default::default() },
            )
        })?;
        let favorites = self.with_conn(|conn| {
            project_service::list(
                conn,
                ProjectFilter { favorite: Some(true), ..Default::default() },
            )
        })?;
        let brief = |p: &crate::models::project::Project| {
            json!({
                "name": p.name,
                "description": p.description,
                "path": p.path,
                "status": p.status,
                "last_opened_at": p.last_opened_at,
            })
        };
        Ok(CallToolResult::success(vec![Content::text(json!({
            "recent": recent.iter().map(brief).collect::<Vec<_>>(),
            "favorites": favorites.iter().map(brief).collect::<Vec<_>>(),
        }).to_string())]))
    }
}

#[tool_handler]
impl ServerHandler for FatFishMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "fatfish-project-hub".into(),
                title: Some("FatFish Project Hub".into()),
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "FatFish 项目管理工具：可查询项目概览、修改项目描述与添加计划、添加 Todo、读取 Git 记录、查看最近与收藏项目。"
                    .into(),
            ),
        }
    }
}

/// 在独立的 tokio 运行时中启动 stdio MCP 服务（阻塞当前线程直到连接关闭）。
pub fn run_mcp_server(db_path: std::path::PathBuf) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime for MCP server");
    rt.block_on(async move {
        let service = FatFishMcp::new(McpDb { db_path: Arc::new(db_path) });
        if let Ok(server) = service.serve(stdio()).await {
            let _ = server.waiting().await;
        }
    });
}
