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
import ProjectTimeline from "../components/project/ProjectTimeline.vue";
import ProjectDialog from "../components/dialog/ProjectDialog.vue";
import LinkDialog from "../components/dialog/LinkDialog.vue";
import ConfirmDialog from "../components/common/ConfirmDialog.vue";
import { tauriApi } from "../services/tauri";
import { useProjectStore } from "../stores/project";
import { useSettingsStore } from "../stores/settings";
import { useUiStore } from "../stores/ui";
import { statusLabel, linkTypeLabel, relativeTime, useI18n } from "../i18n";
import type { Activity, Link, Project } from "../types";

const props = defineProps<{ id: string }>();

const router = useRouter();
const projectStore = useProjectStore();
const settingsStore = useSettingsStore();
const uiStore = useUiStore();
const { t } = useI18n();

const project = ref<Project | null>(null);
const activities = ref<Activity[]>([]);
const showForm = ref(false);
const linkDialogMode = ref<"add" | "edit">("add");
const editingLink = ref<Link | null>(null);
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
    activities.value = await tauriApi.listActivities(props.id);
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
  uiStore.showToast(t("toast.gitRefreshed"), "success");
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
  uiStore.showToast(t("toast.copied"), "success");
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
  uiStore.showToast(t("toast.linkDeleted"), "success");
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
  uiStore.showToast(t("toast.projectDeleted"), "success");
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
        <ChevronLeft :size="15" :stroke-width="1.8" /> {{ t("detail.back") }}
      </button>

      <div style="display: flex; justify-content: flex-end; gap: 6px; margin-bottom: -34px; position: relative">
        <button class="btn ghost small" :title="project.favorite ? t('menu.unfavorite') : t('menu.favorite')" @click="toggleFavorite">
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
        <div class="subtitle">{{ project.description?.split("\n")[0] || t("card.noDesc") }}</div>
        <div class="status-chip" style="justify-content: center">
          <span class="status-dot" :class="`status-${project.status}`"></span>
          {{ statusLabel(settingsStore.locale, project.status) }}
          <template v-if="project.language"> · {{ project.language }}</template>
        </div>
        <div class="chip-row" style="justify-content: center; margin-top: 10px">
          <span v-for="c in project.collections" :key="c.id" class="collection-badge">{{ c.name }}</span>
          <span v-for="item in project.tags" :key="item.id" class="tag-badge">{{ item.name }}</span>
        </div>
      </div>

      <div class="detail-section">
        <h3>{{ t("detail.location") }}</h3>
        <div class="kv-row">
          <FolderOpen :size="15" :stroke-width="1.8" style="color: var(--text-tertiary)" />
          <span class="v"><code>{{ project.path }}</code></span>
          <button class="link-btn" @click="openFolder">{{ t("common.open") }}</button>
          <button class="link-btn" @click="copyPath">{{ t("common.copy") }}</button>
          <button class="link-btn" @click="openTerminal">{{ t("common.terminal") }}</button>
        </div>
      </div>

      <div class="detail-section">
        <h3 style="display: flex; align-items: center">
          {{ t("detail.links") }}
          <button
            class="link-btn"
            style="margin-left: auto; text-transform: none; letter-spacing: 0"
            @click="linkDialogMode = 'add'; editingLink = null; showLinkDialog = true"
          >
            <Plus :size="13" :stroke-width="1.8" /> {{ t("detail.linksAdd") }}
          </button>
        </h3>
        <div v-if="project.links.length">
          <div v-for="link in project.links" :key="link.id" class="link-row">
            <Globe :size="14" :stroke-width="1.8" style="color: var(--text-tertiary); flex-shrink: 0" />
            <span class="link-title">{{ link.title }}</span>
            <span class="caption">{{ linkTypeLabel(settingsStore.locale, link.linkType) }}</span>
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
        <p v-else class="caption">{{ t("detail.noGithubYet") }}</p>
      </div>

      <div class="detail-section" v-if="project.description">
        <h3>{{ t("detail.description") }}</h3>
        <div class="md" v-html="renderedDescription"></div>
      </div>

      <div class="detail-section">
        <h3>{{ t("detail.notes") }}</h3>
        <textarea
          v-model="project.notes"
          class="notes-editor"
          :placeholder="t('detail.notesPh')"
          @input="onNotesInput"
        ></textarea>
      </div>

      <div class="detail-section" v-if="activities.length">
        <h3>{{ t("detail.timeline") }}</h3>
        <ProjectTimeline :activities="activities" />
      </div>

      <div class="detail-section">
        <h3 style="display: flex; align-items: center">
          {{ t("detail.devSection") }}
          <button class="link-btn" style="margin-left: auto; text-transform: none; letter-spacing: 0" @click="refreshGit">
            <RefreshCw :size="12" :stroke-width="1.8" /> {{ t("common.refresh") }}
          </button>
        </h3>
        <template v-if="project.gitInfo && (project.gitInfo.branch || project.gitInfo.remoteUrl)">
          <div class="kv-row">
            <span class="k">{{ t("detail.branch") }}</span>
            <span class="v"><code>{{ project.gitInfo.branch ?? "—" }}</code></span>
          </div>
          <div class="kv-row">
            <span class="k">{{ t("detail.dirtyState") }}</span>
            <span class="v">
              <span :style="{ color: project.gitInfo.isDirty ? 'var(--status-paused)' : 'var(--status-in-progress)' }">
                {{ project.gitInfo.isDirty == null ? t("detail.unknown") : project.gitInfo.isDirty ? t("detail.dirty") : t("detail.clean") }}
              </span>
            </span>
          </div>
          <div class="kv-row">
            <span class="k">{{ t("detail.lastCommit") }}</span>
            <span class="v"><code>{{ shortHash(project.gitInfo.commitHash) }}</code> {{ project.gitInfo.commitMessage ?? "" }}</span>
          </div>
          <div class="kv-row">
            <span class="k">{{ t("detail.commitTime") }}</span>
            <span class="v">{{ formatTime(project.gitInfo.commitTime) }} · {{ t("card.updated") }} {{ relativeTime(settingsStore.locale, project.updatedAt) }}</span>
          </div>
        </template>
        <p v-else class="caption">{{ t("detail.notGit") }}</p>
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
        :title="t('confirm.deleteProjectTitle')"
        :message="t('confirm.deleteProjectMsg', { name: project.name })"
        :confirm-text="t('confirm.delete')"
        danger
        @confirm="remove()"
        @cancel="deleteTarget = false"
      />

      <Teleport to="body">
        <div v-if="showMenu" class="menu" :style="{ left: menuX + 'px', top: menuY + 'px' }" @click.stop>
          <button class="menu-item" @click="onMenuAction('open-folder')"><FolderOpen :size="15" :stroke-width="1.8" /> {{ t("menu.openFolder") }}</button>
          <button class="menu-item" @click="onMenuAction('open-terminal')"><TerminalSquare :size="15" :stroke-width="1.8" /> {{ t("menu.openTerminal") }}</button>
          <button class="menu-item" :disabled="!githubLink" @click="onMenuAction('open-github')"><ExternalLink :size="15" :stroke-width="1.8" /> {{ t("menu.openGithub") }}</button>
          <button class="menu-item" @click="onMenuAction('copy-path')"><Copy :size="15" :stroke-width="1.8" /> {{ t("menu.copyPath") }}</button>
          <div class="menu-divider"></div>
          <button class="menu-item" @click="onMenuAction('edit')"><Pencil :size="15" :stroke-width="1.8" /> {{ t("menu.edit") }}</button>
          <button class="menu-item" @click="onMenuAction('add-link')"><Link2 :size="15" :stroke-width="1.8" /> {{ t("menu.addLink") }}</button>
          <button class="menu-item" @click="onMenuAction('refresh-git')"><RefreshCw :size="15" :stroke-width="1.8" /> {{ t("menu.refreshGit") }}</button>
          <div class="menu-divider"></div>
          <button class="menu-item danger" @click="onMenuAction('delete')"><Trash2 :size="15" :stroke-width="1.8" /> {{ t("menu.delete") }}</button>
        </div>
      </Teleport>
    </div>
  </div>
</template>
