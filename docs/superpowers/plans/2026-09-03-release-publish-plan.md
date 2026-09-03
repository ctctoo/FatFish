# 实现计划：应用内项目版本发布（Release）

日期：2026-09-03
规格：`docs/superpowers/specs/2026-09-03-release-publish-design.md`

任务按依赖顺序排列，每阶段完成后可独立验证。实现过程中不改动与发布无关的现有代码。

## 阶段 1：数据层

### 1.1 `releases` 表
- `src-tauri/src/db/sqlite.rs`：在 `execute_batch` 中新增 `releases` 表 + `idx_releases_project` 索引（字段见规格 §3）
- 验证：启动应用，确认建表无报错

### 1.2 模型与仓库
- 新建 `src-tauri/src/models/release.rs`：`ReleaseRecord`（serde Serialize/SerializeAll camelCase）
- 新建 `src-tauri/src/repository/release_repository.rs`：`insert` / `update_status` / `list_by_project` / `latest_for_project`
- 验证：cargo check

## 阶段 2：Changelog 与版本文件（纯本地，可单测）

### 2.1 git 能力扩展
- `src-tauri/src/services/git_service.rs` 新增：
  - `list_tags(dir)`：`git tag --sort=-creatordate`，解析最新语义化版本 tag
  - `log_since_tag(dir, tag)`：`git log <tag>..HEAD --pretty=format:%H%x1f%s%x1f%b%x1e`（失败或无 tag 时退化为全量 log）
  - `commit_files(dir, paths, message)` / `tag_and_push(dir, tag)`：子进程执行，返回结构化错误

### 2.2 Changelog 生成
- 新建 `src-tauri/src/services/changelog_service.rs`：
  - conventional commits 前缀解析（feat/fix/perf/refactor/docs/chore/test/build/ci + `!` 与 footer `BREAKING CHANGE`）
  - 分组生成 Markdown（✨ Features / 🐛 Fixes / ⚡ Performance / ♻ Refactor / 📝 Docs / 🔧 Others，BREAKING 置顶）
  - 非 conventional 提交归入 Others
- 单测：解析与分组（含 breaking、混合中英文 message）

### 2.3 版本文件写入器
- 新建 `src-tauri/src/services/version_file_service.rs`：
  - 按项目技术栈探测文件（package.json JSON 解析、Cargo.toml TOML 行定位、pyproject.toml、pom.xml）
  - `bump_version(dir, new_version) -> Vec<PathBuf>` 返回被改动的文件
  - 仅当版本确实变化时写入
- 单测：4 种文件读写往返

## 阶段 3：GitHub 集成

### 3.1 鉴权服务
- 新建 `src-tauri/src/services/github_auth_service.rs`：
  - `resolve_token(db)`：app_settings(`github_pat`) → `gh auth token` 子进程 → None
  - `verify_token(token) -> login`：`GET /user`
  - `check_repo_access(token, owner, repo)`：`GET /repos/{owner}/{repo}`，区分 401/403/404
  - `parse_owner_repo(remote_url)`：https 与 ssh 两种形式（复用 `normalize_remote_url` 结果）
- 单测：owner/repo 解析（`https://github.com/a/b.git`、`git@github.com:a/b.git`、带 `.git`/不带）

### 3.2 AI 润色服务
- 新建 `src-tauri/src/services/ai_service.rs`：
  - `polish_changelog(base_url, api_key, markdown)`：OpenAI 兼容 `POST /chat/completions`，系统提示词约束「保留事实、面向用户、不虚构」
  - 失败返回原稿 + 错误信息（润色永远不阻塞发布）
- reqwest 客户端配置：30s 超时

### 3.3 发布编排
- 新建 `src-tauri/src/services/release_service.rs`：状态机
  1. 前置检查（规格 §4.1）
  2. 可选版本号同步 + commit
  3. `git tag` → `git push origin <tag>`
  4. `POST /repos/{o}/{r}/releases`（tag_name, name, body, draft, prerelease）
  5. 逐个上传 assets：`POST https://uploads.github.com/repos/{o}/{r}/releases/{id}/assets?name=`，`application/octet-stream`，逐文件推送 `release://progress` 事件（payload：step、当前文件、已传字节/总字节）
  6. 成功：`releases.status=published` + `release_url` + activity(kind=`release`)；失败：`status=failed` + `error_message`，保留已完成阶段
- 重试：从未完成阶段继续（tag 已存在视为该阶段已完成）

### 3.4 命令层
- 新建 `src-tauri/src/commands/release.rs`：
  - `get_release_context(project_id)`：前置检查结果、最新 tag、建议版本、原始提交列表、生成的初稿 changelog、鉴权状态
  - `polish_changelog(project_id, markdown)`（async）
  - `start_release(project_id, params)`（async，spawn 编排并立即返回 release 记录 id）
  - `list_releases(project_id)`
- `src-tauri/src/lib.rs`：注册 4 个命令
- 验证：`cargo test`

## 阶段 4：前端

### 4.1 类型和 API 层
- `src/types.ts`：`ReleaseRecord`、`ReleaseContext`、`ReleaseParams`、`ReleaseProgress`
- `src/services/tauri.ts`：`getReleaseContext` / `polishChangelog` / `startRelease` / `listReleases` / `listenReleaseProgress`
- 新建 `src/stores/release.ts`：向导状态（当前步、context、changelog 草稿、assets、执行进度、错误/重试）

### 4.2 发布向导
- 新建 `src/components/project/ReleaseWizard.vue`：4 步向导
  - 步骤 1 版本确认（建议值 + 同步版本号开关 + 前置检查结果展示）
  - 步骤 2 Changelog 编辑（textarea/预览切换 + AI 润色按钮 + loading 态）
  - 步骤 3 产物选择（dialog plugin 多选 + 大小展示 + Draft/Pre-release 开关）
  - 步骤 4 执行（监听 `release://progress`，步骤打勾 + 上传进度条；失败显示原因与重试；成功显示 Release 链接）

### 4.3 页面集成
- `src/views/ProjectDetailView.vue`：Dev 区块加「发布新版本」按钮打开向导；下方历史发布列表（version/状态徽标/日期/外链）
- `src/views/SettingsView.vue`：新增「发布 / GitHub」区块——PAT 输入（保存即 `verify_token`，显示掩码与 login）、gh CLI 检测状态卡、AI base URL + Key 配置
- 需要的 Rust 辅助命令：`get_publish_settings` / `set_publish_settings`（token 掩码返回，不回传明文）
- `src/i18n/messages.ts`：中英各加 `release.*` / `settings.publish.*` 词条

## 阶段 5：验收

- `cargo test` 全绿；`npm run build` 无 TS 报错
- 手动验收清单：
  1. 非 GitHub / 无 remote 项目：向导第一步给出明确阻断提示
  2. 未配置 token 且无 gh：指引到设置页
  3. 正常发布：tag 推送、Release 出现在 GitHub、assets 可下载、时间线出现 release 活动
  4. 中途断网失败：记录 failed，重试可续
  5. tag 已存在（422）：正确提示
  6. AI 未配置时润色按钮隐藏/禁用且不影响发布

## 明确不做

自动构建打包、Gitee/GitLab、keyring、审批流、OAuth Device Flow（见规格 §9）。
