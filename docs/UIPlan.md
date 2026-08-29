# Project Hub — UI Plan

## 1. UI 产品定位

Project Hub 是一个**通用的本地项目空间管理器**，而不是开发者专用工具。

UI 需要同时适用于：

```
代码项目
论文项目
课程项目
设计项目
摄影项目
研究项目
旅行项目
个人计划
创业项目
资料整理项目
```

因此前端设计遵循：

> **内容优先、项目中立、轻量现代、低视觉噪音、桌面原生感。**

视觉关键词：

```
Modern
Minimal
Neutral
Calm
Personal
Organized
Flexible
```

避免：

```
IDE 风
极客风
代码雨
霓虹色
大面积渐变
后台管理系统风格
```

------

# 2. 总体 UI 架构

采用：

```
┌──────────────────────────────────────────────────────────────┐
│                      Top Bar                                 │
├────────────────┬─────────────────────────────────────────────┤
│                │                                             │
│    Sidebar     │                 Main Content                │
│                │                                             │
│                │                                             │
│                │                                             │
│                │                                             │
│                │                                             │
└────────────────┴─────────────────────────────────────────────┘
```

整体尺寸：

```
Sidebar: 224px
Top Bar: 64px
Page Padding: 24px
Content Max Width: 1400px
Card Radius: 14px
Small Radius: 8px
```

应用窗口需要支持：

```
最小宽度：1000px
最小高度：640px
```

优先适配桌面环境，不做移动端 UI。

------

# 3. Design Token

## 3.1 Light Theme

```
--bg: #F7F7F5;
--surface: #FFFFFF;
--surface-muted: #F1F1EF;

--border: #E7E7E3;
--border-strong: #D9D9D4;

--text-primary: #202124;
--text-secondary: #6B6D70;
--text-tertiary: #96989B;

--hover: #F2F2F0;
--selected: #EBEBE8;
```

## 3.2 Dark Theme

```
--bg: #151515;
--surface: #1C1C1C;
--surface-muted: #232323;

--border: #303030;
--border-strong: #3A3A3A;

--text-primary: #F5F5F5;
--text-secondary: #A5A5A5;
--text-tertiary: #707070;

--hover: #252525;
--selected: #2A2A2A;
```

## 3.3 状态色

状态色保持低饱和：

```
进行中
绿色

计划中
蓝色

暂停
黄色

已完成
灰色

归档
深灰
```

状态色只用于：

```
状态点
少量文本
极小面积图标
```

禁止用状态色大面积填充 Card。

------

# 4. Typography

字体：

```
UI:
Inter / system-ui

中文:
Noto Sans SC / system fallback

等宽:
JetBrains Mono
```

层级：

```
Page Title
28px / 700

Section Title
18px / 600

Project Title
16px / 600

Body
14px / 400

Secondary
13px / 400

Caption
12px / 400
```

避免页面出现大量超大标题。

------

# 5. Sidebar UI

Sidebar：

```
width: 224px
padding: 12px
background: transparent
```

结构：

```
Project Hub

⌕ Search                         ⌘K

HOME
  Home

PROJECTS
  All Projects
  Recent
  Favorites

COLLECTIONS
  学习
  工作
  个人
  创作

TAGS
  AI
  Design
  Travel
  Java

──────────────

⚙ Settings
```

要求：

### Logo

使用简洁几何图标：

```
◇ Project Hub
```

不要使用代码符号作为 Logo。

------

# 6. Sidebar 交互

Normal：

```
透明
```

Hover：

```
background: var(--hover)
```

Active：

```
background: var(--selected)
font-weight: 600
```

不使用：

```
左侧粗色条
高亮发光
渐变背景
```

Sidebar 支持折叠：

```
Expanded
224px

Collapsed
64px
```

折叠后：

```
◇
⌕
⌂
▣
★
...
```

------

# 7. Top Bar

通用 Top Bar：

```
┌─────────────────────────────────────────────────────────┐
│ Projects                                      + Project │
└─────────────────────────────────────────────────────────┘
```

右侧：

```
Search
Sort
View
Add
```

Project Detail：

```
← Projects                           ⋯
```

Top Bar 不承载过多功能。

------

# 8. Home 页面

Home 是用户进入应用后的默认页面。

目标：

> 快速了解最近在做什么，并快速进入项目。

布局：

```
Good afternoon

Your Projects

[ Project Card ] [ Project Card ] [ Project Card ]

Recent

Project A
Project B
Project C

Favorites

Project X
Project Y
```

