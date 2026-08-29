import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";
import ProjectsView from "../views/ProjectsView.vue";
import ProjectDetailView from "../views/ProjectDetailView.vue";
import TagsView from "../views/TagsView.vue";
import CollectionView from "../views/CollectionView.vue";
import TodosView from "../views/TodosView.vue";
import SettingsView from "../views/SettingsView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", redirect: "/home" },
    { path: "/home", name: "home", component: HomeView },
    { path: "/projects", name: "projects", component: ProjectsView },
    { path: "/projects/:id", name: "project-detail", component: ProjectDetailView, props: true },
    { path: "/recent", name: "recent", component: ProjectsView, meta: { mode: "recent" } },
    { path: "/favorites", name: "favorites", component: ProjectsView, meta: { mode: "favorite" } },
    { path: "/collections/:id", name: "collection", component: CollectionView, props: true },
    { path: "/tags", name: "tags", component: TagsView },
    { path: "/tags/:id", name: "tag", component: ProjectsView, props: true, meta: { mode: "tag" } },
    { path: "/todos", name: "todos", component: TodosView },
    { path: "/settings", name: "settings", component: SettingsView },
  ],
});
