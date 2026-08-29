<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { X } from "lucide-vue-next";
import { useProjectStore } from "../../stores/project";
import { useSettingsStore } from "../../stores/settings";
import { useUiStore } from "../../stores/ui";
import { STATUS_VALUES } from "../../types";
import { statusLabel, useI18n } from "../../i18n";
import type { Project, ProjectInput } from "../../types";
import ProjectOrgSelector from "../common/ProjectOrgSelector.vue";

const props = defineProps<{
  project?: Project | null;
}>();

const emit = defineEmits<{
  close: [];
  saved: [project: Project];
}>();

const projectStore = useProjectStore();
const settingsStore = useSettingsStore();
const uiStore = useUiStore();
const { t } = useI18n();

const name = ref("");
const path = ref("");
const description = ref("");
const notes = ref("");
const status = ref("IN_PROGRESS");
const coverEmoji = ref("");
const selectedTagIds = ref<string[]>([]);
const selectedCollectionIds = ref<string[]>([]);
const saving = ref(false);

const isEdit = computed(() => !!props.project);

onMounted(() => {
  if (props.project) {
    name.value = props.project.name;
    path.value = props.project.path;
    description.value = props.project.description ?? "";
    notes.value = props.project.notes ?? "";
    status.value = props.project.status;
    coverEmoji.value = props.project.coverEmoji ?? "";
    selectedTagIds.value = props.project.tags.map((item) => item.id);
    selectedCollectionIds.value = props.project.collections.map((item) => item.id);
  } else if (settingsStore.defaultFolder) {
    path.value = settingsStore.defaultFolder;
  }
});

async function pickDirectory() {
  const dir = await open({
    directory: true,
    multiple: false,
    defaultPath: settingsStore.defaultFolder || undefined,
    title: t("dialog.project.location"),
  });
  if (typeof dir === "string") {
    path.value = dir;
    if (!name.value) {
      name.value = dir.split(/[\\/]/).filter(Boolean).pop() ?? "";
    }
  }
}

function toggle(list: string[], id: string) {
  const idx = list.indexOf(id);
  if (idx >= 0) list.splice(idx, 1);
  else list.push(id);
}

async function save() {
  if (!name.value.trim()) {
    uiStore.showToast(t("toast.nameRequired"), "error");
    return;
  }
  if (!path.value.trim()) {
    uiStore.showToast(t("toast.pathRequired"), "error");
    return;
  }
  const input: ProjectInput = {
    name: name.value,
    path: path.value,
    description: description.value.trim() || null,
    status: status.value,
    coverEmoji: coverEmoji.value.trim() || null,
    coverColor: props.project?.coverColor ?? null,
    notes: notes.value.trim() || null,
  };
  saving.value = true;
  try {
    const saved = isEdit.value
      ? await projectStore.updateProject(props.project!.id, input)
      : await projectStore.createProject(input);
    await projectStore.setProjectTags(saved.id, selectedTagIds.value);
    await projectStore.setProjectCollections(saved.id, selectedCollectionIds.value);
    uiStore.showToast(isEdit.value ? t("dialog.project.savedAsEdit") : t("dialog.project.savedAsNew"), "success");
    emit("saved", saved);
    emit("close");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-header">
        <h2>{{ isEdit ? t("dialog.project.edit") : t("dialog.project.new") }}</h2>
        <button class="modal-close" @click="emit('close')">
          <X :size="17" :stroke-width="1.8" />
        </button>
      </div>

      <div class="form-grid">
        <div class="field">
          <label>{{ t("dialog.project.name") }}</label>
          <input v-model="name" type="text" :placeholder="t('dialog.project.name')" />
        </div>

        <div class="field">
          <label>{{ t("dialog.project.location") }}</label>
          <div class="path-row">
            <input v-model="path" type="text" :placeholder="t('dialog.project.pathPh')" :disabled="isEdit" />
            <button class="btn" :disabled="isEdit" @click="pickDirectory">{{ t("common.browse") }}</button>
          </div>
        </div>

        <div class="field">
          <label>{{ t("dialog.project.description") }}</label>
          <textarea v-model="description" :placeholder="t('dialog.project.descPh')"></textarea>
        </div>

        <ProjectOrgSelector
          :selected-tag-ids="selectedTagIds"
          :selected-collection-ids="selectedCollectionIds"
          @toggle-tag="toggle(selectedTagIds, $event)"
          @toggle-collection="toggle(selectedCollectionIds, $event)"
        />

        <div class="form-row">
          <div class="field">
            <label>{{ t("dialog.project.icon") }}</label>
            <input v-model="coverEmoji" type="text" :placeholder="t('dialog.project.iconPh')" />
          </div>
          <div class="field">
            <label>{{ t("dialog.project.status") }}</label>
            <select v-model="status">
              <option v-for="value in STATUS_VALUES" :key="value" :value="value">{{ statusLabel(settingsStore.locale, value) }}</option>
            </select>
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button class="btn" @click="emit('close')">{{ t("common.cancel") }}</button>
        <button class="btn primary" :disabled="saving" @click="save">
          {{ saving ? t("common.save") + "…" : isEdit ? t("common.save") : t("common.create") }}
        </button>
      </div>
    </div>
  </div>
</template>
