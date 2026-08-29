import { defineStore } from "pinia";
import { ref } from "vue";
import { tauriApi } from "../services/tauri";
import type { Collection, Project, ProjectFilter, ProjectInput, ScannedProject } from "../types";
import { useSettingsStore } from "./settings";

export const useProjectStore = defineStore("project", () => {
  const settings = useSettingsStore();

  const projects = ref<Project[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const query = ref("");
  const status = ref<string | null>(null);
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

  async function deleteProject(id: string) {
    await tauriApi.deleteProject(id);
    await fetchProjects();
  }

  async function toggleFavorite(project: Project) {
    await tauriApi.setFavorite(project.id, !project.favorite);
    await fetchProjects();
  }

  async function markOpened(id: string) {
    await tauriApi.markOpened(id);
  }

  async function openInFolder(project: Project) {
    await tauriApi.openFolder(project.path);
    await markOpened(project.id);
  }

  async function setProjectTags(projectId: string, tagIds: string[]) {
    await tauriApi.setProjectTags(projectId, tagIds);
  }

  async function setProjectCollections(projectId: string, collectionIds: string[]) {
    await tauriApi.setProjectCollections(projectId, collectionIds);
  }

  async function scanDirectory(path: string): Promise<ScannedProject[]> {
    return tauriApi.scanDirectory(path);
  }

  async function importProjects(paths: string[]): Promise<number> {
    const imported = await tauriApi.importProjects(paths);
    await fetchProjects();
    return imported.length;
  }

  /** 全量项目列表（首页 / 最近 / 收藏等聚合视图用） */
  async function fetchAll(): Promise<Project[]> {
    return tauriApi.listProjects({ sort: "updated" });
  }

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
    deleteProject,
    toggleFavorite,
    markOpened,
    openInFolder,
    setProjectTags,
    setProjectCollections,
    scanDirectory,
    importProjects,
    fetchAll,
  };
});

export type { Collection };
