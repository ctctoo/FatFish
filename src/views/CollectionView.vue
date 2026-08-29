<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import ProjectGrid from "../components/project/ProjectGrid.vue";
import ProjectListTable from "../components/project/ProjectListTable.vue";
import ProjectDialog from "../components/dialog/ProjectDialog.vue";
import CollectionAddDialog from "../components/dialog/CollectionAddDialog.vue";
import ConfirmDialog from "../components/common/ConfirmDialog.vue";
import EmptyState from "../components/common/EmptyState.vue";
import Skeleton from "../components/common/Skeleton.vue";
import { useProjectStore } from "../stores/project";
import { useCollectionStore } from "../stores/collection";
import { useUiStore } from "../stores/ui";
import { useSettingsStore } from "../stores/settings";
import { useI18n } from "../i18n";
import type { Project } from "../types";

const route = useRoute();
const router = useRouter();
const projectStore = useProjectStore();
const collectionStore = useCollectionStore();
const settingsStore = useSettingsStore();
const uiStore = useUiStore();
const { t } = useI18n();

const loading = ref(true);
const showForm = ref(false);
const showAdd = ref(false);
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
    uiStore.showToast(t("toast.collectionRenamed"), "success");
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
    uiStore.showToast(t("toast.collectionDeleted"), "success");
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
        <h1>{{ collection?.name ?? "" }}</h1>
        <span class="count">{{ t("collection.count", { n: projects.length }) }}</span>
      </template>
      <span class="spacer"></span>
      <button class="btn small" @click="showAdd = true">{{ t("collection.addProject") }}</button>
      <button
        v-if="collection && !editing"
        class="btn small ghost"
        @click="editing = true; editName = collection.name"
      >
        {{ t("common.rename") }}
      </button>
      <button v-if="collection" class="btn small ghost danger" @click="deleteTarget = true">{{ t("common.delete") }}</button>
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
    <EmptyState v-else :title="t('collection.emptyTitle')" :message="t('collection.emptyMsg')">
      <button class="btn primary" @click="showAdd = true">{{ t("collection.addProject") }}</button>
    </EmptyState>

    <CollectionAddDialog
      v-if="showAdd && collection"
      :collection-id="collection.id"
      :collection-name="collection.name"
      @close="showAdd = false"
      @added="load()"
    />
    <ProjectDialog v-if="showForm" @close="showForm = false" @saved="load()" />
    <ConfirmDialog
      v-if="deleteTarget"
      :title="t('confirm.deleteCollectionTitle')"
      :message="t('confirm.deleteCollectionMsg', { name: collection?.name ?? '' })"
      :confirm-text="t('confirm.delete')"
      danger
      @confirm="removeCollection()"
      @cancel="deleteTarget = false"
    />
  </div>
</template>
