import { defineStore } from "pinia";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { tauriApi } from "../services/tauri";
import { useUiStore } from "./ui";
import type { Release, ReleaseContext, ReleaseOutcome, ReleaseProgress } from "../types";

const EVENT_PROGRESS = "release://progress";
const EVENT_OUTCOME = "release://outcome";

export const useReleaseStore = defineStore("release", () => {
  const uiStore = useUiStore();

  const releases = ref<Release[]>([]);
  const loadingHistory = ref(false);

  async function loadHistory(projectId: string) {
    loadingHistory.value = true;
    try {
      releases.value = await tauriApi.listReleases(projectId);
    } catch (e) {
      uiStore.showToast(String(e), "error");
    } finally {
      loadingHistory.value = false;
    }
  }

  return { releases, loadingHistory, loadHistory };
});

// ---- 向导状态（独立于全局 store，向导关闭即销毁） ----
import { computed, reactive, ref } from "vue";

export function createReleaseWizard() {
  const uiStore = useUiStore();

  const step = ref<0 | 1 | 2 | 3>(0);
  const loadingContext = ref(false);
  const context = ref<ReleaseContext | null>(null);

  const version = ref("");
  const tagName = ref("");
  const syncVersion = ref(false);
  const changelog = ref("");
  const assets = ref<string[]>([]);
  const draft = ref(false);
  const prerelease = ref(false);

  const polishing = ref(false);
  const executing = ref(false);
  const progress = ref<ReleaseProgress | null>(null);
  const outcome = ref<ReleaseOutcome | null>(null);
  const failedAt = ref<string | null>(null);

  let unlisteners: UnlistenFn[] = [];

  const allChecksPassed = computed(() =>
    context.value ? context.value.checks.every((c) => c.passed) : false
  );

  const blockingChecks = computed(() =>
    context.value ? context.value.checks.filter((c) => !c.passed) : []
  );

  const uploadPercent = computed(() => {
    const p = progress.value;
    if (!p?.totalBytes || p.uploadedBytes == null) return null;
    return Math.min(100, Math.round((p.uploadedBytes / p.totalBytes) * 100));
  });

  async function open(projectId: string) {
    step.value = 0;
    context.value = null;
    outcome.value = null;
    failedAt.value = null;
    progress.value = null;
    executing.value = false;
    assets.value = [];
    draft.value = false;
    prerelease.value = false;
    loadingContext.value = true;

    unlisteners.push(await listen<ReleaseProgress>(EVENT_PROGRESS, (e) => (progress.value = e.payload)));
    unlisteners.push(await listen<ReleaseOutcome>(EVENT_OUTCOME, (e) => {
      outcome.value = e.payload;
      executing.value = false;
      if (!e.payload.success) failedAt.value = e.payload.step;
    }));

    try {
      context.value = await tauriApi.getReleaseContext(projectId);
      version.value = context.value.suggestedVersion;
      tagName.value = context.value.suggestedTag;
      syncVersion.value = false;
      changelog.value = context.value.draftChangelog.replace(/^## NEW\n*/i, "");
    } catch (e) {
      uiStore.showToast(String(e), "error");
    } finally {
      loadingContext.value = false;
    }
  }

  function close() {
    for (const un of unlisteners) un();
    unlisteners = [];
  }

  async function polish() {
    if (polishing.value) return;
    polishing.value = true;
    try {
      changelog.value = await tauriApi.polishChangelog(changelog.value);
    } catch (e) {
      uiStore.showToast(String(e), "error");
    } finally {
      polishing.value = false;
    }
  }

  async function pickAssets() {
    const selected = await openDialog({
      multiple: true,
      title: "Release Assets",
    });
    if (Array.isArray(selected)) {
      for (const p of selected) {
        if (!assets.value.includes(p)) assets.value.push(p);
      }
    } else if (typeof selected === "string" && !assets.value.includes(selected)) {
      assets.value.push(selected);
    }
  }

  function removeAsset(path: string) {
    assets.value = assets.value.filter((a) => a !== path);
  }

  function validateVersion(): boolean {
    if (!version.value.trim() || !tagName.value.trim()) {
      uiStore.showToast("version / tag is required", "error");
      return false;
    }
    return true;
  }

  async function start(projectId: string) {
    if (executing.value || !validateVersion()) return;
    executing.value = true;
    outcome.value = null;
    failedAt.value = null;
    progress.value = null;
    try {
      await tauriApi.startRelease({
        projectId,
        version: version.value.trim(),
        tagName: tagName.value.trim(),
        changelog: changelog.value,
        assets: assets.value,
        draft: draft.value,
        prerelease: prerelease.value,
        syncVersion: syncVersion.value && context.value?.hasVersionFiles === true,
      });
    } catch (e) {
      executing.value = false;
      uiStore.showToast(String(e), "error");
    }
  }

  async function retry(projectId: string) {
    await start(projectId);
  }

  return reactive({
    step,
    loadingContext,
    context,
    version,
    tagName,
    syncVersion,
    changelog,
    assets,
    draft,
    prerelease,
    polishing,
    executing,
    progress,
    outcome,
    failedAt,
    allChecksPassed,
    blockingChecks,
    uploadPercent,
    open,
    close,
    polish,
    pickAssets,
    removeAsset,
    start,
    retry,
  });
}
