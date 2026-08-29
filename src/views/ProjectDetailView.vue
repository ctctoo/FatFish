<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { openUrl } from "@tauri-apps/plugin-opener";
import { marked } from "marked";
import {
  ChevronLeft,
  MoreHorizontal,
  Star,
  Pencil,
  Trash2,
  FolderOpen,
  TerminalSquare,
  Copy,
  ExternalLink,
  RefreshCw,
  Plus,
  Link2,
  Globe,
} from "lucide-vue-next";
import ProjectCover from "../components/project/ProjectCover.vue";
import ProjectDialog from "../components/dialog/ProjectDialog.vue";
import LinkDialog from "../components/dialog/LinkDialog.vue";
import ConfirmDialog from "../components/common/ConfirmDialog.vue";
import { tauriApi } from "../services/tauri";
import { useProjectStore } from "../stores/project";
import { useSettingsStore } from "../stores/settings";
import { useUiStore } from "../stores/ui";
import { statusLabel, linkTypeLabel, relativeTime } from "../types";
import type { Project } from "../types";

const props = defineProps<{ id: string }>();

const router = useRouter();
const projectStore = useProjectStore();
const settingsStore = useSettingsStore();
const uiStore = useUiStore();

const project = ref<Project | null>(null);
const showForm = ref(false);
const linkDialogMode = ref<"add" | "edit">("add");
const editingLink = ref<Project["links"][number] | null>(null);
const showLinkDialog = ref(false);
const deleteTarget = ref(false);
const showMenu = ref(false);
const menuX = ref(0);
const menuY = ref(0);

const renderedDescription = computed(() =>
  project.value?.description ? marked.parse(project.value.description) : ""
);

const githubLink = computed(() => project.value?.links.find((l) => l.linkType === "github"));

onMounted(load);
watch(() => props.id, load);

async function load() {
  try {
    project.value = await tauriApi.getProject(props.id);
  } catch (e) {
    uiStore.showToast(String(e), "error");
    router.push("/projects");
  }
}

async function toggleFavorite() {
  if (!project.value) return;
  await tauriApi.setFavorite(project.value.id, !project.value.favorite);
  await load();
}

async function refreshGit() {
  if (!project.value) return;
  project.value = await tauriApi.refreshGitInfo(project.value.id);
  uiStore.showToast("Git 信息已刷新", "success");
}

