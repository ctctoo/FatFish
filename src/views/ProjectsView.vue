<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Search, ListFilter, ArrowUpDown, LayoutGrid, List, Plus, FolderSearch } from "lucide-vue-next";
import ProjectGrid from "../components/project/ProjectGrid.vue";
import ProjectListTable from "../components/project/ProjectListTable.vue";
import ProjectDialog from "../components/dialog/ProjectDialog.vue";
import ScanDialog from "../components/dialog/ScanDialog.vue";
import LinkDialog from "../components/dialog/LinkDialog.vue";
import ConfirmDialog from "../components/common/ConfirmDialog.vue";
import EmptyState from "../components/common/EmptyState.vue";
import Skeleton from "../components/common/Skeleton.vue";
import { tauriApi } from "../services/tauri";
import { useProjectStore } from "../stores/project";
import { useTagStore } from "../stores/tag";
import { useCollectionStore } from "../stores/collection";
import { useSettingsStore } from "../stores/settings";
import { useUiStore } from "../stores/ui";
import { STATUS_VALUES } from "../types";
import { statusLabel, useI18n } from "../i18n";
import type { Project } from "../types";

const route = useRoute();
const router = useRouter();
const projectStore = useProjectStore();
const tagStore = useTagStore();
const collectionStore = useCollectionStore();
const settingsStore = useSettingsStore();
const uiStore = useUiStore();
const { t } = useI18n();

const mode = computed(() => (route.meta.mode as string) ?? "all");

const title = computed(() => {
  switch (mode.value) {
    case "recent":
      return t("projects.recent");
    case "favorite":
      return t("projects.favorites");
    case "tag": {
      const tag = tagStore.tags.find((item) => item.id === route.params.id);
      return tag ? `${t("projects.tagPrefix")}${tag.name}` : t("side.tags");
    }
    default:
      return t("projects.all");
  }
});

const showForm = ref(false);
const editingProject = ref<Project | null>(null);
const showScan = ref(false);
const linkTarget = ref<Project | null>(null);
const deleteTarget = ref<Project | null>(null);

const showFilter = ref(false);
const showSort = ref(false);

const activeCount = computed(() => {
  let n = 0;
  if (projectStore.status) n++;
  if (projectStore.favorite && mode.value !== "favorite") n++;
  if (projectStore.tagId) n++;
  if (projectStore.collectionId) n++;
  return n;
});

function syncFromRoute() {
  projectStore.resetFilter();
  projectStore.query = "";
  const m = mode.value;
  if (m === "recent") projectStore.recent = true;
  if (m === "favorite") projectStore.favorite = true;
  if (m === "tag") projectStore.tagId = (route.params.id as string) ?? null;
  projectStore.fetchProjects();
}

watch([mode, () => route.params.id], () => syncFromRoute());
watch(() => settingsStore.sort, () => projectStore.fetchProjects());

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  () => projectStore.query,
  () => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => projectStore.fetchProjects(), 250);
  }
);

watch(
  () => [projectStore.status, projectStore.favorite, projectStore.tagId, projectStore.collectionId],
  () => projectStore.fetchProjects()
);

onMounted(async () => {
  await Promise.all([tagStore.fetchTags(), collectionStore.fetchCollections()]);
  syncFromRoute();
});

function openDetail(project: Project) {
  router.push(`/projects/${project.id}`);
}

function newProject() {
  editingProject.value = null;
  showForm.value = true;
}

function editProject(project: Project) {
  editingProject.value = project;
  showForm.value = true;
}

function handleAction(action: string, project: Project) {
  const run = async () => {
    if (action.startsWith("status:")) {
      await projectStore.changeStatus(project, action.slice("status:".length));
      uiStore.showToast(t("toast.statusChanged"), "success");
      return;
    }
    switch (action) {
      case "open":
      case "open-folder":
        await projectStore.openInFolder(project);
        break;
      case "open-terminal":
        await tauriApi.openTerminal(project.path);
        break;
      case "open-github": {
        const gh = project.links.find((l) => l.linkType === "github");
        if (gh) await openUrl(gh.url);
        break;
      }
      case "copy-path":
        await navigator.clipboard.writeText(project.path);
        uiStore.showToast(t("toast.copied"), "success");
        break;
      case "edit":
        editProject(project);
        break;
      case "add-link":
        linkTarget.value = project;
        break;
      case "refresh-git":
        await tauriApi.refreshGitInfo(project.id);
        await projectStore.fetchProjects();
        uiStore.showToast(t("toast.gitRefreshed"), "success");
        break;
      case "delete":
        if (settingsStore.confirmRemove) {
          deleteTarget.value = project;
        } else {
          await removeProject(project);
        }
        break;
    }
  };
  run().catch((e) => uiStore.showToast(String(e), "error"));
}

async function removeProject(project: Project) {
  await projectStore.deleteProject(project.id);
  uiStore.showToast(t("toast.projectDeleted"), "success");
}

function setSort(key: string) {
  settingsStore.sort = key as typeof settingsStore.sort;
  showSort.value = false;
}

