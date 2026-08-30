<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Plus, FolderSearch, LayoutGrid, List } from "lucide-vue-next";
import ProjectGrid from "../components/project/ProjectGrid.vue";
import ProjectListTable from "../components/project/ProjectListTable.vue";
import TodoWidget from "../components/todo/TodoWidget.vue";
import ProjectDialog from "../components/dialog/ProjectDialog.vue";
import ScanDialog from "../components/dialog/ScanDialog.vue";
import EmptyState from "../components/common/EmptyState.vue";
import Skeleton from "../components/common/Skeleton.vue";
import { useProjectStore } from "../stores/project";
import { useSettingsStore } from "../stores/settings";
import { useI18n } from "../i18n";
import type { Project } from "../types";

const router = useRouter();
const projectStore = useProjectStore();
const settingsStore = useSettingsStore();
const { t } = useI18n();

const recentProjects = ref<Project[]>([]);
const loading = ref(true);

const showForm = ref(false);
const showScan = ref(false);

// 时段化问候 + 用户称呼（初始引导收集，设置页可改）
const greetingKey = computed(() => {
  const h = new Date().getHours();
  if (h < 12) return "home.greetingMorning";
  if (h < 18) return "home.greetingAfternoon";
  return "home.greetingEvening";
});
const greeting = computed(() => {
  const name = settingsStore.profile?.name?.trim();
  const base = t(greetingKey.value);
  return name ? t("home.greetingName", { greeting: base, name }) : base;
});

onMounted(load);

async function load() {
  loading.value = true;
  try {
    await projectStore.fetchProjects();
    recentProjects.value = projectStore.projects.slice(0, 6);
  } finally {
    loading.value = false;
  }
}

async function openInFolder(project: Project) {
  await projectStore.openInFolder(project);
}

async function handleAction(action: string, project: Project) {
  if (action === "open-folder" || action === "open") {
    await openInFolder(project);
    return;
  }
  if (action.startsWith("status:")) {
    const updated = await projectStore.changeStatus(project, action.slice("status:".length));
    const idx = recentProjects.value.findIndex((p) => p.id === project.id);
    if (idx !== -1) recentProjects.value[idx] = updated;
    return;
  }
  router.push(`/projects/${project.id}`);
}
</script>

<template>
  <div class="page">
    <div class="home-hero" style="display: flex; align-items: flex-start">
      <div>
        <h1>{{ greeting }} <span style="font-weight: 400">👋</span></h1>
        <p>{{ t("home.subline") }}</p>
      </div>
      <span style="flex: 1"></span>
      <div class="view-toggle" style="margin-top: 8px">
        <button
          :class="{ active: settingsStore.viewMode === 'grid' }"
          :title="t('home.gridView')"
          @click="settingsStore.viewMode = 'grid'"
        >
          <LayoutGrid :size="15" :stroke-width="1.8" />
        </button>
        <button
          :class="{ active: settingsStore.viewMode === 'list' }"
          :title="t('home.listView')"
          @click="settingsStore.viewMode = 'list'"
        >
          <List :size="15" :stroke-width="1.8" />
        </button>
      </div>
    </div>

    <div class="home-section" style="margin-top: 14px">
      <TodoWidget mode="compact" />
    </div>

    <div class="home-section">
      <h2>{{ t("home.yourProjects") }}</h2>
      <Skeleton v-if="loading" :count="3" />
      <template v-else-if="recentProjects.length">
        <ProjectGrid
          v-if="settingsStore.viewMode === 'grid'"
          :projects="recentProjects"
          @open="(p) => router.push(`/projects/${p.id}`)"
          @open-folder="openInFolder"
          @toggle-favorite="projectStore.toggleFavorite($event)"
          @action="handleAction"
        />
        <ProjectListTable v-else :projects="recentProjects" @open="(p) => router.push(`/projects/${p.id}`)" />
      </template>
      <EmptyState v-else :title="t('home.emptyTitle')" :message="t('home.emptyMsg')">
        <button class="btn primary" @click="showForm = true"><Plus :size="15" :stroke-width="1.8" /> {{ t("home.addProject") }}</button>
        <button class="btn" @click="showScan = true"><FolderSearch :size="15" :stroke-width="1.8" /> {{ t("home.scanFolder") }}</button>
      </EmptyState>
    </div>

    <ProjectDialog v-if="showForm" @close="showForm = false" @saved="load()" />
    <ScanDialog v-if="showScan" @close="showScan = false" @imported="load()" />
  </div>
</template>
