import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { tauriApi } from "../services/tauri";
import type { Collection, Project, ProjectFilter, ProjectInput, ProjectStatus, ScannedProject } from "../types";
import { useSettingsStore } from "./settings";

export const useProjectStore = defineStore("project", () => {
  const settings = useSettingsStore();

  const projects = ref<Project[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const query = ref("");
  const status = ref<ProjectStatus | null>(null);
  const favorite = ref(false);
  const tagId = ref<string | null>(null);
  const collectionId = ref<string | null>(null);
  const recent = ref(false);

  function currentFilter(): ProjectFilter {
    const f: ProjectFilter = {};
    if (query.value.trim()) f.query = query.value.trim();
    if (status.value) f.status = status.value;
    if (favorite.value) f.favorite = true;
    if (tagId.value) f.tagId = tagId.value;
    if (collectionId.value) f.collectionId = collectionId.value;
    if (recent.value) f.recent = true;
    f.sort = settings.sort;
    return f;
  }

  async function fetchProjects() {
    loading.value = true;
    error.value = null;
    try {
      projects.value = await tauriApi.listProjects(currentFilter());
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  function resetFilter() {
    status.value = null;
    favorite.value = false;
    tagId.value = null;
    collectionId.value = null;
    recent.value = false;
  }

  async function createProject(input: ProjectInput) {
    const project = await tauriApi.createProject(input);
    await fetchProjects();
    return project;
  }

  async function updateProject(id: string, input: ProjectInput): Promise<Project> {
    const updated = await tauriApi.updateProject(id, input);
    await fetchProjects();
    return updated;
  }

  /** 以 project 为基准，仅打补丁后提交（其余字段保持原值） */
  function toInput(project: Project, patch: Partial<Pick<ProjectInput, "status" | "notes">>): ProjectInput {
    return {
      name: project.name,
      path: project.path,
      description: project.description,
      status: project.status,
      coverEmoji: project.coverEmoji,
      coverColor: project.coverColor,
      notes: project.notes,
      ...patch,
    };
  }

  /** 快捷改状态（卡片菜单用）：仅改 status，其余字段保持原值 */
  async function changeStatus(project: Project, nextStatus: ProjectStatus): Promise<Project> {
    return updateProject(project.id, toInput(project, { status: nextStatus }));
  }

  /** 自动保存笔记：仅改 notes */
  async function updateProjectNotes(project: Project, notes: string | null): Promise<Project> {
    return updateProject(project.id, toInput(project, { notes }));
  }

  async function deleteProject(id: string) {
    await tauriApi.deleteProject(id);
    await fetchProjects();
  }

  /** 收藏切换：返回刷新后的 Project，并同步本地列表缓存 */
  async function toggleFavorite(project: Project): Promise<Project> {
    await tauriApi.setFavorite(project.id, !project.favorite);
    const updated = await tauriApi.getProject(project.id);
    upsertProject(updated);
    return updated;
  }

  async function markOpened(id: string) {
    await tauriApi.markOpened(id);
  }

  async function openInFolder(project: Project) {
    await tauriApi.openFolder(project.path);
    await markOpened(project.id);
  }

  async function openTerminal(project: Project) {
    await tauriApi.openTerminal(project.path);
    await markOpened(project.id);
  }

  /** 刷新单个项目的 Git 信息，返回更新后的 Project */
  async function refreshGit(projectId: string): Promise<Project> {
    const updated = await tauriApi.refreshGitInfo(projectId);
    upsertProject(updated);
    return updated;
  }

  async function getProject(id: string): Promise<Project> {
    return tauriApi.getProject(id);
  }

  async function setProjectTags(projectId: string, tagIds: string[]): Promise<Project> {
    await tauriApi.setProjectTags(projectId, tagIds);
    const updated = await tauriApi.getProject(projectId);
    upsertProject(updated);
    return updated;
  }

  async function setProjectCollections(projectId: string, collectionIds: string[]): Promise<Project> {
    await tauriApi.setProjectCollections(projectId, collectionIds);
    const updated = await tauriApi.getProject(projectId);
    upsertProject(updated);
    return updated;
  }

  async function scanDirectory(path: string): Promise<ScannedProject[]> {
    return tauriApi.scanDirectory(path);
  }

  async function importProjects(paths: string[], collectionId?: string | null): Promise<number> {
    const imported = await tauriApi.importProjects(paths, collectionId);
    await fetchProjects();
    return imported.length;
  }

  /** 全量项目列表（首页 / 最近 / 收藏等聚合视图用） */
  async function fetchAll(): Promise<Project[]> {
    return tauriApi.listProjects({ sort: "updated" });
  }

  /** 集合视图：拉取全量后按集合过滤（避免为每个集合单独发起一次后端调用） */
  async function fetchByCollection(id: string): Promise<Project[]> {
    const all = await fetchAll();
    return all.filter((p) => p.collections.some((c) => c.id === id));
  }

  function upsertProject(updated: Project) {
    const idx = projects.value.findIndex((p) => p.id === updated.id);
    if (idx !== -1) projects.value[idx] = updated;
  }

  // ---- 侧栏计数：派生自本地缓存，无需额外后端调用 ----
  const totalCount = computed(() => projects.value.length);
  const recentCount = computed(
    () => projects.value.filter((p) => p.lastOpenedAt).length
  );
  const favoriteCount = computed(
    () => projects.value.filter((p) => p.favorite).length
  );

  return {
    projects,
    loading,
    error,
    query,
    status,
    favorite,
    tagId,
    collectionId,
    recent,
    fetchProjects,
    resetFilter,
    createProject,
    updateProject,
    changeStatus,
    updateProjectNotes,
    deleteProject,
    toggleFavorite,
    markOpened,
    openInFolder,
    openTerminal,
    refreshGit,
    getProject,
    setProjectTags,
    setProjectCollections,
    scanDirectory,
    importProjects,
    fetchAll,
    fetchByCollection,
    totalCount,
    recentCount,
    favoriteCount,
  };
});

export type { Collection };
