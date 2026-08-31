<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useRouter } from "vue-router";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Github,
  LogOut,
  RefreshCw,
  ExternalLink,
  Star,
  GitFork,
  Lock,
  LoaderCircle,
  X,
} from "lucide-vue-next";
import { useGithubStore } from "../stores/github";
import { useSettingsStore } from "../stores/settings";
import { useUiStore } from "../stores/ui";
import { relativeTime, useI18n } from "../i18n";
import EmptyState from "../components/common/EmptyState.vue";
import type { GithubDeviceCode, GithubRepo } from "../types";
import { githubLangColor } from "../utils/github";

const router = useRouter();
const githubStore = useGithubStore();
const settingsStore = useSettingsStore();
const uiStore = useUiStore();
const { t, locale } = useI18n();

const device = ref<GithubDeviceCode | null>(null);
const starting = ref(false);
const error = ref("");
const remaining = ref(0);

let pollTimer: number | null = null;
let countdownTimer: number | null = null;
let deadline = 0;

onMounted(async () => {
  await githubStore.fetchStatus();
  if (githubStore.account) await githubStore.fetchRepos();
});

onUnmounted(() => stopPolling());

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
  if (countdownTimer) {
    clearInterval(countdownTimer);
    countdownTimer = null;
  }
}

async function startLogin() {
  const clientId = settingsStore.githubClientId.trim();
  if (!clientId) {
    uiStore.showToast(t("github.needClientId"), "error");
    router.push("/settings");
    return;
  }
  starting.value = true;
  error.value = "";
  try {
    device.value = await githubStore.startLogin(clientId);
    deadline = Date.now() + device.value.expiresIn * 1000;
    remaining.value = device.value.expiresIn;
    openUrl(device.value.verificationUri).catch(() => undefined);
    schedulePoll(device.value.interval);
    countdownTimer = window.setInterval(() => {
      remaining.value = Math.max(0, Math.round((deadline - Date.now()) / 1000));
    }, 1000);
  } catch (e) {
    error.value = String(e);
    uiStore.showToast(String(e), "error");
  } finally {
    starting.value = false;
  }
}

function cancelLogin() {
  stopPolling();
  device.value = null;
  error.value = "";
}

function schedulePoll(interval: number) {
  if (pollTimer) clearInterval(pollTimer);
  pollTimer = window.setInterval(pollOnce, Math.max(1, interval) * 1000);
}

async function pollOnce() {
  const activeDevice = device.value;
  if (!activeDevice) return;
  if (Date.now() > deadline) {
    stopPolling();
    device.value = null;
    uiStore.showToast(t("github.expired"), "error");
    return;
  }
  try {
    const result = await githubStore.pollLogin(
      settingsStore.githubClientId.trim(),
      activeDevice.deviceCode
    );
    if (result.status === "success" && result.account) {
      stopPolling();
      githubStore.setAccount(result.account);
      device.value = null;
      uiStore.showToast(t("github.loginSuccess"), "success");
      await githubStore.fetchRepos();
      return;
    }
    if (result.status === "slow_down") {
      schedulePoll(result.interval ?? activeDevice.interval + 5);
      return;
    }
    if (result.status === "expired" || result.status === "denied") {
      stopPolling();
      device.value = null;
      uiStore.showToast(result.status === "expired" ? t("github.expired") : t("github.denied"), "error");
      return;
    }
    if (result.status === "failed") {
      stopPolling();
      device.value = null;
      const msg = result.message ? `${t("github.loginFailed")}：${result.message}` : t("github.loginFailed");
      uiStore.showToast(msg, "error");
    }
    // pending: 继续轮询
  } catch (e) {
    error.value = String(e);
  }
}