### Home 顶部

```
Good afternoon

Keep track of everything you're working on.
```

不要根据真实时间强行生成复杂问候逻辑，静态：

```
Your projects
```

也完全可以。

------

# 9. Projects 页面

这是核心页面。

顶部：

```
All Projects                         + New Project

24 projects

[Search] [Filter] [Sort] [Grid/List]
```

默认 Grid：

```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│              │  │              │  │              │
│ AiCooking    │  │ Tokyo 2026   │  │ Thesis       │
│              │  │              │  │              │
│ AI · RAG     │  │ Travel       │  │ Research     │
│              │  │              │  │              │
│ ● 进行中     │  │ ● 整理中     │  │ ● 写作中     │
└──────────────┘  └──────────────┘  └──────────────┘
```

响应式：

```
> 1200px
3 columns

> 1500px
4 columns

< 1100px
2 columns
```

------

# 10. Project Card

Project Card 结构：

```
┌──────────────────────────────────────┐
│ Cover / Icon                         │
│                                      │
│ Project Name                    ⋯   │
│ Description                          │
│                                      │
│ [Tag] [Tag] [Tag]                    │
│                                      │
│ ● Status                    Updated  │
└──────────────────────────────────────┘
```

默认高度：

```
220–260px
```

Card 不显示过多技术细节。

------

# 11. Cover 系统

项目允许：

```
Icon
Emoji
Image
Color
None
```

默认：

```
自动生成简洁封面
```

例如：

```
AiCooking
AI

Tokyo 2026
Travel

毕业论文
Research
```

Cover 区域：

```
height: 100px
```

MVP 不需要复杂图片编辑器。

------

# 12. Project Card Hover

Hover：

```
border → slightly stronger
background → slightly brighter
```

同时右上角显示：

```
⋯
```

底部可以出现：

```
Open
```

不要做：

```
大幅浮起
缩放
阴影扩散
动画卡片
```

动画时间：

```
150–200ms
```

------

# 13. List View

点击 View：

```
Grid
List
```

List：

```
┌─────────────────────────────────────────────────────────┐
│ Project        Tags              Status      Updated     │
├─────────────────────────────────────────────────────────┤
│ AiCooking      AI · RAG          进行中      2h ago      │
│ Tokyo 2026     Travel            整理中      Yesterday   │
│ Thesis         Research          写作中      Aug 28      │
└─────────────────────────────────────────────────────────┘
```

List 适合大量项目。

------

# 14. Filter UI

点击 Filter：

```
┌──────────────────────────┐
│ Filter                   │
│                          │
│ Status                   │
│ ○ All                    │
│ ○ 进行中                 │
│ ○ 暂停                   │
│ ○ 已完成                 │
│                          │
│ Collections             │
│ □ 学习                   │
│ □ 工作                   │
│ □ 个人                   │
│                          │
│ Tags                     │
│ □ AI                     │
│ □ Travel                 │
│ □ Design                 │
└──────────────────────────┘
```

第一版先实现：

```
Status
Tag
Favorite
Collection
```

------

# 15. Search

全局搜索：

```
Ctrl + K
```

弹出 Command Palette：

```
┌────────────────────────────────────────┐
│ ⌕ Search projects...                   │
├────────────────────────────────────────┤
│                                        │
│ RECENT                                 │
│                                        │
│ ◇ AiCooking                            │
│   AI · RAG                             │
│                                        │
│ ◇ Tokyo 2026                           │
│   Travel · Photo                       │
│                                        │
│ ◇ Thesis                               │
│   Research                             │
│                                        │
├────────────────────────────────────────┤
│ ↑↓ Navigate    Enter Open      Esc      │
└────────────────────────────────────────┘
```

支持：

```
Project name
Description
Tags
Path
Links
Collection
```

------

# 16. New Project Dialog

创建项目：

```
┌──────────────────────────────────────┐
│ New Project                      ×   │
│                                      │
│ Name                                 │
│ [ ______________________________ ]   │
│                                      │
│ Location                             │
│ [ D:\Projects\xxx          ] [ ... ]│
│                                      │
│ Description                          │
│ [                                  ] │
│ [                                  ] │
│                                      │
│ Tags                                 │
│ [ AI ] [ RAG ] [ + ]                 │
│                                      │
│ Status                               │
│ [ 进行中 ▾ ]                          │
│                                      │
│                Cancel     Create      │
└──────────────────────────────────────┘
```

核心：