const sortLabel = computed(
  () =>
    ({ updated: t("projects.sortUpdated"), name: t("projects.sortName"), opened: t("projects.sortOpened") })[
      settingsStore.sort
    ] ?? t("projects.sort")
);
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1>{{ title }}</h1>
      <span class="count">{{ t("projects.count", { n: projectStore.projects.length }) }}</span>
      <span class="spacer"></span>
      <button class="btn" @click="showScan = true">
        <FolderSearch :size="15" :stroke-width="1.8" /> {{ t("projects.scanFolder") }}
      </button>
      <button class="btn primary" @click="newProject">
        <Plus :size="15" :stroke-width="1.8" /> {{ t("projects.newProject") }}
      </button>
    </div>

    <div class="page-toolbar" v-if="mode === 'all'">
      <div class="toolbar-search">
        <Search :size="14" :stroke-width="1.8" />
        <input v-model="projectStore.query" type="text" :placeholder="t('projects.searchPh')" />
      </div>

      <div style="position: relative">
        <button class="tool-btn" :class="{ active: showFilter || activeCount > 0 }" @click="showFilter = !showFilter">
          <ListFilter :size="14" :stroke-width="1.8" />
          {{ t("projects.filter") }}{{ activeCount ? ` (${activeCount})` : "" }}
        </button>
        <div v-if="showFilter" class="popover" style="left: 0">
          <h4>{{ t("projects.filterStatus") }}</h4>
          <button class="opt-row" :class="{ active: !projectStore.status }" @click="projectStore.status = null">
            {{ t("projects.filterAll") }}
          </button>
          <button
            v-for="value in STATUS_VALUES"
            :key="value"
            class="opt-row"
            :class="{ active: projectStore.status === value }"
            @click="projectStore.status = value"
          >
            <span class="status-dot" :class="`status-${value}`"></span>
            {{ statusLabel(settingsStore.locale, value) }}
          </button>

          <h4>{{ t("projects.filterCollections") }}</h4>
          <button
            v-for="c in collectionStore.collections"
            :key="c.id"
            class="opt-row"
            :class="{ active: projectStore.collectionId === c.id }"
            @click="projectStore.collectionId = projectStore.collectionId === c.id ? null : c.id"
          >
            {{ c.name }}
          </button>

          <h4>{{ t("projects.filterTags") }}</h4>
          <button
            v-for="item in tagStore.tags"
            :key="item.id"
            class="opt-row"
            :class="{ active: projectStore.tagId === item.id }"
            @click="projectStore.tagId = projectStore.tagId === item.id ? null : item.id"
          >
            {{ item.name }}
          </button>

          <h4>{{ t("projects.filterOther") }}</h4>
          <button
            class="opt-row"
            :class="{ active: projectStore.favorite }"
            @click="projectStore.favorite = !projectStore.favorite"
          >
            {{ t("projects.favOnly") }}
          </button>
        </div>
      </div>

      <div style="position: relative">
        <button class="tool-btn" @click="showSort = !showSort">
          <ArrowUpDown :size="14" :stroke-width="1.8" />
          {{ sortLabel }}
        </button>
        <div v-if="showSort" class="popover" style="left: 0; width: 160px">
          <button class="opt-row" :class="{ active: settingsStore.sort === 'updated' }" @click="setSort('updated')">{{ t("projects.sortUpdated") }}</button>
          <button class="opt-row" :class="{ active: settingsStore.sort === 'name' }" @click="setSort('name')">{{ t("projects.sortName") }}</button>
          <button class="opt-row" :class="{ active: settingsStore.sort === 'opened' }" @click="setSort('opened')">{{ t("projects.sortOpened") }}</button>
        </div>
      </div>

      <span class="spacer" style="flex: 1"></span>

      <div class="view-toggle">
        <button :class="{ active: settingsStore.viewMode === 'grid' }" :title="t('projects.grid')" @click="settingsStore.viewMode = 'grid'">
          <LayoutGrid :size="15" :stroke-width="1.8" />
        </button>
        <button :class="{ active: settingsStore.viewMode === 'list' }" :title="t('projects.list')" @click="settingsStore.viewMode = 'list'">
          <List :size="15" :stroke-width="1.8" />
        </button>
      </div>
    </div>

    <p v-if="projectStore.error" style="color: #c0554f; font-size: 13px">{{ projectStore.error }}</p>

    <Skeleton v-if="projectStore.loading && !projectStore.projects.length" :count="6" />

    <template v-else-if="projectStore.projects.length">
      <ProjectGrid
        v-if="settingsStore.viewMode === 'grid'"
        :projects="projectStore.projects"
        @open="openDetail"
        @open-folder="handleAction('open-folder', $event)"
        @toggle-favorite="projectStore.toggleFavorite($event)"
        @action="handleAction"
      />
      <ProjectListTable v-else :projects="projectStore.projects" @open="openDetail" />
    </template>

    <EmptyState
      v-else
      :title="projectStore.query ? t('projects.emptySearchTitle') : t('projects.emptyTitle')"
      :message="projectStore.query ? t('projects.emptySearchMsg') : t('projects.emptyMsg')"
    >
      <template v-if="!projectStore.query">
        <button class="btn primary" @click="newProject">＋ {{ t("home.addProject") }}</button>
        <button class="btn" @click="showScan = true">{{ t("home.scanFolder") }}</button>
      </template>
    </EmptyState>

    <ProjectDialog v-if="showForm" :project="editingProject" @close="showForm = false" @saved="projectStore.fetchProjects()" />
    <ScanDialog v-if="showScan" @close="showScan = false" @imported="projectStore.fetchProjects()" />
    <LinkDialog v-if="linkTarget" :project-id="linkTarget.id" @close="linkTarget = null" @saved="projectStore.fetchProjects()" />
    <ConfirmDialog
      v-if="deleteTarget"
      :title="t('confirm.deleteProjectTitle')"
      :message="t('confirm.deleteProjectMsg', { name: deleteTarget.name })"
      :confirm-text="t('confirm.delete')"
      danger
      @confirm="removeProject(deleteTarget!); deleteTarget = null"
      @cancel="deleteTarget = null"
    />
  </div>
</template>
