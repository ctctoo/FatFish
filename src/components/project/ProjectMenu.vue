<script setup lang="ts">
import { ref } from "vue";
import { FolderOpen, TerminalSquare, Link2, Copy, Pencil, RefreshCw, Trash2, ExternalLink } from "lucide-vue-next";
import { useI18n } from "../../i18n";

const props = defineProps<{
  hasGitHubLink: boolean;
}>();

const emit = defineEmits<{
  action: [action: string];
}>();

const { t } = useI18n();

const visible = ref(false);
const x = ref(0);
const y = ref(0);

function open(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  x.value = Math.min(e.clientX, window.innerWidth - 230);
  y.value = Math.min(e.clientY, window.innerHeight - 320);
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
      <button class="menu-item danger" @click="run('delete')">
        <Trash2 :size="15" :stroke-width="1.8" /> {{ t("menu.delete") }}
      </button>
    </div>
  </Teleport>
</template>
