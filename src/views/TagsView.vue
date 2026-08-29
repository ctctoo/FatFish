<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Plus } from "lucide-vue-next";
import { useTagStore } from "../stores/tag";
import { useUiStore } from "../stores/ui";
import { useI18n } from "../i18n";
import { tauriApi } from "../services/tauri";
import type { Project } from "../types";

const router = useRouter();
const tagStore = useTagStore();
const uiStore = useUiStore();
const { t } = useI18n();

const newName = ref("");
const newColor = ref("");
const editingId = ref<string | null>(null);
const editName = ref("");
const editColor = ref("");
const counts = ref<Record<string, number>>({});

onMounted(async () => {
  await Promise.all([tagStore.fetchTags(), countProjects()]);
});

async function countProjects() {
  try {
    const projects: Project[] = await tauriApi.listProjects({});
    const map: Record<string, number> = {};
    for (const p of projects) {
      for (const item of p.tags) map[item.id] = (map[item.id] ?? 0) + 1;
    }
    counts.value = map;
  } catch {
    counts.value = {};
  }
}

async function create() {
  if (!newName.value.trim()) return;
  try {
    await tagStore.createTag({ name: newName.value, color: newColor.value || null });
    newName.value = "";
    newColor.value = "";
    uiStore.showToast(t("toast.tagCreated"), "success");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

function startEdit(id: string, name: string, color: string | null) {
  editingId.value = id;
  editName.value = name;
  editColor.value = color ?? "";
}

async function saveEdit() {
  if (!editingId.value) return;
  try {
    await tagStore.updateTag(editingId.value, { name: editName.value, color: editColor.value || null });
    editingId.value = null;
    uiStore.showToast(t("toast.tagUpdated"), "success");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

async function remove(id: string, name: string) {
  if (!confirm(t("confirm.deleteTagMsg", { name }))) return;
  try {
    await tagStore.deleteTag(id);
    uiStore.showToast(t("toast.tagDeleted"), "success");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1>{{ t("tags.title") }}</h1>
      <span class="count">{{ t("tags.total", { n: tagStore.tags.length }) }}</span>
    </div>

    <div class="settings-section">
      <div class="settings-row">
        <input
          v-model="newName"
          type="text"
          :placeholder="t('tags.newNamePh')"
          style="flex: 1; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 8px 12px; font-size: 13.5px; outline: none"
          @keyup.enter="create"
        />
        <input v-model="newColor" type="color" title="color" />
        <button class="btn primary small" @click="create"><Plus :size="14" :stroke-width="1.8" /> {{ t("tags.newBtn") }}</button>
      </div>
    </div>

    <div style="display: flex; flex-direction: column; gap: 8px; max-width: 640px">
      <div v-for="tag in tagStore.tags" :key="tag.id" class="entity-row">
        <template v-if="editingId === tag.id">
          <input v-model="editName" type="text" @keyup.enter="saveEdit" />
          <input v-model="editColor" type="color" />
          <button class="btn small primary" @click="saveEdit">{{ t("common.save") }}</button>
          <button class="btn small" @click="editingId = null">{{ t("common.cancel") }}</button>
        </template>
        <template v-else>
          <span
            class="tag-badge"
            :style="tag.color ? { borderColor: tag.color, color: tag.color } : undefined"
          >
            {{ tag.name }}
          </span>
          <span class="count">{{ t("tags.count", { n: counts[tag.id] ?? 0 }) }}</span>
          <span class="spacer"></span>
          <button class="btn small ghost" @click="startEdit(tag.id, tag.name, tag.color)">{{ t("common.edit") }}</button>
          <button class="btn small ghost" @click="router.push(`/tags/${tag.id}`)">{{ t("tags.viewProjects") }}</button>
          <button class="btn small ghost danger" @click="remove(tag.id, tag.name)">{{ t("common.delete") }}</button>
        </template>
      </div>
      <p v-if="!tagStore.tags.length" class="text-secondary" style="font-size: 13px">
        {{ t("tags.empty") }}
      </p>
    </div>
  </div>
</template>