async function refreshRepos() {
  try {
    await githubStore.fetchRepos();
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

async function logout() {
  try {
    await githubStore.logout();
    uiStore.showToast(t("github.logoutSuccess"), "success");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

function openRepo(repo: GithubRepo) {
  openUrl(repo.htmlUrl).catch(() => undefined);
}

</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1>{{ t("github.title") }}</h1>
      <span v-if="githubStore.account" class="count">{{ githubStore.repos.length }} {{ t("github.repos") }}</span>
      <div class="spacer" />
      <button v-if="githubStore.account" class="btn ghost" @click="logout">
        <LogOut :size="15" :stroke-width="1.8" />
        {{ t("github.logout") }}
      </button>
    </div>

    <!-- 未登录 -->
    <div v-if="!githubStore.account && !device" class="gh-card">
      <div class="gh-card-icon"><Github :size="36" :stroke-width="1.5" /></div>
      <h3>{{ t("github.loginTitle") }}</h3>
      <p class="text-secondary">{{ t("github.loginDesc") }}</p>
      <p v-if="!settingsStore.githubClientId.trim()" class="gh-warn">
        {{ t("github.needClientId") }}
        <button class="link-btn" @click="router.push('/settings')">{{ t("github.goSettings") }}</button>
      </p>
      <div class="gh-actions">
        <button class="btn primary" :disabled="starting" @click="startLogin">
          <LoaderCircle v-if="starting" class="spin" :size="15" />
          {{ starting ? t("github.starting") : t("github.login") }}
        </button>
      </div>
    </div>

    <!-- 等待授权 -->
    <div v-else-if="device" class="gh-card">
      <div class="gh-card-icon"><Github :size="36" :stroke-width="1.5" /></div>
      <h3>{{ t("github.waiting") }}</h3>
      <p class="text-secondary">{{ t("github.enterCode") }}</p>
      <div class="gh-user-code mono">{{ device.userCode }}</div>
      <div class="gh-actions">
        <button class="btn primary" @click="openUrl(device.verificationUri).catch(() => undefined)">
          <ExternalLink :size="15" :stroke-width="1.8" />
          {{ t("github.openVerification") }}
        </button>
        <button class="btn ghost" @click="cancelLogin">
          <X :size="15" :stroke-width="1.8" />
          {{ t("github.cancel") }}
        </button>
      </div>
      <p class="caption">
        {{ t("github.verificationUri") }}：{{ device.verificationUri }}
        · {{ t("github.expiresIn", { n: Math.ceil(remaining / 60) }) }}
      </p>
      <p v-if="error" class="error-text">{{ error }}</p>
    </div>

    <!-- 已登录 -->
    <template v-else>
      <div class="gh-account">
        <img v-if="githubStore.account?.user.avatarUrl" :src="githubStore.account.user.avatarUrl" class="gh-avatar" alt="" />
        <Github v-else :size="40" :stroke-width="1.5" />
        <div class="gh-account-info">
          <h3>{{ githubStore.account?.user.name || githubStore.account?.user.login }}</h3>
          <p class="text-secondary">@{{ githubStore.account?.user.login }}</p>
          <p v-if="githubStore.account?.user.bio" class="gh-bio">{{ githubStore.account.user.bio }}</p>
        </div>
      </div>

      <div class="gh-toolbar">
        <h2>{{ t("github.repos") }}</h2>
        <div class="spacer" />
        <button class="btn ghost" :disabled="githubStore.loadingRepos" @click="refreshRepos">
          <RefreshCw :size="14" :stroke-width="1.8" :class="{ spin: githubStore.loadingRepos }" />
          {{ t("common.refresh") }}
        </button>
      </div>

      <div v-if="githubStore.loadingRepos" class="gh-loading">
        <LoaderCircle class="spin" :size="20" />
        <span class="caption">{{ t("github.loading") }}</span>
      </div>

      <EmptyState
        v-else-if="!githubStore.repos.length"
        glyph="⌥"
        :title="t('github.noRepos')"
        :message="t('github.noReposDesc')"
      />

      <div v-else class="gh-repo-grid">
        <button v-for="repo in githubStore.repos" :key="repo.id" class="gh-repo-card" @click="openRepo(repo)">
          <div class="gh-repo-head">
            <span class="gh-repo-name">{{ repo.name }}</span>
            <Lock v-if="repo.private" :size="13" :stroke-width="1.8" class="gh-repo-badge" />
            <GitFork v-if="repo.fork" :size="13" :stroke-width="1.8" class="gh-repo-badge" />
          </div>
          <p class="gh-repo-desc">{{ repo.description || t("github.noDesc") }}</p>
          <div class="gh-repo-meta">
            <span class="gh-lang"><span class="lang-dot" :style="{ background: githubLangColor(repo.language) }" />{{ repo.language || "—" }}</span>
            <span v-if="repo.stargazersCount" class="gh-meta-item"><Star :size="13" :stroke-width="1.8" />{{ repo.stargazersCount }}</span>
            <span v-if="repo.forksCount" class="gh-meta-item"><GitFork :size="13" :stroke-width="1.8" />{{ repo.forksCount }}</span>
            <span class="gh-meta-item gh-updated">{{ t("github.updated") }} {{ relativeTime(locale, repo.updatedAt) }}</span>
          </div>
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.gh-card {
  max-width: 520px;
  margin: 48px auto 0;
  padding: 40px 36px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 10px;
}

.gh-card-icon {
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.gh-card h3 {
  font-size: 17px;
}

.gh-warn {
  font-size: 13px;
  color: var(--danger);
  margin-top: 4px;
}

.gh-actions {
  display: flex;
  gap: 10px;
  margin-top: 10px;
}

.gh-user-code {
  font-size: 26px;
  font-weight: 700;
  letter-spacing: 6px;
  padding: 10px 18px;
  background: var(--surface-muted);
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-md);
  margin: 6px 0 2px;
}

.link-btn {
  border: none;
  background: none;
  color: var(--accent);
  font-size: 13px;
  padding: 0 2px;
}

.gh-account {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0 20px;
}

.gh-avatar {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  border: 1px solid var(--border);
  flex-shrink: 0;
  background: var(--surface-muted);
}

.gh-account-info h3 {
  font-size: 18px;
}

.gh-bio {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 2px;
  max-width: 480px;
}

.gh-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 6px 0 14px;
}

.gh-toolbar h2 {
  font-size: 16px;
}

.gh-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 40px 0;
  justify-content: center;
  color: var(--text-tertiary);
}

.gh-repo-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 14px;
}

.gh-repo-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 14px 16px;
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition: border-color 0.15s, background 0.15s, transform 0.18s ease, box-shadow 0.18s ease;
}

.gh-repo-card:hover {
  border-color: var(--border-strong);
  background: var(--surface-muted);
  transform: translateY(-2px);
  box-shadow: var(--shadow-menu);
}

.gh-repo-head {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.gh-repo-name {
  font-size: 14.5px;
  font-weight: 600;
  color: var(--accent);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.gh-repo-badge {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.gh-repo-desc {
  font-size: 13px;
  color: var(--text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 38px;
}

.gh-repo-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-tertiary);
}

.gh-meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.gh-lang {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.lang-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex-shrink: 0;
}

.gh-updated {
  margin-left: auto;
}

.spin {
  animation: gh-spin 0.9s linear infinite;
}

@keyframes gh-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
