<script setup lang="ts">
import { onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { X } from "lucide-vue-next";
import { useProjectStore } from "../../stores/project";
import { useSettingsStore } from "../../stores/settings";
import { useUiStore } from "../../stores/ui";
import { useI18n } from "../../i18n";
import type { ScannedProject } from "../../types";

const emit = defineEmits<{
  close: [];
  imported: [];
}>();

const projectStore = useProjectStore();
const settingsStore = useSettingsStore();
const uiStore = useUiStore();
const { t } = useI18n();

const rootDir = ref("");
const results = ref<ScannedProject[]>([]);
const selected = ref<Set<string>>(new Set());
const scanning = ref(false);
const importing = ref(false);

onMounted(() => {
  rootDir.value = settingsStore.defaultFolder || settingsStore.scanDirs[0] || "";
});

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
  results.value = [];
  selected.value.clear();
  try {
    results.value = await projectStore.scanDirectory(rootDir.value);
    for (const item of results.value) {
      if (!item.alreadyImported) selected.value.add(item.path);
    }
    settingsStore.addScanDir(rootDir.value);
    if (!settingsStore.defaultFolder) settingsStore.defaultFolder = rootDir.value;
    if (!results.value.length) uiStore.showToast(t("toast.scanEmpty"), "info");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  } finally {
    scanning.value = false;
  }
}

function toggle(path: string) {
  if (selected.value.has(path)) selected.value.delete(path);
  else selected.value.add(path);
}

async function importSelected() {
  const paths = [...selected.value];
  if (!paths.length) return;
  importing.value = true;
  try {
    const count = await projectStore.importProjects(paths);
    uiStore.showToast(t("toast.imported", { n: count }), "success");
    emit("imported");
    emit("close");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-header">
        <h2>{{ t("dialog.scan.title") }}</h2>
        <button class="modal-close" @click="emit('close')">
          <X :size="17" :stroke-width="1.8" />
        </button>
      </div>

      <div class="form-grid">
        <div class="field">
          <label>{{ t("dialog.scan.dirLabel") }}</label>
          <div class="path-row">
            <input v-model="rootDir" type="text" placeholder="D:\Projects" />
            <button class="btn" @click="pickDirectory">{{ t("common.browse") }}</button>
          </div>
        </div>

        <div v-if="settingsStore.scanDirs.length" class="field">
          <label>{{ t("dialog.scan.recent") }}</label>
          <div class="chip-row">
            <button v-for="dir in settingsStore.scanDirs" :key="dir" class="chip" @click="rootDir = dir">
              {{ dir }}
            </button>
          </div>
        </div>

        <div v-if="scanning" class="caption">{{ t("dialog.scan.discovering") }}</div>

        <div v-if="results.length" class="field">
          <label>{{ t("dialog.scan.found", { n: results.length, m: results.filter((r) => r.alreadyImported).length }) }}</label>
          <div class="scan-list">
            <label
              v-for="item in results"
              :key="item.path"
              class="scan-item"
              :class="{ dim: item.alreadyImported }"
            >
              <input
                type="checkbox"
                :checked="selected.has(item.path)"
                :disabled="item.alreadyImported"
                @change="toggle(item.path)"
              />
              <div style="min-width: 0">
                <div style="font-weight: 600; font-size: 13.5px">{{ item.name }}</div>
                <div class="caption" style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap">{{ item.path }}</div>
              </div>
              <span style="margin-left: auto; flex-shrink: 0" class="caption">
                {{ item.alreadyImported ? t("dialog.scan.imported") : item.language ?? "" }}
              </span>
            </label>
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button class="btn" @click="emit('close')">{{ t("common.cancel") }}</button>
        <button class="btn" :disabled="scanning" @click="scan">
          {{ scanning ? t("dialog.scan.scanning") : t("dialog.scan.start") }}
        </button>
        <button
          v-if="results.length"
          class="btn primary"
          :disabled="importing || !selected.size"
          @click="importSelected"
        >
          {{ importing ? t("dialog.scan.scanning") : t("dialog.scan.importBtn", { n: selected.size }) }}
        </button>
      </div>
    </div>
  </div>
</template>
