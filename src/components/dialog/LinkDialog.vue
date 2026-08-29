<script setup lang="ts">
import { computed, ref } from "vue";
import { X } from "lucide-vue-next";
import { tauriApi } from "../../services/tauri";
import { useUiStore } from "../../stores/ui";
import { LINK_TYPE_OPTIONS } from "../../types";
import type { Link, LinkInput } from "../../types";

const props = defineProps<{
  projectId: string;
  link?: Link | null;
}>();

const emit = defineEmits<{
  close: [];
  saved: [];
}>();

const uiStore = useUiStore();

const title = ref(props.link?.title ?? "");
const url = ref(props.link?.url ?? "");
const linkType = ref(props.link?.linkType ?? "website");
const saving = ref(false);

const isEdit = computed(() => !!props.link);

async function save() {
  if (!title.value.trim()) {
    uiStore.showToast("链接名称不能为空", "error");
    return;
  }
  if (!url.value.trim()) {
    uiStore.showToast("链接地址不能为空", "error");
    return;
  }
  const input: LinkInput = {
    title: title.value,
    url: url.value,
    linkType: linkType.value,
  };
  saving.value = true;
  try {
    if (isEdit.value) {
      await tauriApi.updateLink(props.link!.id, input);
    } else {
      await tauriApi.addLink(props.projectId, input);
    }
    uiStore.showToast(isEdit.value ? "链接已更新" : "链接已添加", "success");
    emit("saved");
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
    <div class="modal" style="width: min(440px, 90vw)">
      <div class="modal-header">
        <h2>{{ isEdit ? "编辑链接" : "添加链接" }}</h2>
        <button class="modal-close" @click="emit('close')">
          <X :size="17" :stroke-width="1.8" />
        </button>
      </div>

      <div class="form-grid">
        <div class="field">
          <label>名称</label>
          <input v-model="title" type="text" placeholder="GitHub / 官网 / 文档 / Figma…" @keyup.enter="save" />
        </div>
        <div class="field">
          <label>地址</label>
          <input v-model="url" type="text" placeholder="https://…" @keyup.enter="save" />
        </div>
        <div class="field">
          <label>类型</label>
          <select v-model="linkType">
            <option v-for="opt in LINK_TYPE_OPTIONS" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </div>
      </div>

      <div class="modal-actions">
        <button class="btn" @click="emit('close')">取消</button>
        <button class="btn primary" :disabled="saving" @click="save">{{ saving ? "保存中…" : "保存" }}</button>
      </div>
    </div>
  </div>
</template>
