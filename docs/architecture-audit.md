# Project Hub Native Architecture Audit

审计日期：2026-08-31  
审计分支：`slint`

## 1. 审计结论

当前仓库是一个可运行的 Vue 3 + Vite + Tauri 2 桌面 MVP。Rust 侧已经具备较清晰的 `command -> service -> repository -> SQLite` 分层，并覆盖项目、Todo、Tag、Collection、Link、目录扫描和只读 Git 信息。

目标计划要求迁移到纯 Rust + Slint Native Desktop。当前仓库尚未包含 Slint、独立 Rust workspace、原生窗口层或目标数据目录结构，因此这不是前端替换，而是一次保留数据兼容性的应用边界重建。

当前产品名称和包标识仍为 `FatFish`（`package.json`、`src-tauri/Cargo.toml`、README），计划名称使用 `Project Hub`。本轮不擅自改名；正式迁移前应将显示名称、可执行文件名、数据库文件名和升级渠道统一决定，并提供兼容旧数据路径的迁移逻辑。

## 2. 当前架构

```text
Vue 3 + TypeScript + Pinia + Vue Router
                |
                v
         Tauri 2 WebView
                |
                v
Rust commands -> services -> repositories -> rusqlite(SQLite)
                |
                v
        app_data_dir()/fatfish.db
```

### 入口和构建

| 区域 | 当前实现 | 位置 |
| --- | --- | --- |
| Web 入口 | Vite + Vue | `src/main.ts`、`src/App.vue` |
| 页面 | Home、Projects、Project Detail、Todos、Tags、Collections、Settings | `src/views/` |
| 状态 | Pinia stores | `src/stores/` |
| 桌面桥接 | `invoke`、dialog、opener | `src/services/tauri.ts`、`package.json` |
| Native 容器 | Tauri 2 + 系统 WebView | `src-tauri/src/lib.rs`、`src-tauri/tauri.conf.json` |
| Rust 数据访问 | `rusqlite` bundled | `src-tauri/src/db/`、`src-tauri/src/repository/` |
| 当前数据库位置 | Tauri `app_data_dir()/fatfish.db` | `src-tauri/src/lib.rs:19-24` |

## 3. 已实现能力

| 能力 | 当前状态 | 说明 |
| --- | --- | --- |
| Project CRUD | 已实现 | 名称、路径、描述、状态、封面 emoji/颜色、备注、收藏、最近打开时间 |
| Project scan/import | 已实现 | 扫描根目录一级子目录，识别 Java/Android/JS/TS/Rust/Go/Python |
| Todo CRUD | 已实现 | 标题、完成状态、项目、截止日期、排序；没有优先级 |
| Tag | 已实现 | CRUD、颜色、项目多对多关联 |
| Collection | 已实现 | CRUD、项目多对多关联 |
| Links | 已实现 | CRUD、类型字段；没有显式排序字段 |
| Notes | 部分实现 | 使用 `projects.notes` 保存 Markdown/文本，没有独立 notes 模型 |
| Search | 部分实现 | repository 组合条件查询，覆盖项目字段、Tag、Collection、Link；不是 FTS5，搜索未注册为 Tauri command |
| Git | 可选已实现 | 读取 remote、branch、commit、dirty；Git 失败时降级 |
| Activity | 部分实现 | 已有表和写入逻辑，详情展示依赖 Vue 页面 |
| Screenshots | 未实现 | 无表、无文件服务、无 UI |
| Attachments | 未实现 | 无表、无文件服务、无 UI |
| Custom fields | 未实现 | 无表、无数据模型、无 UI |
| Settings | UI/持久化待核实 | 当前有 Settings store，但 Rust 数据库没有 settings 表；需作为迁移任务验证 |
| Backup/restore | 未实现 | 无导出、导入、恢复前备份 |
| Updater | Tauri 配置依赖 | 计划要求移除 Tauri Updater；当前没有 Rust Native updater |
| Keyboard/desktop interaction | Web UI 已有部分 | `Ctrl+K` 等交互主要由 Vue/DOM 实现，尚未有 Slint 输入层 |

## 4. 当前数据库结构

数据库初始化集中在 `src-tauri/src/db/sqlite.rs`，通过 `CREATE TABLE IF NOT EXISTS` 和运行时 `ALTER TABLE` 完成，尚未使用版本化 migration 文件或 `schema_version` 表。

现有表：

- `projects`: `id`、`name`、`path`、`description`、`status`、`favorite`、`cover_emoji`、`cover_color`、`notes`、`language`、时间字段。
- `tags`、`project_tags`: Tag 及项目多对多关系。
- `collections`、`project_collections`: Collection 及项目多对多关系。
- `links`: 项目链接，包含 `title`、`url`、`link_type`。
- `todos`: 标题、完成状态、可选项目、截止日期、排序。
- `activities`: 项目动态时间线。
- `git_info`: 项目 Git 派生信息。

已有兼容逻辑：

- `projects` 缺失 `cover_emoji`、`cover_color`、`notes` 时补列。
- 旧状态 `DEVELOPING` / `MAINTAINING` 转为 `IN_PROGRESS`。
- 旧 `projects.github_url` 存在时写入 `links` 的 GitHub 记录。

迁移风险：

