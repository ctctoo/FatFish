import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";
import ProjectsView from "../views/ProjectsView.vue";
import ProjectDetailView from "../views/ProjectDetailView.vue";
import TagsView from "../views/TagsView.vue";
import CollectionView from "../views/CollectionView.vue";
import TodosView from "../views/TodosView.vue";
import SettingsView from "../views/SettingsView.vue";
import GithubView from "../views/GithubView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", redirect: "/home" },
    { path: "/home", name: "home", component: HomeView, meta: { depth: 0 } },
    { path: "/projects", name: "projects", component: ProjectsView, meta: { depth: 0 } },
    { path: "/projects/:id", name: "project-detail", component: ProjectDetailView, props: true, meta: { depth: 1 } },
    { path: "/recent", name: "recent", component: ProjectsView, meta: { depth: 0, mode: "recent" } },
    { path: "/favorites", name: "favorites", component: ProjectsView, meta: { depth: 0, mode: "favorite" } },
    { path: "/collections/:id", name: "collection", component: CollectionView, props: true, meta: { depth: 1 } },
    { path: "/tags", name: "tags", component: TagsView, meta: { depth: 0 } },
    { path: "/tags/:id", name: "tag", component: ProjectsView, props: true, meta: { depth: 1, mode: "tag" } },
    { path: "/todos", name: "todos", component: TodosView, meta: { depth: 0 } },
    { path: "/settings", name: "settings", component: SettingsView, meta: { depth: 0 } },
    { path: "/github", name: "github", component: GithubView, meta: { depth: 0 } },
  ],
});
