export interface Todo {
  id: string;
  title: string;
  done: boolean;
  projectId: string | null;
  projectName: string | null;
  projectStatus: string | null;
  /** 截止日期 YYYY-MM-DD */
  dueDate: string | null;
}

export interface TodoInput {
  title: string;
  projectId: string | null;
  dueDate: string | null;
}

export interface Tag {
  id: string;
  name: string;
  color: string | null;
}

export interface Collection {
  id: string;
  name: string;
}

export interface Link {
  id: string;
  projectId: string;
  title: string;
  url: string;
  linkType: string | null;
}

export interface LinkInput {
  title: string;
  url: string;
  linkType: string | null;
}

export interface GitInfo {
  remoteUrl: string | null;
  branch: string | null;
  commitHash: string | null;
  commitMessage: string | null;
  commitTime: string | null;
  isDirty: boolean | null;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  description: string | null;
  status: string;
  favorite: boolean;
  tags: Tag[];
  collections: Collection[];
  links: Link[];
  coverEmoji: string | null;
  coverColor: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
  lastOpenedAt: string;
  language: string | null;
  gitInfo: GitInfo | null;
}

export interface ProjectInput {
  name: string;
  path: string;
  description: string | null;
  status: string;
  coverEmoji: string | null;
  coverColor: string | null;
  notes: string | null;
}

export interface ProjectFilter {
  query?: string;
  status?: string;
  favorite?: boolean;
  tagId?: string;
  collectionId?: string;
  recent?: boolean;
  sort?: string;
}

export interface ScannedProject {
  name: string;
  path: string;
  language: string | null;
  alreadyImported: boolean;
}

export interface TagInput {
  name: string;
  color: string | null;
}

export interface CollectionInput {
  name: string;
}

/** 状态枚举 — UIPlan §3.3：进行中 / 计划中 / 暂停 / 已完成 / 归档 */
export const STATUS_OPTIONS: { value: string; label: string }[] = [
  { value: "IN_PROGRESS", label: "进行中" },
  { value: "PLANNED", label: "计划中" },
  { value: "PAUSED", label: "暂停" },
  { value: "COMPLETED", label: "已完成" },
  { value: "ARCHIVED", label: "归档" },
];

export function statusLabel(status: string): string {
  return STATUS_OPTIONS.find((s) => s.value === status)?.label ?? status;
}

export const LINK_TYPE_OPTIONS: { value: string; label: string }[] = [
  { value: "github", label: "GitHub" },
  { value: "website", label: "官网" },
  { value: "docs", label: "文档" },
  { value: "design", label: "设计" },
  { value: "demo", label: "在线 Demo" },
  { value: "paper", label: "论文" },
  { value: "cloud", label: "网盘" },
  { value: "other", label: "其他" },
];

export function linkTypeLabel(type: string | null): string {
  if (!type) return "链接";
  return LINK_TYPE_OPTIONS.find((t) => t.value === type)?.label ?? type;
}

/** 相对时间：卡片 "Updated" 列展示 */
export function relativeTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;
  const diff = Date.now() - d.getTime();
  const minute = 60_000;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < minute) return "刚刚";
  if (diff < hour) return `${Math.floor(diff / minute)} 分钟前`;
  if (diff < day) return `${Math.floor(diff / hour)} 小时前`;
  if (diff < 2 * day) return "昨天";
  if (diff < 30 * day) return `${Math.floor(diff / day)} 天前`;
  return d.toLocaleDateString();
}

/** 按名称生成稳定的封面颜色（低饱和） */
const COVER_PALETTE = ["#7A9E7E", "#7E93B8", "#B8A47E", "#A98BB8", "#B8877E", "#7EB0AC", "#96890F", "#8B8DB0"];

export function coverColorFor(name: string, explicit: string | null): string {
  if (explicit) return explicit;
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0;
  return COVER_PALETTE[hash % COVER_PALETTE.length];
}
