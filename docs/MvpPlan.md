# FatFish — MVP Plan

## 1. 项目定位

**FatFish** 是一个 Local-First 的桌面项目管理工具，用于统一管理本机开发项目文件夹，并通过项目元数据建立项目索引。

MVP 的核心价值：

> **扫描本地项目 → 建立项目索引 → 添加 GitHub / 标签 / 描述 → 搜索和筛选 → 一键打开项目**

MVP 暂不负责代码编辑、远程 Git 操作和完整的 DevOps 管理。

------

# 2. MVP 目标

MVP 完成后，用户能够：

```
选择一个或多个本地代码目录
        ↓
扫描并识别项目
        ↓
建立项目列表
        ↓
编辑项目元数据
        ↓
添加 GitHub 地址
        ↓
添加标签 / 状态 / 简介
        ↓
搜索、筛选、收藏
        ↓
快速打开项目
```

核心使用场景：

```
“我的项目到底放在哪里？”
“这个项目是干什么的？”
“这个项目对应哪个 GitHub？”
“我有哪些 AI / Java / Android 项目？”
“打开这个项目最快的方式是什么？”
```

------

# 3. 技术栈

MVP 推荐：

```
Desktop
├── Tauri 2
├── Vue 3
├── TypeScript
├── Vite
└── Pinia

Backend / Native
├── Rust
├── Tauri Commands
├── SQLite
└── Git CLI

UI
└── 自定义 CSS / Tailwind（可选）
```

建议：

**不要在 MVP 引入 Spring Boot。**

这是一个本地桌面应用，Tauri + Rust + SQLite 更符合产品形态。

------

# 4. MVP 功能范围

## P0 — 必须完成

### 4.1 项目管理

支持：

```
添加项目
删除项目
编辑项目
查看项目
打开项目目录
复制项目路径
```

项目基础信息：

```
项目名称
项目路径
项目简介
项目状态
收藏状态
创建时间
更新时间
最后打开时间
```

项目状态：

```
开发中
维护中
暂停
已完成
归档
```

------

## P0 — 本地项目扫描

提供：

```
添加项目目录
扫描目录
识别项目
导入项目
```

例如：

```
D:\Projects
```

扫描：

```
D:\Projects
├── AiCooking
├── DeepSeekHarness
├── MoodDay
├── ProjectManager
└── EnvironmentalSystem
```

自动识别基本项目特征：

```
pom.xml
package.json
build.gradle
build.gradle.kts
settings.gradle
Cargo.toml
go.mod
requirements.txt
pyproject.toml
```

初步判断：

```
Java
Kotlin
JavaScript
TypeScript
Python
Rust
Go
Android
```

### 扫描规则

MVP 不做无限递归。

默认：

```
扫描目录
    ↓
一级子目录
    ↓
检测项目特征文件
    ↓
识别为项目
```

用户可以手动添加单个项目目录。

------

# 5. GitHub / Git 信息

如果项目存在：

```
.git/
```

自动解析：

```
.git/config
```

读取：

```
remote.origin.url
```

例如：

```
https://github.com/example/AiCooking.git
```

转换成：

```
https://github.com/example/AiCooking
```

项目详情显示：

```
GitHub
github.com/example/AiCooking    ↗
```

同时读取：

```
当前 Branch
Git Status
最后 Commit
最后 Commit 时间
```

MVP 只读 Git 信息。

暂不提供：

```
commit
push
pull
merge
branch management
```

Git 操作仍然交给 Git 客户端。

------

# 6. 标签系统

支持：

```
创建标签
删除标签
修改标签
项目添加标签
项目移除标签
按标签筛选
```

例如：

```
AI
RAG
Java
Spring Boot
Vue
Android
Kotlin
毕业设计
开源
实验
简历
```

标签数据：

```
Tag
├── id
├── name
└── color
```

一个项目可以拥有多个 Tag。

------

# 7. 项目简介

项目支持 Markdown 简介。

例如：

```
# AiCooking

基于 Spring Boot + LangChain4j + RAG
构建的智能菜谱系统。

## 功能

- 菜谱检索
- 食材推荐
- RAG
- AI 菜谱生成
```

MVP 可以：

```
纯 Markdown 编辑
```

暂时不需要：

```
实时复杂 Markdown 编辑器
多人协作
版本控制
```

------

# 8. 搜索

搜索是 MVP 的核心能力之一。

顶部提供：

```
Search projects...    Ctrl/Cmd + K
```

