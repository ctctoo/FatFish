<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import ProjectGrid from "../components/project/ProjectGrid.vue";
import ProjectListTable from "../components/project/ProjectListTable.vue";
import ProjectDialog from "../components/dialog/ProjectDialog.vue";
import ConfirmDialog from "../components/common/ConfirmDialog.vue";
import EmptyState from "../components/common/EmptyState.vue";
import Skeleton from "../components/common/Skeleton.vue";
import { useProjectStore } from "../stores/project";
import { useCollectionStore } from "../stores/collection";
import { useUiStore } from "../stores/ui";
import { useSettingsStore } from "../stores/settings";
import type { Project } from "../types";

const route = useRoute();
const router = useRouter();
const projectStore = useProjectStore();
const collectionStore = useCollectionStore();
const settingsStore = useSettingsStore();
const uiStore = useUiStore();

const loading = ref(true);
const showForm = ref(false);
const editing = ref(false);
const editName = ref("");
const deleteTarget = ref(false);

const collectionId = computed(() => route.params.id as string);
const collection = computed(() =>
  collectionStore.collections.find((c) => c.id === collectionId.value)
);

const projects = ref<Project[]>([]);

async function load() {
  loading.value = true;
  try {
    projects.value = await projectStore.fetchAll().then((all) =>
      all.filter((p) => p.collections.some((c) => c.id === collectionId.value))
    );
  } finally {
    loading.value = false;
  }
}

onMounted(load);
watch(collectionId, load);

async function rename() {
  if (!collection.value || !editName.value.trim()) {
    editing.value = false;
    return;
  }
  try {
    await collectionStore.updateCollection(collection.value.id, { name: editName.value.trim() });
    uiStore.showToast("集合已重命名", "success");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  } finally {
    editing.value = false;
  }
}

async function removeCollection() {
  if (!collection.value) return;
  try {
    await collectionStore.deleteCollection(collection.value.id);
    uiStore.showToast("集合已删除", "success");
    router.push("/projects");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <template v-if="editing && collection">
        <input
          v-model="editName"
          type="text"
          style="background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 6px 12px; font-size: 18px; outline: none"
          @keyup.enter="rename"
          @blur="rename"
        />
      </template>
      <template v-else>
        <h1>{{ collection?.name ?? "集合" }}</h1>
        <span class="count">{{ projects.length }} 个项目</span>
      </template>
      <span class="spacer"></span>
      <button
        v-if="collection && !editing"
        class="btn small ghost"
        @click="editing = true; editName = collection.name"
      >
        重命名
      </button>
      <button v-if="collection" class="btn small ghost danger" @click="deleteTarget = true">删除集合</button>
    </div>

    <Skeleton v-if="loading" :count="3" />
    <template v-else-if="projects.length">
      <ProjectGrid
        v-if="settingsStore.viewMode === 'grid'"
        :projects="projects"
        @open="(p) => router.push(`/projects/${p.id}`)"
        @open-folder="projectStore.openInFolder($event)"
        @toggle-favorite="projectStore.toggleFavorite($event)"
        @action="(_, p) => router.push(`/projects/${p.id}`)"
      />
      <ProjectListTable v-else :projects="projects" @open="(p) => router.push(`/projects/${p.id}`)" />
    </template>
    <EmptyState v-else title="这个集合还是空的" message="在项目编辑对话框中把它加入此集合。">
      <button class="btn primary" @click="showForm = true">＋ 新建项目</button>
    </EmptyState>

    <ProjectDialog v-if="showForm" @close="showForm = false" @saved="load()" />
    <ConfirmDialog
      v-if="deleteTarget"
      title="删除集合"
      :message="`删除集合「${collection?.name}」？项目本身不会被删除。`"
      confirm-text="删除"
      danger
      @confirm="removeCollection()"
      @cancel="deleteTarget = false"
    />
  </div>
</template>
