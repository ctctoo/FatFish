<script setup lang="ts">
import { computed, ref } from "vue";
import { X } from "lucide-vue-next";
import { tauriApi } from "../../services/tauri";
import { useUiStore } from "../../stores/ui";
import { LINK_TYPE_VALUES } from "../../types";
import { linkTypeLabel, useI18n } from "../../i18n";
import { useSettingsStore } from "../../stores/settings";
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
const settings = useSettingsStore();
const { t } = useI18n();

const title = ref(props.link?.title ?? "");
const url = ref(props.link?.url ?? "");
const linkType = ref(props.link?.linkType ?? "website");
const saving = ref(false);

const isEdit = computed(() => !!props.link);

async function save() {
  if (!title.value.trim()) {
    uiStore.showToast(t("toast.titleRequired"), "error");
    return;
  }
  if (!url.value.trim()) {
    uiStore.showToast(t("toast.urlRequired"), "error");
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
    uiStore.showToast(isEdit.value ? t("toast.linkUpdated") : t("toast.linkAdded"), "success");
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
        <h2>{{ isEdit ? t("dialog.link.edit") : t("dialog.link.add") }}</h2>
        <button class="modal-close" @click="emit('close')">
          <X :size="17" :stroke-width="1.8" />
        </button>
      </div>

      <div class="form-grid">
        <div class="field">
          <label>{{ t("dialog.link.title") }}</label>
          <input v-model="title" type="text" :placeholder="t('dialog.link.titlePh')" @keyup.enter="save" />
        </div>
        <div class="field">
          <label>{{ t("dialog.link.url") }}</label>
          <input v-model="url" type="text" placeholder="https://…" @keyup.enter="save" />
        </div>
        <div class="field">
          <label>{{ t("dialog.link.type") }}</label>
          <select v-model="linkType">
            <option v-for="value in LINK_TYPE_VALUES" :key="value" :value="value">
              {{ linkTypeLabel(settings.locale, value) }}
            </option>
          </select>
        </div>
      </div>

      <div class="modal-actions">
        <button class="btn" @click="emit('close')">{{ t("common.cancel") }}</button>
        <button class="btn primary" :disabled="saving" @click="save">{{ saving ? t("common.save") + "…" : t("common.save") }}</button>
      </div>
    </div>
  </div>
</template>
