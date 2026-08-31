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
import { useProjectActions } from "../composables/useProjectActions";
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

// 动作统一分发（打开/终端/复制路径/状态切换/刷新 Git 等）
const { handleAction } = useProjectActions({
  onStatusChanged: (updated) => {
    const idx = recentProjects.value.findIndex((p) => p.id === updated.id);
    if (idx !== -1) recentProjects.value[idx] = updated;
  },
});
</script>

<template>
  <div class="page">
    <div class="home-hero">
      <div>
        <h1>{{ greeting }} <span style="font-weight: 400">👋</span></h1>
        <p>{{ t("home.subline") }}</p>
      </div>
      <span class="spacer"></span>
      <div class="view-toggle">
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

    <div class="home-section first">
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
          @open-folder="handleAction('open-folder', $event)"
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
