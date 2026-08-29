<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { X } from "lucide-vue-next";
import { useProjectStore } from "../../stores/project";
import { useSettingsStore } from "../../stores/settings";
import { useUiStore } from "../../stores/ui";
import { STATUS_OPTIONS } from "../../types";
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
    selectedTagIds.value = props.project.tags.map((t) => t.id);
    selectedCollectionIds.value = props.project.collections.map((c) => c.id);
  } else if (settingsStore.defaultFolder) {
    path.value = settingsStore.defaultFolder;
  }
});

async function pickDirectory() {
  const dir = await open({
    directory: true,
    multiple: false,
    defaultPath: settingsStore.defaultFolder || undefined,
    title: "选择项目文件夹",
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
    uiStore.showToast("项目名称不能为空", "error");
    return;
  }
  if (!path.value.trim()) {
    uiStore.showToast("项目路径不能为空", "error");
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
    uiStore.showToast(isEdit.value ? "项目已更新" : "项目已添加", "success");
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
        <h2>{{ isEdit ? "编辑项目" : "新建项目" }}</h2>
        <button class="modal-close" @click="emit('close')">
          <X :size="17" :stroke-width="1.8" />
        </button>
      </div>

      <div class="form-grid">
        <div class="field">
          <label>Name</label>
          <input v-model="name" type="text" placeholder="项目名称" />
        </div>

        <div class="field">
          <label>Location</label>
          <div class="path-row">
            <input v-model="path" type="text" placeholder="D:\Projects\xxx" :disabled="isEdit" />
            <button class="btn" :disabled="isEdit" @click="pickDirectory">浏览…</button>
          </div>
        </div>

        <div class="field">
          <label>Description</label>
          <textarea v-model="description" placeholder="这个项目是做什么的？"></textarea>
        </div>

        <ProjectOrgSelector
          :selected-tag-ids="selectedTagIds"
          :selected-collection-ids="selectedCollectionIds"
          @toggle-tag="toggle(selectedTagIds, $event)"
          @toggle-collection="toggle(selectedCollectionIds, $event)"
        />

        <div class="form-row">
          <div class="field">
            <label>Icon（Emoji，可选）</label>
            <input v-model="coverEmoji" type="text" placeholder="如 ✈️ 📚 🎨，留空自动生成" />
          </div>
          <div class="field">
            <label>Status</label>
            <select v-model="status">
              <option v-for="opt in STATUS_OPTIONS" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button class="btn" @click="emit('close')">取消</button>
        <button class="btn primary" :disabled="saving" @click="save">
          {{ saving ? "保存中…" : isEdit ? "保存" : "创建" }}
        </button>
      </div>
    </div>
  </div>
</template>