- 初始化前没有 schema version，无法可靠表达 v1/v2/v3，也无法区分已经执行过的临时迁移。
- `github_url` 不在当前 `CREATE TABLE projects` 定义中，旧库迁移依赖 SQLite 仍保留该列；需要在正式迁移中覆盖旧版本数据库。
- 所有表使用 SQLite 文本时间和字符串枚举；Native core 应继续兼容现有值，避免一次性重命名造成数据损失。
- 目标字段 `priority`、`custom_fields`、截图、附件、设置、备份元数据还没有来源，需要以新增 nullable/default 字段或新表迁移。

## 5. 可复用代码

可以优先抽离或改造到 Native core：

- `models/` 中的 Project、Todo、Tag、Collection、Link、Activity、ScannedProject 数据结构及 camelCase 序列化边界。
- `repository/` 中的 SQLite CRUD 和多对多关系操作。
- `services/project_service.rs` 的项目校验、重复路径保护、扫描导入、Git 信息补全和活动记录规则。
- `services/scanner_service.rs` 的一级目录扫描和语言识别规则。
- `services/git_service.rs` 的 remote 规范化、branch/commit/dirty 读取和无 Git 降级行为。
- 现有 `docs/MvpPlan.md`、`docs/UIPlan.md` 中的产品行为与视觉规格。

这些代码目前耦合于 Tauri command 的 `State<Db>` 和 `Result<T, String>` 边界，迁移时应把业务层错误改为统一 `AppError`，把应用状态从 Tauri `Manager` 中移出。

## 6. 需要保留但最终废弃的代码

在 Native UI 达到 parity 前，不删除以下内容：

- `src/` 下 Vue 页面、组件、stores、router、styles 和 `src/services/tauri.ts`。
- `package.json`、Vite 配置、Node 依赖及 Tauri frontend 构建配置。
- `src-tauri/src/commands/` 的现有命令，作为行为和数据兼容参考。

达到 parity 后，才可以移除 WebView 相关代码。`promo.html`、网站部署文件和 GitHub Pages workflow 属于宣传站点边界，不应因为桌面 Native 重构而无条件删除。

## 7. 目标迁移方案

### 数据目录

采用平台数据目录下的稳定应用目录，程序包和用户数据分离：

```text
ProjectHub/
├── data/
│   ├── projecthub.db
│   ├── attachments/
│   ├── screenshots/
│   └── backups/
├── cache/
└── logs/
```

升级程序只替换 app binaries/assets，不删除 `data/`。首次启动 Native 版本时：

1. 定位 Native 数据目录。
2. 识别新数据库是否存在。
3. 若不存在，探测旧 Tauri 数据目录中的 `fatfish.db` 和兼容路径。
4. 对数据库执行版本化 migration，并复制附件/截图到新目录（当前版本没有这些文件，需要为未来版本预留）。
5. 在迁移前生成一次只读快照/备份，并记录迁移结果。
6. 只有成功校验后才写入迁移标记；失败时保留旧库和备份。

### Schema 版本

下一步应把现有内联 SQL 固化为 `migrations/0001_initial.sql`，并新增 `schema_version` 或 SQLite `user_version` 管理版本。建议增量顺序：

- v1：将当前生产表结构原样固化。
- v2：补齐 `settings`、Todo `priority`、Link `sort_order` 和必要索引。
- v3：加入 `custom_fields` / `project_custom_fields`。
- v4：加入 `attachments` / `screenshots` 元数据表，文件留在 data 目录。
- v5：加入 FTS5 索引和触发器，搜索行为保持与旧组合查询兼容。

具体版本号应以真实旧库样本验证后确定，不能假定所有用户数据库都来自当前代码版本。

## 8. Native 重构分层建议

```text
src/
├── app/          # 应用状态、路由/页面状态、命令分发
├── project/      # 项目领域模型与服务
├── todo/
├── collection/
├── tag/
├── link/
├── attachment/
├── screenshot/
├── search/
├── settings/
├── database/     # 连接、migration、repository
├── filesystem/   # 路径、扫描、watcher、文件复制
├── backup/
└── updater/
ui/               # Slint only
```

建议 UI 只接收 view model 和事件回调，不直接执行 SQL、文件系统或进程命令。所有用户可见错误通过 `AppError` 映射为稳定的本地化文案，避免显示 panic 或底层堆栈。

## 9. 第一批实施边界

本轮审计后，第一批实现应按以下可验证顺序推进：

1. 建立不依赖 Tauri 的 Rust core crate/module 边界。
2. 固化 schema migration runner，并为当前表增加迁移测试。
3. 把数据目录、路径规范化和 AppError 建立起来。
4. 在 core 测试覆盖 Project/Todo/Tag/Collection/Link CRUD、旧库迁移和路径不移动保证。
5. 再接入 Slint app shell 和首页，逐步替换 Vue 页面。

Updater、文件 watcher、复杂拖拽和 Windows release 应在 core 数据迁移稳定后实现，否则无法将运行失败归因到正确层级。

## 10. 基线验证

审计时运行：

- `npm run build`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml`：通过，3 passed，0 failed。
- Rust 有 8 个 dead-code warning，主要是状态常量和未注册的 `search_service::search`；不影响当前构建，但 Native core 接入时应清理。

