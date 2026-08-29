<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Plus, FolderSearch } from "lucide-vue-next";
import ProjectGrid from "../components/project/ProjectGrid.vue";
import TodoWidget from "../components/todo/TodoWidget.vue";
import ProjectDialog from "../components/dialog/ProjectDialog.vue";
import ScanDialog from "../components/dialog/ScanDialog.vue";
import EmptyState from "../components/common/EmptyState.vue";
import Skeleton from "../components/common/Skeleton.vue";
import { useProjectStore } from "../stores/project";

const router = useRouter();
const projectStore = useProjectStore();

const recentProjects = ref<typeof projectStore.projects>([]);
const loading = ref(true);

const showForm = ref(false);
const showScan = ref(false);

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

async function openInFolder(project: (typeof recentProjects.value)[number]) {
  await projectStore.openInFolder(project);
}

function handleAction(action: string, project: (typeof recentProjects.value)[number]) {
  if (action === "open-folder" || action === "open") openInFolder(project);
  else router.push(`/projects/${project.id}`);
}
</script>

<template>
  <div class="page">
    <div class="home-hero">
      <h1>Good afternoon</h1>
      <p>Keep track of everything you're working on.</p>
    </div>

    <div class="home-section" style="margin-top: 14px">
      <TodoWidget mode="compact" />
    </div>

    <div class="home-section">
      <h2>Your Projects</h2>
      <Skeleton v-if="loading" :count="3" />
      <ProjectGrid
        v-else-if="recentProjects.length"
        :projects="recentProjects"
        @open="(p) => router.push(`/projects/${p.id}`)"
        @open-folder="openInFolder"
        @toggle-favorite="projectStore.toggleFavorite($event)"
        @action="handleAction"
      />
      <EmptyState v-else title="还没有项目" message="添加本地项目，或扫描一个文件夹来建立你的项目空间。">
        <button class="btn primary" @click="showForm = true"><Plus :size="15" :stroke-width="1.8" /> 添加项目</button>
        <button class="btn" @click="showScan = true"><FolderSearch :size="15" :stroke-width="1.8" /> 扫描文件夹</button>
      </EmptyState>
    </div>

    <ProjectDialog v-if="showForm" @close="showForm = false" @saved="load()" />
    <ScanDialog v-if="showScan" @close="showScan = false" @imported="load()" />
  </div>
</template>
