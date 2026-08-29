# Project Hub

Local-First 的本地项目管理工具。统一管理本机开发项目文件夹，通过项目元数据建立索引：

> 扫描本地项目 → 建立项目索引 → 添加 GitHub / 标签 / 描述 → 搜索和筛选 → 一键打开项目

## 技术栈

- **桌面**: Tauri 2
- **前端**: Vue 3 + TypeScript + Vite + Pinia + Vue Router
- **后端**: Rust（Command → Service → Repository → SQLite 分层）
- **存储**: SQLite（rusqlite，bundled）
- **Git**: 只读解析 `.git/config` + Git CLI

## 功能

- 项目 CRUD：添加 / 编辑 / 删除 / 查看 / 收藏
- 本地扫描：识别一级子目录中的项目（pom.xml / package.json / Cargo.toml / go.mod / build.gradle / requirements.txt…），自动判断 Java / Kotlin / JS / TS / Python / Rust / Go / Android
- Git 只读信息：remote origin、branch、最后 commit、dirty 状态（Git 不可用时自动降级，不影响项目加载）
- 标签系统：创建 / 编辑 / 删除 / 按标签筛选
- 搜索：名称、简介、路径、GitHub、标签、技术栈，`Ctrl+K` 快捷聚焦
- 筛选：全部 / 收藏 / 最近打开 / 状态
- 快捷操作：打开文件夹、打开终端、打开 GitHub、复制路径

## 开发

```bash
npm install
npm run tauri dev     # 开发模式启动
npm run tauri build   # 打包
```

要求：Node 18+、Rust stable（MSVC）、WebView2（Windows 10+ 自带）。

## 数据

SQLite 数据库位于系统应用数据目录（`%APPDATA%/com.projecthub.app/project-hub.db`）。删除索引中的项目不会影响磁盘上的项目文件夹。
