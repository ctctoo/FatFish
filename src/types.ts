export interface Activity {
  id: string;
  projectId: string;
  /** created | updated | link | todo */
  kind: string;
  message: string;
  createdAt: string;
}

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
  status: ProjectStatus;
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
  status: ProjectStatus;
  coverEmoji: string | null;
  coverColor: string | null;
  notes: string | null;
}

export interface ProjectFilter {
  query?: string;
  status?: ProjectStatus;
  favorite?: boolean;
  tagId?: string;
  collectionId?: string;
  recent?: boolean;
  sort?: SortKey;
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

/** 状态枚举值 — UIPlan §3.3：进行中 / 计划中 / 暂停 / 已完成 / 归档 */
export const STATUS_VALUES = ["IN_PROGRESS", "PLANNED", "PAUSED", "COMPLETED", "ARCHIVED"] as const;

export type ProjectStatus = (typeof STATUS_VALUES)[number];

/** 项目列表排序键（与 settings.sort 一致） */
export type SortKey = "updated" | "name" | "opened";

export const LINK_TYPE_VALUES = [
  "github",
  "website",
  "docs",
  "design",
  "demo",
  "paper",
  "cloud",
  "other",
] as const;

/** 按名称生成稳定的封面颜色（低饱和） */
const COVER_PALETTE = ["#7A9E7E", "#7E93B8", "#B8A47E", "#A98BB8", "#B8877E", "#7EB0AC", "#96890F", "#8B8DB0"];

export function coverColorFor(name: string, explicit: string | null): string {
  if (explicit) return explicit;
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0;
  return COVER_PALETTE[hash % COVER_PALETTE.length];
}

/** 标签彩色圆片配色（背景 / 文字成对），按名称稳定取色 */
export const TAG_CHIP_PALETTE: { bg: string; text: string }[] = [
  { bg: "#E8EEFB", text: "#3B6FD4" },
  { bg: "#E5F3EA", text: "#3D8B5F" },
  { bg: "#FBF0DC", text: "#B07A1A" },
  { bg: "#F6E9F7", text: "#9A4FA8" },
  { bg: "#FBE9E4", text: "#C05B3C" },
  { bg: "#E4F3F1", text: "#2F8A7E" },
  { bg: "#EDEcFa", text: "#6B5BD2" },
  { bg: "#F3EEE2", text: "#8A7A2F" },
];

export function tagChipFor(name: string, explicit: string | null): { bg: string; text: string } {
  if (explicit) {
    return { bg: `color-mix(in srgb, ${explicit} 14%, transparent)`, text: explicit };
  }
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0;
  return TAG_CHIP_PALETTE[hash % TAG_CHIP_PALETTE.length];
}
