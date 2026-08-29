<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { X, Search, FolderSearch } from "lucide-vue-next";
import { tauriApi } from "../../services/tauri";
import { useSettingsStore } from "../../stores/settings";
import { useUiStore } from "../../stores/ui";
import { useI18n } from "../../i18n";
import type { Project, ScannedProject } from "../../types";

const props = defineProps<{
  collectionId: string;
  collectionName: string;
}>();

const emit = defineEmits<{
  close: [];
  added: [];
}>();

const settingsStore = useSettingsStore();
const uiStore = useUiStore();
const { t } = useI18n();

// ---- 模式一：从现有项目多选 ----
const allProjects = ref<Project[]>([]);
const search = ref("");
const selectedIds = ref<Set<string>>(new Set());

const candidates = computed(() => {
  const q = search.value.trim().toLowerCase();
  return allProjects.value.filter((p) => {
    if (p.collections.some((c) => c.id === props.collectionId)) return false;
    if (!q) return true;
    return p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q);
  });
});

function toggleExisting(id: string) {
  if (selectedIds.value.has(id)) selectedIds.value.delete(id);
  else selectedIds.value.add(id);
}

// ---- 模式二：扫描导入 ----
const rootDir = ref("");
const scanResults = ref<ScannedProject[]>([]);
const scanSelected = ref<Set<string>>(new Set());
const scanning = ref(false);

async function pickDirectory() {
  const dir = await open({ directory: true, multiple: false, title: t("dialog.scan.title") });
  if (typeof dir === "string") rootDir.value = dir;
}

async function scan() {
  if (!rootDir.value.trim()) {
    uiStore.showToast(t("toast.pickScanDir"), "error");
    return;
  }
  scanning.value = true;
  scanResults.value = [];
  scanSelected.value.clear();
  try {
    scanResults.value = await tauriApi.scanDirectory(rootDir.value);
    for (const item of scanResults.value) {
      if (!item.alreadyImported) scanSelected.value.add(item.path);
    }
    if (!scanResults.value.length) uiStore.showToast(t("toast.scanEmpty"), "info");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  } finally {
    scanning.value = false;
  }
}

function toggleScan(path: string) {
  if (scanSelected.value.has(path)) scanSelected.value.delete(path);
  else scanSelected.value.add(path);
}

const busy = ref(false);

async function addExisting() {
  if (!selectedIds.value.size) return;
  busy.value = true;
  try {
    for (const project of allProjects.value) {
      if (!selectedIds.value.has(project.id)) continue;
      await tauriApi.setProjectCollections(project.id, [
        ...project.collections.map((c) => c.id),
        props.collectionId,
      ]);
    }
    uiStore.showToast(t("toast.projectsAdded", { n: selectedIds.value.size }), "success");
    emit("added");
    emit("close");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  } finally {
    busy.value = false;
  }
}

async function importScanned() {
  if (!scanSelected.value.size) return;
  busy.value = true;
  try {
    const imported = await tauriApi.importProjects([...scanSelected.value], props.collectionId);
    uiStore.showToast(t("toast.importedToCollection", { n: imported.length }), "success");
    emit("added");
    emit("close");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  } finally {
    busy.value = false;
  }
}

onMounted(async () => {
  allProjects.value = await tauriApi.listProjects({ sort: "name" });
  rootDir.value = settingsStore.defaultFolder || settingsStore.scanDirs[0] || "";
});
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal wide">
      <div class="modal-header">
        <h2>{{ t("dialog.collectionAdd.to", { name: props.collectionName }) }}</h2>
        <button class="modal-close" @click="emit('close')">
          <X :size="17" :stroke-width="1.8" />
        </button>
      </div>

      <!-- 从现有项目多选 -->
      <div class="add-section">
        <div class="add-section-title">{{ t("dialog.collectionAdd.fromExisting") }}</div>
        <div class="add-search">
          <Search :size="14" :stroke-width="1.8" />
          <input v-model="search" type="text" :placeholder="t('dialog.collectionAdd.searchPh')" />
        </div>
        <div class="add-list">
          <label
            v-for="project in candidates"
            :key="project.id"
            class="add-item"
            :class="{ checked: selectedIds.has(project.id) }"
          >
            <input
              type="checkbox"
              :checked="selectedIds.has(project.id)"
              @change="toggleExisting(project.id)"
            />
            <span class="add-item-name">{{ project.name }}</span>
            <span class="caption add-item-path">{{ project.path }}</span>
          </label>
          <div v-if="!candidates.length" class="caption" style="padding: 12px 4px">
            {{ search ? t("dialog.collectionAdd.noMatch") : t("dialog.collectionAdd.allInCollection") }}
          </div>
        </div>
        <div style="display: flex; justify-content: flex-end; margin-top: 10px">
          <button class="btn primary small" :disabled="busy || !selectedIds.size" @click="addExisting">
            {{ busy ? t("dialog.collectionAdd.adding") : t("dialog.collectionAdd.addSelected", { n: selectedIds.size }) }}
          </button>
        </div>
      </div>

      <!-- 扫描导入 -->
      <div class="add-section" style="border-top: 1px solid var(--border); padding-top: 14px; margin-top: 16px">
        <div class="add-section-title">{{ t("dialog.collectionAdd.importSection") }}</div>
        <div class="path-row" style="margin: 8px 0">
          <input v-model="rootDir" type="text" placeholder="D:\Projects" />
          <button class="btn" @click="pickDirectory">{{ t("common.browse") }}</button>
          <button class="btn" :disabled="scanning" @click="scan">
            <FolderSearch :size="14" :stroke-width="1.8" />
            {{ scanning ? t("dialog.scan.scanning") : t("dialog.scan.start") }}
          </button>
        </div>
        <div v-if="scanResults.length" class="add-list" style="max-height: 180px">
          <label
            v-for="item in scanResults"
            :key="item.path"
            class="add-item"
            :class="{ dim: item.alreadyImported, checked: scanSelected.has(item.path) }"
          >
            <input
              type="checkbox"
              :checked="scanSelected.has(item.path)"
              :disabled="item.alreadyImported"
              @change="toggleScan(item.path)"
            />
            <span class="add-item-name">{{ item.name }}</span>
            <span class="caption add-item-path">{{ item.path }}</span>
            <span v-if="item.alreadyImported" class="caption" style="flex-shrink: 0">{{ t("dialog.collectionAdd.inLibrary") }}</span>
          </label>
        </div>
        <div v-if="scanResults.length" style="display: flex; justify-content: flex-end; margin-top: 10px">
          <button class="btn primary small" :disabled="busy || !scanSelected.size" @click="importScanned">
            {{ busy ? t("dialog.collectionAdd.adding") : t("dialog.scan.importBtn", { n: scanSelected.size }) }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.add-section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.add-search {
  position: relative;
  margin-bottom: 8px;
}

.add-search svg {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-tertiary);
  pointer-events: none;
}

.add-search input {
  width: 100%;
  padding: 7px 12px 7px 32px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 13px;
  outline: none;
}

.add-search input:focus {
  border-color: var(--border-strong);
}

.add-list {
  max-height: 240px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 6px;
  background: var(--bg);
}

.add-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  min-width: 0;
}

.add-item:hover {
  background: var(--hover);
}

.add-item.checked {
  background: var(--accent-soft);
}

.add-item.dim {
  opacity: 0.55;
}

.add-item input[type="checkbox"] {
  accent-color: var(--accent);
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.add-item-name {
  font-size: 13px;
  font-weight: 500;
  flex-shrink: 0;
}

.add-item-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-left: auto;
  padding-left: 12px;
}
</style>