搜索范围：

```
项目名称
项目简介
项目路径
GitHub URL
标签
技术栈
```

例如：

```
搜索：
RAG
```

返回：

```
AiCooking
DeepSeekHarness
KnowledgeBase
```

------

# 9. 筛选

支持：

```
全部
收藏
最近打开
开发中
维护中
已完成
```

标签筛选：

```
AI
Java
Android
Vue
RAG
```

组合筛选属于 MVP+，可以先实现简单 AND 条件：

```
标签：AI
状态：开发中
```

------

# 10. 项目快速操作

项目卡片支持：

```
打开文件夹
打开终端
打开 GitHub
复制路径
收藏
编辑
删除
```

MVP 中 IDE 集成暂时只做：

```
Open in default application
```

以及终端：

```
Open Terminal Here
```

IntelliJ IDEA / VS Code 专属检测可以放到下一版本。

------

# 11. UI 信息架构

采用三栏结构。

```
┌──────────────────────────────────────────────────────┐
│ FatFish                         Search   Settings │
├──────────────┬───────────────────────────────────────┤
│              │                                       │
│ Dashboard    │  All Projects                         │
│ Projects     │                                       │
│ Recent       │  ┌───────────────┐ ┌───────────────┐ │
│ Favorites    │  │ AiCooking     │ │ DeepSeek...   │ │
│              │  │ AI · RAG      │ │ Agent         │ │
│ Tags         │  │ ● 开发中      │ │ ● 开发中      │ │
│  AI           │ └───────────────┘ └───────────────┘ │
│  Java         │                                       │
│  Android      │  ┌───────────────┐                  │
│  Vue          │  │ MoodDay       │                  │
│              │  └───────────────┘                  │
│ Status       │                                       │
│  开发中       │                                       │
│  暂停         │                                       │
│  已完成       │                                       │
└──────────────┴───────────────────────────────────────┘
```

页面：

```
Dashboard
Projects
Project Detail
Tags
Settings
```

MVP 可以不实现独立 Dashboard，首页直接作为 Projects 页面。

------

# 12. Project Detail

详情页面：

```
项目名称
项目描述

状态
收藏

Tags

本地路径
GitHub

技术栈

Git 信息

最近修改

快捷操作
```

建议布局：

```
┌─────────────────────────────────────────────┐
│ ← Projects                         ⋯        │
│                                             │
│ AiCooking                                   │
│ 智能菜谱系统                                 │
│                                             │
│ ● 开发中     ★                              │
│                                             │
│ [AI] [RAG] [Spring Boot] [Vue]             │
│                                             │
│ Local                                      │
│ D:\Projects\AiCooking       [打开]         │
│                                             │
│ GitHub                                      │
│ github.com/example/AiCooking   [打开]      │
│                                             │
│ Description                                 │
│ 基于 Spring Boot + RAG ...                  │
│                                             │
│ Git                                         │
│ main                                        │
│ Clean                                       │
│                                             │
└─────────────────────────────────────────────┘
```

------

# 13. SQLite 数据库

MVP 数据库：

## projects

```
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'DEVELOPING',
    github_url TEXT,
    favorite INTEGER NOT NULL DEFAULT 0,
    language TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_opened_at TEXT
);
```

## tags

```
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);
```

## project_tags

```
CREATE TABLE project_tags (
    project_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (project_id, tag_id),
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

## git_info

```
CREATE TABLE git_info (
    project_id TEXT PRIMARY KEY,
    remote_url TEXT,
    branch TEXT,
    commit_hash TEXT,
    commit_message TEXT,
    commit_time TEXT,
    is_dirty INTEGER DEFAULT 0,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);
```

------

# 14. Rust 后端模块

目录：

```
src-tauri/
├── src/
│   ├── main.rs
│   ├── commands/
│   │   ├── project.rs
│   │   ├── scanner.rs
│   │   ├── git.rs
│   │   └── tag.rs
│   │
│   ├── services/
│   │   ├── project_service.rs
│   │   ├── scanner_service.rs
│   │   ├── git_service.rs
│   │   └── search_service.rs
│   │
│   ├── repository/
│   │   ├── project_repository.rs
│   │   ├── tag_repository.rs
│   │   └── git_repository.rs
│   │
│   ├── models/
│   │   ├── project.rs
│   │   ├── tag.rs
│   │   └── git.rs
│   │
│   └── db/
│       └── sqlite.rs
```

核心原则：

```
Command
   ↓
