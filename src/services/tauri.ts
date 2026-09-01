import { invoke } from "@tauri-apps/api/core";
import type {
  Collection,
  CollectionInput,
  Link,
  LinkInput,
  Project,
  ProjectFilter,
  ProjectInput,
  ScannedProject,
  Tag,
  TagInput,
  Todo,
  TodoInput,
  UpdateInfo,
  Activity,
  GithubAccount,
  GithubDeviceCode,
  GithubLoginResult,
  GithubRepo,
} from "../types";

export const tauriApi = {
  // ---- projects ----
  listProjects(filter: ProjectFilter): Promise<Project[]> {
    return invoke("list_projects", { filter: Object.keys(filter).length ? filter : null });
  },
  getProject(id: string): Promise<Project> {
    return invoke("get_project", { id });
  },
  createProject(input: ProjectInput): Promise<Project> {
    return invoke("create_project", { input });
  },
  updateProject(id: string, input: ProjectInput): Promise<Project> {
    return invoke("update_project", { id, input });
  },
  deleteProject(id: string): Promise<void> {
    return invoke("delete_project", { id });
  },
  setFavorite(id: string, favorite: boolean): Promise<void> {
    return invoke("set_favorite", { id, favorite });
  },
  markOpened(id: string): Promise<void> {
    return invoke("mark_opened", { id });
  },
  setProjectTags(projectId: string, tagIds: string[]): Promise<void> {
    return invoke("set_project_tags", { projectId, tagIds });
  },
  setProjectCollections(projectId: string, collectionIds: string[]): Promise<void> {
    return invoke("set_project_collections", { projectId, collectionIds });
  },

  // ---- tags ----
  listTags(): Promise<Tag[]> {
    return invoke("list_tags");
  },
  createTag(input: TagInput): Promise<Tag> {
    return invoke("create_tag", { input });
  },
  updateTag(id: string, input: TagInput): Promise<void> {
    return invoke("update_tag", { id, input });
  },
  deleteTag(id: string): Promise<void> {
    return invoke("delete_tag", { id });
  },

  // ---- collections ----
  listCollections(): Promise<Collection[]> {
    return invoke("list_collections");
  },
  createCollection(input: CollectionInput): Promise<Collection> {
    return invoke("create_collection", { input });
  },
  updateCollection(id: string, input: CollectionInput): Promise<void> {
    return invoke("update_collection", { id, input });
  },
  deleteCollection(id: string): Promise<void> {
    return invoke("delete_collection", { id });
  },

  // ---- links ----
  addLink(projectId: string, input: LinkInput): Promise<Link> {
    return invoke("add_link", { projectId, input });
  },
  updateLink(id: string, input: LinkInput): Promise<void> {
    return invoke("update_link", { id, input });
  },
  deleteLink(id: string): Promise<void> {
    return invoke("delete_link", { id });
  },

  // ---- todos ----
  listTodos(): Promise<Todo[]> {
    return invoke("list_todos");
  },
  createTodo(input: TodoInput): Promise<Todo> {
    return invoke("create_todo", { input });
  },
  updateTodo(id: string, input: TodoInput): Promise<void> {
    return invoke("update_todo", { id, input });
  },
  toggleTodo(id: string, done: boolean): Promise<void> {
    return invoke("toggle_todo", { id, done });
  },
  deleteTodo(id: string): Promise<void> {
    return invoke("delete_todo", { id });
  },

  // ---- activities ----
  listActivities(projectId: string): Promise<Activity[]> {
    return invoke("list_activities", { projectId });
  },

  // ---- scanner ----
  scanDirectory(path: string): Promise<ScannedProject[]> {
    return invoke("scan_directory", { path });
  },
  importProjects(paths: string[], collectionId?: string | null): Promise<ScannedProject[]> {
    return invoke("import_projects", { paths, collectionId: collectionId ?? null });
  },

  // ---- git ----
  refreshGitInfo(projectId: string): Promise<Project> {
    return invoke("refresh_git_info", { projectId });
  },

  // ---- github ----
  githubLoginStart(clientId: string): Promise<GithubDeviceCode> {
    return invoke("github_login_start", { clientId });
  },
  githubLoginPoll(clientId: string, deviceCode: string): Promise<GithubLoginResult> {
    return invoke("github_login_poll", { clientId, deviceCode });
  },
  githubStatus(): Promise<GithubAccount | null> {
    return invoke("github_status");
  },
  githubLogout(): Promise<void> {
    return invoke("github_logout");
  },
  githubListRepos(): Promise<GithubRepo[]> {
    return invoke("github_list_repos");
  },

  // ---- update ----
  checkForUpdate(currentVersion: string): Promise<UpdateInfo | null> {
    return invoke("check_for_update", { currentVersion });
  },

  // ---- system ----
  openFolder(path: string): Promise<void> {
    return invoke("open_folder", { path });
  },
  openTerminal(path: string): Promise<void> {
    return invoke("open_terminal", { path });
  },
};