```
Name
Location
Description
Tags
Status
```

------

# 17. Scan Folder UI

选择：

```
Scan Folder
```

显示：

```
┌──────────────────────────────────────┐
│ Scan Folder                       ×  │
│                                      │
│ D:\Projects                          │
│                                      │
│ Discovering projects...              │
│                                      │
│ ✓ AiCooking                          │
│ ✓ MoodDay                            │
│ ✓ Tokyo2026                          │
│ ✓ Thesis                             │
│                                      │
│ 18 projects found                    │
│                                      │
│            Cancel     Import 18      │
└──────────────────────────────────────┘
```

扫描过程可实时更新。

------

# 18. Project Detail 页面

核心布局：

```
┌──────────────────────────────────────────────┐
│ ← Projects                            ⋯     │
│                                              │
│              [ Cover ]                       │
│                                              │
│              AiCooking                       │
│              智能菜谱系统                     │
│                                              │
│              ● 进行中                        │
│                                              │
│              [AI] [RAG] [个人]              │
│                                              │
├──────────────────────────────────────────────┤
│ LOCATION                                     │
│ 📁 D:\Projects\AiCooking           Open      │
│                                              │
│ LINKS                                        │
│ GitHub                         ↗             │
│ Documentation                  ↗             │
│                                              │
│ DESCRIPTION                                  │
│ 基于 Spring Boot + RAG 构建的智能菜谱系统。 │
│                                              │
│ NOTES                                        │
│ 下一步增加推荐系统……                         │
└──────────────────────────────────────────────┘
```

Detail 页面重点：

```
信息
链接
位置
描述
备注
```

而不是：

```
Git
Commit
Branch
Terminal
```

Git 信息属于可选模块。

------

# 19. Project Links

不要设计：

```
GitHub URL
```

作为唯一链接。

统一设计成：

```
Links

GitHub
官网
文档
Figma
在线 Demo
论文
网盘
其他
```

数据模型：

```
title
url
type
```

这样项目类型不会被限制。

------

# 20. Notes

Project 可以拥有 Notes：

```
Notes

下一阶段：
1. 优化 RAG
2. 增加推荐
3. 完善 UI
```

MVP：

```
纯文本 / Markdown
```

------

# 21. Tags UI

Tags 页面：

```
Tags

[ + New Tag ]

AI              8 projects
Research        6 projects
Java             5 projects
Travel           3 projects
Design           4 projects
```

点击 Tag：

```
AI

8 projects

[Project]
[Project]
[Project]
```

标签视觉保持中性。

------

# 22. Collections UI

Collection 与 Tag 分离。

Collection：

```
学习
工作
个人
创作
研究
```

一个项目可以属于多个 Collection。

例如：

```
AiCooking
Collections:
学习
个人
```

Collection 主要用于：

> **人为组织项目。**

Tag 用于：

> **描述项目属性。**

------

# 23. Settings UI

设置页不做复杂后台。

```
Settings

Appearance
────────────────────────
Theme
○ System
○ Light
○ Dark

Project Library
────────────────────────
Default Project Folder
D:\Projects

Auto Scan
[ ON ]

Behavior
────────────────────────
Open project on double click
[ ON ]

Confirm project removal
[ ON ]

About
────────────────────────
Project Hub
Version 0.1.0
```

------

# 24. Context Menu

Project 右键：

```
Open
Open Folder
Open Terminal
Copy Path

Edit
Add Tag
Move to Collection

Favorite

──────────────

Remove Project
```

不使用复杂多级菜单。

------

# 25. Empty State

### 没有项目

```
◇

No projects yet

Add a local project or scan a folder
to build your project space.

[ + Add Project ]   [ Scan Folder ]
```

### 搜索无结果

```
No projects found

Try another keyword or remove filters.
```

### 收藏为空

```
No favorite projects

Star projects you access frequently.
```

------

# 26. Loading

统一使用 Skeleton。

Project Card：

```
████████████
████████
████████████████
██████
```

避免大量：

```
Loading...
```

项目扫描使用进度反馈。

------

# 27. Toast

操作成功：

```
Project added
```

```
Project updated
```

```
Link copied
```

失败：

```
Failed to open project folder
```

Toast：

```
右下角
持续 2–3 秒
```

------

# 28. Modal 规范

Dialog：

```
width: 480–560px
radius: 16px
padding: 24px
```

Modal 背景：

```
rgba(0,0,0,0.35)
```

动画：

```
opacity
translateY(4px)
150–200ms
```

------