async function openFolder() {
  if (!project.value) return;
  try {
    await projectStore.openInFolder(project.value);
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

async function openTerminal() {
  if (!project.value) return;
  try {
    await tauriApi.openTerminal(project.value.path);
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

async function copyPath() {
  if (!project.value) return;
  await navigator.clipboard.writeText(project.value.path);
  uiStore.showToast("路径已复制", "success");
}

function openLink(url: string) {
  openUrl(url).catch((e) => uiStore.showToast(String(e), "error"));
}

function openGitHub() {
  if (githubLink.value) openLink(githubLink.value.url);
}

async function removeLink(linkId: string) {
  await tauriApi.deleteLink(linkId);
  await load();
  uiStore.showToast("链接已删除", "success");
}

let notesTimer: ReturnType<typeof setTimeout> | null = null;

function onNotesInput() {
  if (notesTimer) clearTimeout(notesTimer);
  notesTimer = setTimeout(async () => {
    if (!project.value) return;
    await tauriApi.updateProject(project.value.id, {
      name: project.value.name,
      path: project.value.path,
      description: project.value.description,
      status: project.value.status,
      coverEmoji: project.value.coverEmoji,
      coverColor: project.value.coverColor,
      notes: project.value.notes,
    });
  }, 600);
}

function onMenuAction(action: string) {
  showMenu.value = false;
  const run = async () => {
    switch (action) {
      case "open-folder": await openFolder(); break;
      case "open-terminal": await openTerminal(); break;
      case "copy-path": await copyPath(); break;
      case "refresh-git": await refreshGit(); break;
      case "open-github": openGitHub(); break;
      case "edit": showForm.value = true; break;
      case "add-link":
        linkDialogMode.value = "add";
        editingLink.value = null;
        showLinkDialog.value = true;
        break;
      case "delete":
        if (settingsStore.confirmRemove) deleteTarget.value = true;
        else await remove();
        break;
    }
  };
  run().catch((e) => uiStore.showToast(String(e), "error"));
}

async function remove() {
  if (!project.value) return;
  await projectStore.deleteProject(project.value.id);
  uiStore.showToast("项目已删除", "success");
  router.push("/projects");
}

function openHeaderMenu(e: MouseEvent) {
  menuX.value = Math.min(e.clientX, window.innerWidth - 230);
  menuY.value = Math.min(e.clientY, window.innerHeight - 320);
  showMenu.value = true;
  setTimeout(() => document.addEventListener("click", () => (showMenu.value = false), { once: true }), 0);
}

function formatTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  const d = new Date(iso);
  return isNaN(d.getTime()) ? iso : d.toLocaleString();
}

function shortHash(hash: string | null): string {
  return hash ? hash.slice(0, 8) : "—";
}
</script>

<template>
  <div class="page">
    <div class="detail" v-if="project">
      <button class="back-link" @click="router.push('/projects')">
        <ChevronLeft :size="15" :stroke-width="1.8" /> 返回项目列表
      </button>

      <div style="display: flex; justify-content: flex-end; gap: 6px; margin-bottom: -34px; position: relative">
        <button class="btn ghost small" :title="project.favorite ? '取消收藏' : '收藏'" @click="toggleFavorite">
          <Star
            :size="16"
            :stroke-width="1.8"
            :style="{ color: project.favorite ? '#d9a92f' : undefined, fill: project.favorite ? 'currentColor' : 'none' }"
          />
        </button>
        <button class="btn ghost small" @click="openHeaderMenu">
          <MoreHorizontal :size="16" :stroke-width="1.8" />
        </button>
      </div>

      <div class="detail-hero">
        <ProjectCover :name="project.name" :emoji="project.coverEmoji" :color="project.coverColor" size="detail" />
        <h1>{{ project.name }}</h1>
        <div class="subtitle">{{ project.description?.split("\n")[0] || "暂无描述" }}</div>
        <div class="status-chip" style="justify-content: center">
          <span class="status-dot" :class="`status-${project.status}`"></span>
          {{ statusLabel(project.status) }}
          <template v-if="project.language"> · {{ project.language }}</template>
        </div>
        <div class="chip-row" style="justify-content: center; margin-top: 10px">
          <span v-for="c in project.collections" :key="c.id" class="collection-badge">{{ c.name }}</span>
          <span v-for="t in project.tags" :key="t.id" class="tag-badge">{{ t.name }}</span>
        </div>
      </div>

      <div class="detail-section">
        <h3>Location</h3>
        <div class="kv-row">
          <FolderOpen :size="15" :stroke-width="1.8" style="color: var(--text-tertiary)" />
          <span class="v"><code>{{ project.path }}</code></span>
          <button class="link-btn" @click="openFolder">打开</button>
          <button class="link-btn" @click="copyPath">复制</button>
          <button class="link-btn" @click="openTerminal">终端</button>
        </div>
      </div>

      <div class="detail-section">
        <h3 style="display: flex; align-items: center">
          Links
          <button
            class="link-btn"
            style="margin-left: auto; text-transform: none; letter-spacing: 0"
            @click="linkDialogMode = 'add'; editingLink = null; showLinkDialog = true"
          >
            <Plus :size="13" :stroke-width="1.8" /> 添加链接
          </button>
        </h3>
        <div v-if="project.links.length">
          <div v-for="link in project.links" :key="link.id" class="link-row">
            <Globe :size="14" :stroke-width="1.8" style="color: var(--text-tertiary); flex-shrink: 0" />
            <span class="link-title">{{ link.title }}</span>
            <span class="caption">{{ linkTypeLabel(link.linkType) }}</span>
            <span class="link-url">{{ link.url }}</span>
            <span class="spacer" style="flex: 1"></span>
            <button class="link-btn" @click="openLink(link.url)">
              <ExternalLink :size="13" :stroke-width="1.8" />
            </button>
            <button
              class="link-btn"
              @click="linkDialogMode = 'edit'; editingLink = link; showLinkDialog = true"
            >
              <Pencil :size="13" :stroke-width="1.8" />
            </button>
            <button class="link-btn" @click="removeLink(link.id)">
              <Trash2 :size="13" :stroke-width="1.8" />
            </button>
          </div>
        </div>
        <p v-else class="caption">还没有链接。GitHub 地址会在刷新 Git 信息时自动识别。</p>
      </div>

      <div class="detail-section" v-if="project.description">
        <h3>Description</h3>
        <div class="md" v-html="renderedDescription"></div>
      </div>

      <div class="detail-section">
        <h3>Notes</h3>
        <textarea
          v-model="project.notes"
          class="notes-editor"
          placeholder="记录下一步计划、想法…（自动保存，支持 Markdown）"
          @input="onNotesInput"
        ></textarea>
      </div>

      <div class="detail-section">
        <h3 style="display: flex; align-items: center">
          开发者信息（可选模块）
          <button class="link-btn" style="margin-left: auto; text-transform: none; letter-spacing: 0" @click="refreshGit">
            <RefreshCw :size="12" :stroke-width="1.8" /> 刷新
          </button>
        </h3>
        <template v-if="project.gitInfo && (project.gitInfo.branch || project.gitInfo.remoteUrl)">
          <div class="kv-row">
            <span class="k">Branch</span>
            <span class="v"><code>{{ project.gitInfo.branch ?? "—" }}</code></span>
          </div>
          <div class="kv-row">
            <span class="k">状态</span>
            <span class="v">
              <span :style="{ color: project.gitInfo.isDirty ? 'var(--status-paused)' : 'var(--status-in-progress)' }">
                {{ project.gitInfo.isDirty == null ? "未知" : project.gitInfo.isDirty ? "有未提交更改" : "Clean" }}
              </span>
            </span>
          </div>
          <div class="kv-row">
            <span class="k">最后 Commit</span>
            <span class="v"><code>{{ shortHash(project.gitInfo.commitHash) }}</code> {{ project.gitInfo.commitMessage ?? "" }}</span>
          </div>
          <div class="kv-row">
            <span class="k">时间</span>
            <span class="v">{{ formatTime(project.gitInfo.commitTime) }} · Updated {{ relativeTime(project.updatedAt) }}</span>
          </div>
        </template>
        <p v-else class="caption">Not a Git repository</p>
      </div>

      <ProjectDialog v-if="showForm" :project="project" @close="showForm = false" @saved="load()" />
      <LinkDialog
        v-if="showLinkDialog"
        :project-id="project.id"
        :link="editingLink"
        @close="showLinkDialog = false"
        @saved="load()"
      />
      <ConfirmDialog
        v-if="deleteTarget"
        title="删除项目"
        :message="`确定从索引中移除「${project.name}」吗？磁盘上的文件夹不会被删除。`"
        confirm-text="删除"
        danger
        @confirm="remove()"
        @cancel="deleteTarget = false"
      />

      <Teleport to="body">
        <div v-if="showMenu" class="menu" :style="{ left: menuX + 'px', top: menuY + 'px' }" @click.stop>
          <button class="menu-item" @click="onMenuAction('open-folder')"><FolderOpen :size="15" :stroke-width="1.8" /> 打开文件夹</button>
          <button class="menu-item" @click="onMenuAction('open-terminal')"><TerminalSquare :size="15" :stroke-width="1.8" /> 打开终端</button>
          <button class="menu-item" :disabled="!githubLink" @click="onMenuAction('open-github')"><ExternalLink :size="15" :stroke-width="1.8" /> 打开 GitHub</button>
          <button class="menu-item" @click="onMenuAction('copy-path')"><Copy :size="15" :stroke-width="1.8" /> 复制路径</button>
          <div class="menu-divider"></div>
          <button class="menu-item" @click="onMenuAction('edit')"><Pencil :size="15" :stroke-width="1.8" /> 编辑</button>
          <button class="menu-item" @click="onMenuAction('add-link')"><Link2 :size="15" :stroke-width="1.8" /> 添加链接</button>
          <button class="menu-item" @click="onMenuAction('refresh-git')"><RefreshCw :size="15" :stroke-width="1.8" /> 刷新 Git 信息</button>
          <div class="menu-divider"></div>
          <button class="menu-item danger" @click="onMenuAction('delete')"><Trash2 :size="15" :stroke-width="1.8" /> 删除项目</button>
        </div>
      </Teleport>
    </div>
  </div>
</template>
