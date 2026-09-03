<script setup lang="ts">
import { computed, onUnmounted } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { X, CheckCircle2, XCircle, Sparkles, Loader2, FileUp, Rocket } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import { useUiStore } from "../../stores/ui";
import { createReleaseWizard } from "../../stores/release";

const props = defineProps<{ projectId: string }>();
const emit = defineEmits<{
  close: [];
  published: [];
}>();

const { t } = useI18n();
const uiStore = useUiStore();
const wiz = createReleaseWizard();

wiz.open(props.projectId);

onUnmounted(() => wiz.close());

const steps = computed(() => [
  t("release.step.version"),
  t("release.step.changelog"),
  t("release.step.assets"),
  t("release.step.publish"),
]);

const outcomeOk = computed(() => wiz.outcome?.success === true);
const outcomeFailed = computed(() => wiz.outcome && wiz.outcome.success === false);

function formatSize(bytes: number | null): string {
  if (bytes == null) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function openReleaseUrl(url: string | null) {
  if (url) openUrl(url).catch((e) => uiStore.showToast(String(e), "error"));
}

function next() {
  if (wiz.step === 0 && !wiz.version.trim()) return;
  wiz.step = Math.min(3, wiz.step + 1) as 0 | 1 | 2 | 3;
}

function back() {
  wiz.step = Math.max(0, wiz.step - 1) as 0 | 1 | 2 | 3;
}

function startPublish() {
  wiz.start(props.projectId);
}

function onPublishedClose() {
  emit("published");
  emit("close");
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal release-modal">
      <div class="modal-header">
        <h2>{{ t("release.wizardTitle") }}</h2>
        <button class="modal-close" @click="emit('close')">
          <X :size="17" :stroke-width="1.8" />
        </button>
      </div>

      <!-- 步骤条 -->
      <div class="release-steps">
        <template v-for="(label, i) in steps" :key="label">
          <span v-if="i > 0" class="release-step-line" :class="{ done: wiz.step > (i - 1) }"></span>
          <span class="release-step" :class="{ active: wiz.step === i, done: wiz.step > i }">
            {{ label }}
          </span>
        </template>
      </div>

      <!-- 加载中 -->
      <div v-if="wiz.loadingContext" class="release-loading">
        <Loader2 :size="20" class="spin" /> {{ t("misc.loading") }}
      </div>

      <!-- 步骤 0：版本确认 -->
      <div v-else-if="wiz.step === 0" class="release-body">
        <h3 class="release-sub">{{ t("release.checkTitle") }}</h3>
        <ul class="release-checks">
          <li v-for="check in wiz.context?.checks ?? []" :key="check.key" :class="check.passed ? 'pass' : 'fail'">
            <CheckCircle2 v-if="check.passed" :size="15" :stroke-width="1.8" />
            <XCircle v-else :size="15" :stroke-width="1.8" />
            <span>{{ t(`release.check.${check.key}`) }}</span>
            <em>{{ check.message }}</em>
          </li>
        </ul>
        <p v-if="wiz.context?.isDirty" class="release-hint">{{ t("release.dirtyHint") }}</p>

        <div class="form-grid" style="margin-top: 14px">
          <div class="field">
            <label>{{ t("release.versionLabel") }}</label>
            <input v-model="wiz.version" type="text" placeholder="1.2.0" />
          </div>
          <div class="field">
            <label>{{ t("release.tagLabel") }}</label>
            <input v-model="wiz.tagName" type="text" placeholder="v1.2.0" />
          </div>
        </div>
        <label class="release-checkline" v-if="wiz.context?.hasVersionFiles">
          <input type="checkbox" v-model="wiz.syncVersion" />
          {{ t("release.syncVersion") }}
        </label>
        <p v-else class="caption">{{ t("release.noVersionFiles") }}</p>
      </div>

      <!-- 步骤 1：Changelog -->
      <div v-else-if="wiz.step === 1" class="release-body">
        <div class="release-changelog-head">
          <label>{{ t("release.changelogLabel") }}</label>
          <button
            class="btn small ghost"
            :disabled="wiz.polishing || !wiz.context"
            :title="wiz.context?.tokenSource === 'none' || !wiz.context ? t('release.aiNeedKey') : undefined"
            @click="wiz.polish()"
          >
            <Sparkles :size="13" :stroke-width="1.8" />
            {{ wiz.polishing ? t("release.aiPolishing") : t("release.aiPolish") }}
          </button>
        </div>
        <textarea v-model="wiz.changelog" class="release-changelog" rows="14" spellcheck="false"></textarea>
        <p class="caption" v-if="wiz.context?.commits.length">
          {{ t("release.commitsSince", { n: wiz.context.commits.length, tag: wiz.context.latestTag ?? "HEAD" }) }}
        </p>
        <p class="caption" v-else>{{ t("release.changelogEmpty") }}</p>
      </div>

      <!-- 步骤 2：产物选择 -->
      <div v-else-if="wiz.step === 2" class="release-body">
        <label>{{ t("release.assetsLabel") }}</label>
        <div class="release-assets">
          <div v-for="asset in wiz.assets" :key="asset" class="release-asset">
            <FileUp :size="14" :stroke-width="1.8" />
            <span class="release-asset-name">{{ asset.split(/[\\/]/).pop() }}</span>
            <button class="link-btn" @click="wiz.removeAsset(asset)">✕</button>
          </div>
          <p v-if="!wiz.assets.length" class="caption">{{ t("release.noAssets") }}</p>
        </div>
        <div class="release-asset-actions">
          <button class="btn small" @click="wiz.pickAssets">{{ t("release.pickAssets") }}</button>
        </div>
        <div class="release-flags">
          <label class="release-checkline">
            <input type="checkbox" v-model="wiz.draft" />
            {{ t("release.draft") }}
          </label>
          <label class="release-checkline">
            <input type="checkbox" v-model="wiz.prerelease" />
            {{ t("release.prerelease") }}
          </label>
        </div>
      </div>

      <!-- 步骤 3：执行 -->
      <div v-else class="release-body">
        <div v-if="wiz.executing" class="release-running">
          <Loader2 :size="22" class="spin" />
          <div class="release-running-info">
            <strong>{{ t("release.publishing") }}</strong>
            <span class="caption">{{ wiz.progress?.message ?? "…" }}</span>
          </div>
        </div>
        <div v-if="wiz.progress?.step === 'assets' && wiz.uploadPercent != null" class="release-progress">
          <div class="release-progress-bar">
            <div class="release-progress-fill" :style="{ width: wiz.uploadPercent + '%' }"></div>
          </div>
          <span class="caption">
            {{ wiz.progress.currentFile }} · {{ wiz.uploadPercent }}%
            ({{ formatSize(wiz.progress.uploadedBytes) }} / {{ formatSize(wiz.progress.totalBytes) }})
          </span>
        </div>

        <div v-if="outcomeOk" class="release-done">
          <CheckCircle2 :size="34" :stroke-width="1.6" style="color: var(--accent)" />
          <strong>{{ t("release.done") }}</strong>
          <button v-if="wiz.outcome?.releaseUrl" class="btn small" @click="openReleaseUrl(wiz.outcome.releaseUrl)">
            {{ t("release.viewOnGithub") }}
          </button>
        </div>

        <div v-if="outcomeFailed" class="release-failed">
          <XCircle :size="30" :stroke-width="1.6" style="color: var(--danger, #d9534f)" />
          <strong>{{ t("release.failed") }}</strong>
          <code class="release-error caption">{{ wiz.outcome?.error }}</code>
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="modal-actions">
        <template v-if="!wiz.executing && !outcomeOk">
          <button
            v-if="wiz.step > 0 && wiz.step < 3"
            class="btn"
            @click="back"
          >
            {{ t("release.back") }}
          </button>
          <button
            v-if="wiz.step < 2"
            class="btn primary"
            :disabled="wiz.loadingContext || (wiz.step === 0 && wiz.blockingChecks.length > 0)"
            @click="next"
          >
            {{ t("release.next") }}
          </button>
          <button
            v-else-if="wiz.step === 2"
            class="btn primary"
            @click="next"
          >
            {{ t("release.next") }}
          </button>
          <button
            v-else
            class="btn primary"
            @click="outcomeFailed ? wiz.retry(props.projectId) : startPublish()"
          >
            <Rocket :size="14" :stroke-width="1.8" />
            {{ outcomeFailed ? t("release.retry") : t("release.start") }}
          </button>
        </template>
        <button v-if="outcomeOk" class="btn primary" @click="onPublishedClose">
          {{ t("release.close") }}
        </button>
        <button v-else-if="!wiz.executing && wiz.step < 3" class="btn ghost" @click="emit('close')">
          {{ t("common.cancel") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.release-modal {
  width: min(640px, 92vw);
  max-height: 86vh;
  display: flex;
  flex-direction: column;
}

.release-steps {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 4px 0 16px;
}

.release-step {
  font-size: 12.5px;
  color: var(--text-tertiary);
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  white-space: nowrap;
}

.release-step.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
  font-weight: 600;
}

.release-step.done {
  color: var(--accent);
  border-color: transparent;
  background: var(--accent-soft);
}

.release-step-line {
  flex: 1;
  height: 1px;
  background: var(--border, rgba(128, 128, 128, 0.25));
}

.release-body {
  overflow-y: auto;
  flex: 1;
  min-height: 180px;
}

.release-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  justify-content: center;
  padding: 60px 0;
  color: var(--text-secondary);
}

.release-sub {
  font-size: 13px;
  margin: 0 0 8px;
}

.release-checks {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.release-checks li {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  padding: 7px 10px;
  border-radius: var(--radius-md);
  background: var(--hover);
}

.release-checks li.pass {
  color: var(--accent);
}

.release-checks li.fail {
  color: var(--danger, #d9534f);
}

.release-checks li em {
  margin-left: auto;
  font-style: normal;
  font-size: 12px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 60%;
}

.release-hint {
  margin: 10px 0 0;
  font-size: 12.5px;
  color: var(--text-secondary);
}

.release-checkline {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  margin-top: 12px;
  cursor: pointer;
}

.release-checkline input {
  accent-color: var(--accent);
  width: 14px;
  height: 14px;
}

.release-changelog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.release-changelog {
  width: 100%;
  font-family: var(--font-mono, monospace);
  font-size: 12.5px;
  line-height: 1.55;
  padding: 10px 12px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.25));
  border-radius: var(--radius-md);
  background: var(--hover);
  color: var(--text-primary);
  resize: vertical;
}

.release-assets {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 6px;
}

.release-asset {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  padding: 7px 10px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  border-radius: var(--radius-md);
}

.release-asset-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.release-asset .link-btn {
  margin-left: auto;
}

.release-asset-actions {
  margin-top: 10px;
}

.release-flags {
  display: flex;
  gap: 20px;
  margin-top: 4px;
}

.release-running {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 24px 0 12px;
}

.release-running-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.release-progress {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 8px 0 16px;
}

.release-progress-bar {
  height: 6px;
  border-radius: 999px;
  background: var(--hover);
  overflow: hidden;
}

.release-progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 999px;
  transition: width 0.25s ease;
}

.release-done,
.release-failed {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 32px 0;
  text-align: center;
}

.release-error {
  max-width: 100%;
  white-space: pre-wrap;
  word-break: break-all;
}

.spin {
  animation: release-spin 1s linear infinite;
}

@keyframes release-spin {
  to { transform: rotate(360deg); }
}

.release-modal .modal-actions {
  flex-wrap: wrap;
}
</style>