# 29. Icon 规范

建议：

```
Lucide Icons
```

统一：

```
stroke: 1.8px
size: 16 / 18 / 20px
```

禁止混用：

```
Emoji
Material Icons
Font Awesome
Lucide
```

除非 Emoji 是用户主动设置的项目图标。

------

# 30. 响应式规则

桌面优先：

```
≥ 1500px
4 Card

1200–1499px
3 Card

900–1199px
2 Card

< 900px
Sidebar Collapse
```

不是移动端设计，而是：

> **桌面窗口缩放适配。**

------

# 31. 前端组件架构

```
src/
├── components/
│   ├── app/
│   │   ├── AppShell.vue
│   │   ├── Sidebar.vue
│   │   ├── TopBar.vue
│   │   └── CommandPalette.vue
│   │
│   ├── project/
│   │   ├── ProjectCard.vue
│   │   ├── ProjectGrid.vue
│   │   ├── ProjectList.vue
│   │   ├── ProjectCover.vue
│   │   ├── ProjectStatus.vue
│   │   ├── ProjectLinks.vue
│   │   ├── ProjectMetadata.vue
│   │   └── ProjectMenu.vue
│   │
│   ├── tag/
│   │   ├── TagBadge.vue
│   │   └── TagSelector.vue
│   │
│   ├── collection/
│   │   └── CollectionSelector.vue
│   │
│   ├── common/
│   │   ├── EmptyState.vue
│   │   ├── Skeleton.vue
│   │   ├── Toast.vue
│   │   └── ConfirmDialog.vue
│   │
│   └── dialog/
│       ├── ProjectDialog.vue
│       ├── ScanDialog.vue
│       └── LinkDialog.vue
│
├── views/
│   ├── HomeView.vue
│   ├── ProjectsView.vue
│   ├── ProjectDetailView.vue
│   ├── TagsView.vue
│   ├── CollectionView.vue
│   └── SettingsView.vue
│
├── stores/
│   ├── project.ts
│   ├── tag.ts
│   ├── collection.ts
│   ├── search.ts
│   └── settings.ts
│
├── composables/
│   ├── useProject.ts
│   ├── useSearch.ts
│   ├── useTheme.ts
│   └── useShortcut.ts
│
├── styles/
│   ├── tokens.css
│   ├── reset.css
│   ├── global.css
│   └── components.css
│
└── router/
    └── index.ts
```

------

# 32. 路由设计

```
/
├── /home
├── /projects
├── /projects/:id
├── /recent
├── /favorites
├── /collections/:id
├── /tags/:id
└── /settings
```

默认：

```
/ → /home
```

------

# 33. 页面开发顺序

严格按照：

```
01 App Shell
        ↓
02 Sidebar
        ↓
03 TopBar
        ↓
04 Projects Grid
        ↓
05 Project Card
        ↓
06 Project Detail
        ↓
07 New Project Dialog
        ↓
08 Search
        ↓
09 Tags
        ↓
10 Collections
        ↓
11 Settings
        ↓
12 Dark Mode
        ↓
13 Animation / Polish
```

不要一开始同时开发所有页面。

------

# 34. MVP UI 验收标准

## 视觉

必须满足：

```
无大面积渐变
无明显 IDE 风
无后台管理系统风格
Light / Dark 都可用
间距统一
圆角统一
字体统一
图标统一
```

## 交互

必须支持：

```
Ctrl + K 搜索
项目新建
项目编辑
项目删除
项目打开
项目收藏
标签
Collection
筛选
排序
Grid/List
Dark/Light
```

## 项目卡片

至少显示：

```
Cover
Name
Description
Tags
Status
Updated
```

## 项目详情

至少显示：

```
Name
Cover
Description
Location
Links
Tags
Status
Notes
```

------

# 35. 最终视觉方向

最终应该让用户产生这样的感觉：

```
              Project Hub

        ┌───────────────────┐
        │                   │
        │   Tokyo 2026      │
        │                   │
        └───────────────────┘

          My Project Space

   ┌────────────┐  ┌────────────┐
   │ AiCooking  │  │ Thesis     │
   │ AI · RAG   │  │ Research   │
   └────────────┘  └────────────┘
```

它不是：

> “管理我的代码仓库。”

而是：

> **“管理我电脑里所有正在做的事情。”**

这也决定了后续的数据模型必须从第一天就保持**Project 中立**：GitHub、Git、技术栈都只能作为项目的可选能力，而不能成为 Project 的核心定义。