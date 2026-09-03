# 规格说明：应用内项目版本发布（Release）

日期：2026-09-03
状态：已批准（方案 A 为主体 + 鉴权改为「设置页 Token 为主，gh CLI 检测兜底」，放弃 OAuth Device Flow）

## 1. 目标

在 FatFish 内实现项目的版本发布流程：生成 Changelog → 打 tag 并推送 → 创建 GitHub Release → 上传构建产物 → 可选同步版本号。全程本地编排，符合 Local-First 定位。

成功标准：对一个 GitHub 托管的项目，用户在详情页点击「发布新版本」，走完向导后 GitHub 上出现带 Changelog 与 assets 的 Release，本地留下发布记录，时间线出现 release 活动。

## 2. 架构总览

遵循现有分层 `Vue → Pinia → tauriApi → command → service → repository`，Rust 侧新增：

| 单元 | 文件 | 职责 |
|---|---|---|
| 发布命令 | `src-tauri/src/commands/release.rs` | 发布编排入口、进度事件、发布历史查询 |
| 发布编排 | `src-tauri/src/services/release_service.rs` | 前置检查→版本号→tag→push→Release→assets 的状态机 |
| Changelog | `src-tauri/src/services/changelog_service.rs` | `git log` 解析 + conventional commits 分组 → Markdown |
| AI 润色 | `src-tauri/src/services/ai_service.rs` | 可选，OpenAI 兼容 chat completions 调用 |
| GitHub 鉴权 | `src-tauri/src/services/github_auth_service.rs` | Token 管理优先级、gh CLI 检测、token 验证 |
| 发布记录 | `src-tauri/src/repository/release_repository.rs` | `releases` 表读写 |

- GitHub REST 调用复用已有 `reqwest`（0.12, rustls）；assets 上传改走 `uploads.github.com`
- 上传为长耗时操作：唯一引入 async 之处，用 `tauri::async_runtime::spawn` 后台执行，通过 Tauri Event `release://progress` 向前端推送进度
- tag / push 沿用 `git_service.rs` 的子进程模式
- 版本号写入不做正则全局替换，按文件结构精准定位 version 字段，支持 `package.json`、`Cargo.toml`、`pyproject.toml`、`pom.xml`

## 3. 数据模型

新增 `releases` 表（加在 `db/sqlite.rs` 的 `execute_batch`，不使用遗留的 `migrations/*.sql`）：

```sql
CREATE TABLE IF NOT EXISTS releases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  version TEXT NOT NULL,
  tag_name TEXT NOT NULL,
  status TEXT NOT NULL,           -- preparing / tag_pushed / published / failed
  changelog TEXT,
  release_url TEXT,
  error_message TEXT,
  released_at TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_releases_project ON releases(project_id, created_at DESC);
```

每次发布成功/失败写一条记录，并写 `activities`（kind=`release`），时间线自动展示。

## 4. 发布流程

### 4.1 前置检查（向导第一步自动执行）

| 检查项 | 不满足时 |
|---|---|
| origin remote 存在且为 github.com（复用 `normalize_remote_url`） | 阻断，指引先配置 remote |
| git CLI 可用 | 阻断 |
| 工作区 dirty | 仅提示，不阻断 |
| Token 有效且对 `owner/repo` 有推送权限（`GET /repos/{owner}/{repo}`） | 阻断，区分 401/403 |

`owner/repo` 从 remote URL 解析（https 与 ssh 两种形式均支持）。

### 4.2 向导步骤

1. **版本确认**：建议版本号 = 上一个 tag + 1（用户可改）；可选勾选「同步版本号到项目文件」→ 写入后自动 `git commit`（仅包含被改动的版本文件）
2. **Changelog**：`git log <上个tag>..HEAD --pretty=format:%H%x1f%s%x1f%b`，按 conventional commits 分组：✨ Features / 🐛 Fixes / ⚡ Performance / ♻ Refactor / 📝 Docs / 🔧 Others；含 `BREAKING CHANGE` 时置顶标注。生成可编辑 Markdown；「AI 润色」为可选按钮（需配置 API Key）
3. **产物选择**：文件选择器（dialog plugin）多选本地文件作为 assets，显示文件大小
4. **执行**：`git tag <tag>` → `git push origin <tag>` → `POST /repos/{o}/{r}/releases` → 逐个上传 assets（带进度）→ 结果页展示 Release 链接。任一步失败记 `failed` 并保留已完成进度，支持重试

标记选项：Draft / Pre-release 各一个开关。

## 5. 鉴权设计

优先级：**设置页 PAT → gh CLI → 未配置**

- PAT（`repo` scope）存 `app_settings` 表，key=`github_pat`，与现有明文存储惯例一致（keyring 加密留作后续独立事项）
- gh CLI 检测：`gh auth token` 成功输出即视为可用；设置页与向导中显示「检测到 gh CLI 已登录，可直接使用」
- Token 验证：`GET /user`，显示 login；无效时明确报错
- AI Key 存 `app_settings`，key=`ai_api_key`；OpenAI 兼容 base URL 可配，key=`ai_base_url`（默认 `https://api.openai.com/v1`）

## 6. 前端 UI

- **入口**：`ProjectDetailView.vue` 的「Dev」区块新增「发布新版本」按钮；区块下方列出历史发布（version / 状态 / 日期 / Release 链接）
- **发布向导** `src/components/project/ReleaseWizard.vue`：4 步进度条向导；执行页实时监听 `release://progress` 事件显示当前步骤与上传百分比；失败可重试
- **设置页**：新增「发布 / GitHub」区块——PAT 输入与验证、gh CLI 状态卡、AI 润色 Key 配置
- **i18n**：`src/i18n/messages.ts` 中英各加 `release.*` 词条
- Token 不传回前端明文，只显示掩码（如 `ghp_****`）

## 7. 错误处理

- 沿用「绝不 panic、失败降级」风格
- git 失败 → 结构化错误码（tag 已存在 / push 被拒 / 认证失败）
- GitHub API 失败 → 区分 401（token 无效）/ 403（权限不足）/ 422（tag 已存在或字段非法），给出行动建议
- 所有失败落库 `releases.status=failed` + `error_message`，UI 提供重试入口

## 8. 测试

- Rust 单测（不依赖网络）：conventional commits 分组解析、4 种版本文件 version 写入、remote URL → owner/repo 解析
- GitHub 调用层薄封装，集成测试需真实 token，默认 `#[ignore]`
- 向导流程手动验收清单

## 9. 范围外（YAGNI）

自动构建打包、Gitee/GitLab 多平台、keyring 加密、发布审批流、OAuth Device Flow。
