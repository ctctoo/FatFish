<script setup lang="ts">
import { ref } from "vue";
import { FolderOpen, TerminalSquare, Link2, Copy, Pencil, RefreshCw, Trash2, ExternalLink, Check } from "lucide-vue-next";
import { useI18n, statusLabel } from "../../i18n";
import { STATUS_VALUES } from "../../types";
import type { ProjectStatus } from "../../types";
import { useSettingsStore } from "../../stores/settings";

const props = defineProps<{
  hasGitHubLink: boolean;
  status: ProjectStatus;
}>();

const emit = defineEmits<{
  action: [action: string];
}>();

const { t } = useI18n();
const settings = useSettingsStore();

const visible = ref(false);
const x = ref(0);
const y = ref(0);

function open(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  x.value = Math.min(e.clientX, window.innerWidth - 230);
  y.value = Math.min(e.clientY, Math.max(40, window.innerHeight - 520));
  visible.value = true;
  setTimeout(() => document.addEventListener("click", close, { once: true }), 0);
}

function close() {
  visible.value = false;
}

function run(action: string) {
  visible.value = false;
  emit("action", action);
}

defineExpose({ open });
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="menu-backdrop"></div>
    <div v-if="visible" class="menu" :style="{ left: x + 'px', top: y + 'px' }" @click.stop>
      <button class="menu-item" @click="run('open')">
        <ExternalLink :size="15" :stroke-width="1.8" /> {{ t("common.open") }}
      </button>
      <button class="menu-item" @click="run('open-folder')">
        <FolderOpen :size="15" :stroke-width="1.8" /> {{ t("menu.showInExplorer") }}
      </button>
      <button class="menu-item" @click="run('open-terminal')">
        <TerminalSquare :size="15" :stroke-width="1.8" /> {{ t("menu.openTerminal") }}
      </button>
      <button class="menu-item" @click="run('copy-path')">
        <Copy :size="15" :stroke-width="1.8" /> {{ t("menu.copyPath") }}
      </button>
      <div class="menu-divider"></div>
      <button class="menu-item" @click="run('edit')">
        <Pencil :size="15" :stroke-width="1.8" /> {{ t("menu.edit") }}
      </button>
      <button class="menu-item" @click="run('add-link')">
        <Link2 :size="15" :stroke-width="1.8" /> {{ t("menu.addLink") }}
      </button>
      <button class="menu-item" @click="run('refresh-git')">
        <RefreshCw :size="15" :stroke-width="1.8" /> {{ t("menu.refreshGit") }}
      </button>
      <button class="menu-item" :disabled="!props.hasGitHubLink" @click="run('open-github')">
        <ExternalLink :size="15" :stroke-width="1.8" /> {{ t("menu.openGithub") }}
      </button>
      <div class="menu-divider"></div>
      <div class="menu-heading">{{ t("menu.status") }}</div>
      <button
        v-for="value in STATUS_VALUES"
        :key="value"
        class="menu-item status-option"
        :class="{ current: value === props.status }"
        @click="run(`status:${value}`)"
      >
        <span class="status-dot" :class="`status-${value}`"></span>
        {{ statusLabel(settings.locale, value) }}
        <Check v-if="value === props.status" :size="14" :stroke-width="2" class="status-check" />
      </button>
      <div class="menu-divider"></div>
      <button class="menu-item danger" @click="run('delete')">
        <Trash2 :size="15" :stroke-width="1.8" /> {{ t("menu.delete") }}
      </button>
    </div>
  </Teleport>
</template>

<style scoped>
.menu-heading {
  padding: 6px 12px 2px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.status-option .status-dot {
  flex: none;
  margin-right: 2px;
}
.status-option .status-check {
  margin-left: auto;
  color: var(--accent);
}
</style>
