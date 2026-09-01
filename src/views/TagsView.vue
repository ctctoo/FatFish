<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Plus } from "lucide-vue-next";
import { useTagStore } from "../stores/tag";
import { useUiStore } from "../stores/ui";
import { useProjectStore } from "../stores/project";
import ConfirmDialog from "../components/common/ConfirmDialog.vue";
import { useI18n } from "../i18n";

const router = useRouter();
const tagStore = useTagStore();
const projectStore = useProjectStore();
const uiStore = useUiStore();
const { t } = useI18n();

const newName = ref("");
const newColor = ref("");
const editingId = ref<string | null>(null);
const editName = ref("");
const editColor = ref("");
const counts = ref<Record<string, number>>({});
const deleteTarget = ref<{ id: string; name: string } | null>(null);

onMounted(async () => {
  await Promise.all([tagStore.fetchTags(), countProjects()]);
});

async function countProjects() {
  try {
    const projects = await projectStore.fetchAll();
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

async function remove() {
  if (!deleteTarget.value) return;
  try {
    await tagStore.deleteTag(deleteTarget.value.id);
    uiStore.showToast(t("toast.tagDeleted"), "success");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  } finally {
    deleteTarget.value = null;
  }
}
</script>

<template>
  <div class="page page-stagger">
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
          class="text-input"
          style="flex: 1"
          @keyup.enter="create"
        />
        <input v-model="newColor" type="color" title="color" />
        <button class="btn primary small" @click="create"><Plus :size="14" :stroke-width="1.8" /> {{ t("tags.newBtn") }}</button>
      </div>
    </div>

    <div class="tag-list">
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
          <button class="btn small ghost danger" @click="deleteTarget = { id: tag.id, name: tag.name }">{{ t("common.delete") }}</button>
        </template>
      </div>
      <p v-if="!tagStore.tags.length" class="text-secondary" style="font-size: 13px">
        {{ t("tags.empty") }}
      </p>
    </div>

    <ConfirmDialog
      v-if="deleteTarget"
      :title="t('confirm.deleteTagTitle')"
      :message="t('confirm.deleteTagMsg', { name: deleteTarget.name })"
      :confirm-text="t('confirm.delete')"
      danger
      @confirm="remove()"
      @cancel="deleteTarget = null"
    />
  </div>
</template>
