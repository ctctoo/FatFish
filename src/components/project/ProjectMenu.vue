<script setup lang="ts">
import { ref } from "vue";
import { FolderOpen, TerminalSquare, Link2, Copy, Pencil, RefreshCw, Trash2, ExternalLink } from "lucide-vue-next";

const props = defineProps<{
  hasGitHubLink: boolean;
}>();

const emit = defineEmits<{
  action: [action: string];
}>();

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
        <ExternalLink :size="15" :stroke-width="1.8" /> 打开项目文件夹
      </button>
      <button class="menu-item" @click="run('open-folder')">
        <FolderOpen :size="15" :stroke-width="1.8" /> 在文件管理器中显示
      </button>
      <button class="menu-item" @click="run('open-terminal')">
        <TerminalSquare :size="15" :stroke-width="1.8" /> 在终端中打开
      </button>
      <button class="menu-item" @click="run('copy-path')">
        <Copy :size="15" :stroke-width="1.8" /> 复制路径
      </button>
      <div class="menu-divider"></div>
      <button class="menu-item" @click="run('edit')">
        <Pencil :size="15" :stroke-width="1.8" /> 编辑
      </button>
      <button class="menu-item" @click="run('add-link')">
        <Link2 :size="15" :stroke-width="1.8" /> 添加链接
      </button>
      <button class="menu-item" @click="run('refresh-git')">
        <RefreshCw :size="15" :stroke-width="1.8" /> 刷新 Git 信息
      </button>
      <button class="menu-item" :disabled="!props.hasGitHubLink" @click="run('open-github')">
        <ExternalLink :size="15" :stroke-width="1.8" /> 打开 GitHub
      </button>
      <div class="menu-divider"></div>
      <button class="menu-item danger" @click="run('delete')">
        <Trash2 :size="15" :stroke-width="1.8" /> 删除项目
      </button>
    </div>
  </Teleport>
</template>
