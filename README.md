# FatFish

[![Deploy promo page to GitHub Pages](https://github.com/ctctoo/FatFish/actions/workflows/pages.yml/badge.svg)](https://github.com/ctctoo/FatFish/actions/workflows/pages.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://github.com/ctctoo/FatFish#license)

**Local-First 的桌面项目管理工具** —— 把散落在磁盘各处的项目收进一个清爽的首页：扫描、索引、组织、记录，随时一键回到工作现场。

> 数据全部存在本机，不依赖任何云端服务。你的项目文件夹永远不会被改动——FatFish 只做记录者。

🌐 **[产品介绍页](https://ctctoo.github.io/FatFish/)**

## 功能

### 项目库

- **本地扫描导入**：自动识别一级子目录中的项目（`pom.xml` / `package.json` / `Cargo.toml` / `go.mod` / `build.gradle` / `requirements.txt`…），并判断 Java / Kotlin / JS / TS / Python / Rust / Go / Android 等技术栈
- **项目状态**：`● 计划中 / ● 进行中 / ● 已暂停 / ● 已完成 / ● 已归档`，卡片右键或 ⋯ 菜单即可一键切换，状态一目了然
- **封面系统**：Emoji + 主题色封面，项目卡片不再千篇一律
- **收藏与最近打开**：常用项目置顶显示

### 组织方式

- **集合（Collections）**：按用途自由分组，支持从扫描结果批量导入、多选现有项目加入集合
- **标签（Tags）**：彩色标签系统，按标签筛选
- **多维度筛选**：状态 / 集合 / 标签 / 收藏 / 最近，配合排序（更新时间 / 名称 / 打开时间）快速定位

### 记录与追踪

- **Todo 待办**：首页 Todo 面板，任务可关联项目、设置截止日期、勾选完成
- **项目链接**：GitHub / 官网 / 文档 / 设计稿 / 在线 Demo / 论文 / 网盘——一个项目的所有入口
- **备注**：自由文本记录项目上下文
- **活动时间线（Timeline）**：自动记录项目创建、描述更新、链接添加、Todo 创建、状态变更，回看项目脉络
- **Git 只读信息**：remote、分支、最后 commit、dirty 状态（Git 不可用时自动降级，不影响使用）

### 桌面体验

- **命令面板**：`Ctrl+K` 全局搜索项目，键盘党友好
- **双主题**：浅色 / 深色 / 跟随系统，基于 Design Token 的完整主题体系
- **中英双语**：设置内一键切换，界面即时生效
- **网格 / 列表**两种视图，可折叠侧栏，原生桌面窗口

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2（Rust + 系统 WebView） |
| 前端 | Vue 3 + TypeScript + Vite + Pinia + Vue Router |
| 后端 | Rust，Command → Service → Repository → SQLite 分层 |
| 存储 | SQLite（rusqlite bundled，幂等迁移） |
| 国际化 | 自研轻量 i18n（字典 + `useI18n()`，locale 持久化） |

## 快速开始

环境要求：Node 18+、Rust stable（MSVC）、WebView2（Windows 10+ 自带）。

```bash
npm install          # 安装依赖
npm run tauri dev    # 开发模式
npm run tauri build  # 构建发行版（exe + 安装包，位于 src-tauri/target/release/bundle/）
```

## 数据与隐私

- SQLite 数据库位于系统应用数据目录：`%APPDATA%/com.fatfish.app/fatfish.db`
- 从索引中移除项目**不会**删除磁盘上的项目文件夹
- 不联网、不上报、不同步，卸载即走

## 文档

- [产品介绍页（在线）](https://ctctoo.github.io/FatFish/) —— 由 `.github/workflows/pages.yml` 从 `promo.html` 自动部署
- [promo.html](promo.html) —— 产品介绍页源文件
- [MvpPlan.md](docs/MvpPlan.md) —— MVP 规划
- [UIPlan.md](docs/UIPlan.md) —— UI 设计方案

## License

MIT