Service
   ↓
Repository
   ↓
SQLite
```

------

# 15. Vue 前端结构

```
src/
├── components/
│   ├── ProjectCard.vue
│   ├── ProjectList.vue
│   ├── ProjectForm.vue
│   ├── TagBadge.vue
│   ├── SearchBar.vue
│   └── EmptyState.vue
│
├── views/
│   ├── ProjectsView.vue
│   ├── ProjectDetailView.vue
│   ├── TagsView.vue
│   └── SettingsView.vue
│
├── stores/
│   ├── project.ts
│   ├── tag.ts
│   └── settings.ts
│
├── services/
│   └── tauri.ts
│
├── router/
│   └── index.ts
│
└── App.vue
```

------

# 16. MVP 开发顺序

严格按照依赖关系开发。

### Phase 0 — 基础工程

```
创建 Tauri 项目
Vue 3
TypeScript
Pinia
SQLite
基础 UI
```

完成标准：

```
应用能够启动
SQLite 能连接
Vue ↔ Rust IPC 正常
```

------

### Phase 1 — Project CRUD

实现：

```
创建项目
读取项目
修改项目
删除项目
```

完成：

```
ProjectCard
ProjectDetail
ProjectForm
```

------

### Phase 2 — 文件夹扫描

实现：

```
选择目录
扫描一级目录
识别项目
批量导入
```

项目判断：

```
pom.xml
package.json
Cargo.toml
go.mod
build.gradle
...
```

------

### Phase 3 — Git

实现：

```
读取 .git/config
解析 remote
读取 branch
读取 commit
检查 dirty 状态
```

Git 不可用时：

```
项目仍然正常
Git 信息显示：
Not a Git repository
```

不能因为 Git 失败导致项目加载失败。

------

### Phase 4 — Tag

实现：

```
Tag CRUD
Project ↔ Tag
Tag Filter
```

------

### Phase 5 — Search

实现：

```
关键词搜索
标签筛选
状态筛选
收藏筛选
最近打开
```

------

### Phase 6 — UX

实现：

```
⌘K / Ctrl+K
快捷键
右键菜单
打开终端
复制路径
打开 GitHub
拖拽添加项目
```

------

# 17. MVP 验收标准

最终必须满足：

### 项目发现

```
可以选择目录
可以扫描项目
可以识别常见项目
可以导入
```

### 项目管理

```
可以增加
可以删除
可以修改
可以查看
```

### 项目元数据

```
Path
Description
GitHub
Status
Favorite
Tags
Language
```

### Git

```
Remote
Branch
Commit
Dirty Status
```

### 搜索

```
名称搜索
标签筛选
状态筛选
收藏筛选
```

### 操作

```
打开项目
打开终端
打开 GitHub
复制路径
```

### 数据可靠性

```
应用重启后数据仍存在
删除文件夹不会导致程序崩溃
移动项目后可以重新定位
无 Git 项目正常显示
无 GitHub 项目正常显示
```

------

# 18. 明确排除项

MVP **不要做**：

```
AI 助手
代码生成
Git Commit / Push
GitHub API 深度集成
云同步
账号体系
多人协作
Docker 管理
进程监控
日志中心
任务管理
Issue 管理
CI/CD
插件系统
MCP
远程服务器
```

这些都属于后续版本。

------

# 19. MVP 完成后的产品形态

最终 MVP 应该形成这样一个闭环：

```
                FatFish
                     │
        ┌────────────┼────────────┐
        │            │            │
      本地项目      项目元数据      Git
        │            │            │
      Folder       Tag/Desc      GitHub
        │            │            │
        └────────────┼────────────┘
                     │
                  Search
                     │
                  Launch
```

也就是：

> **“我所有项目在哪里、是什么、属于什么类型、对应哪个仓库，以及如何最快打开它。”**

这就是 MVP 最核心的价值。

------

# 20. 后续 Roadmap

MVP 稳定后，再自然演进：

```
v0.1
本地项目管理
    ↓
v0.2
IDE / Terminal 启动
    ↓
v0.3
项目运行配置
    ↓
v0.4
进程 / Port / Docker
    ↓
v0.5
统一日志
    ↓
v0.6
项目工作台
    ↓
v1.0
Local Developer Hub
```

其中 **v0.2 之后**可以逐步和你之前设想的“桌面总控台”融合，但 MVP 阶段一定要把边界控制住。