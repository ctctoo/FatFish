<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir } from "@tauri-apps/api/path";
import { getVersion } from "@tauri-apps/api/app";
import {
  User,
  Palette,
  Languages,
  FolderOpen,
  Settings2,
  Github,
  Info,
} from "lucide-vue-next";
import { useSettingsStore, type Gender } from "../stores/settings";
import { useI18n } from "../i18n";
import { tauriApi } from "../services/tauri";
import type { UpdateInfo } from "../types";
import SettingsSection from "../components/settings/SettingsSection.vue";
import SettingsRow from "../components/settings/SettingsRow.vue";
import RadioGroup from "../components/settings/RadioGroup.vue";
import ToggleSwitch from "../components/settings/ToggleSwitch.vue";

const settingsStore = useSettingsStore();
const { t } = useI18n();
const dataDir = ref(t("misc.loading"));
const appVersion = ref("");

appDataDir()
  .then((dir) => (dataDir.value = dir))
  .catch(() => (dataDir.value = t("misc.cannotGet")));

getVersion()
  .then((v) => (appVersion.value = v))
  .catch(() => (appVersion.value = ""));

// ---- 检查更新 ----
type UpdateState = "idle" | "checking" | "latest" | "available" | "error";
const updateState = ref<UpdateState>("idle");
const updateInfo = ref<UpdateInfo | null>(null);
const updateError = ref("");

async function checkForUpdate() {
  if (!appVersion.value || updateState.value === "checking") return;
  updateState.value = "checking";
  updateError.value = "";
  try {
    updateInfo.value = await tauriApi.checkForUpdate(appVersion.value);
    updateState.value = updateInfo.value ? "available" : "latest";
  } catch (e) {
    updateError.value = String(e);
    updateState.value = "error";
  }
}

async function pickDefaultFolder() {
  const dir = await open({ directory: true, multiple: false, title: t("settings.defaultFolder") });
  if (typeof dir === "string") settingsStore.defaultFolder = dir;
}

// 个人资料：初始引导收集，这里可随时修改
function updateProfile(patch: { name?: string; gender?: Gender; occupation?: string }) {
  settingsStore.profile = {
    name: "",
    gender: "unspecified",
    occupation: "",
    ...(settingsStore.profile ?? {}),
    ...patch,
  };
}

const GENDERS: Gender[] = ["male", "female", "unspecified"];

// 选项数组用 computed，保证切换语言时标签实时刷新
const themeOptions = computed(() => [
  { value: "system", label: t("settings.themeSystem") },
  { value: "light", label: t("settings.themeLight") },
  { value: "dark", label: t("settings.themeDark") },
]);

const localeOptions = computed(() => [
  { value: "zh", label: t("settings.langZh") },
  { value: "en", label: t("settings.langEn") },
]);

const genderOptions = computed(() =>
  GENDERS.map((g) => ({ value: g, label: t(`gender.${g}`) }))
);

// ---- 左侧分类导航 ----
const sections = [
  { id: "profile", title: "settings.profile", icon: User },
  { id: "appearance", title: "settings.appearance", icon: Palette },
  { id: "language", title: "settings.language", icon: Languages },
  { id: "library", title: "settings.library", icon: FolderOpen },
  { id: "behavior", title: "settings.behavior", icon: Settings2 },
  { id: "github", title: "settings.github", icon: Github },
  { id: "about", title: "settings.about", icon: Info },
] as const;

const activeSection = ref<string>(sections[0].id);
let scrollContainer: HTMLElement | null = null;

function targetId(id: string) {
  return `settings-${id}`;
}

function scrollToSection(id: string) {
  document.getElementById(targetId(id))?.scrollIntoView({ behavior: "smooth", block: "start" });
}

function updateActive() {
  if (!scrollContainer) return;
  const containerRect = scrollContainer.getBoundingClientRect();
  const offset = 96;
  let current: string = sections[0].id;
  for (const s of sections) {
    const el = document.getElementById(targetId(s.id));
    if (!el) continue;
    const relTop = el.getBoundingClientRect().top - containerRect.top;
    if (relTop <= offset) current = s.id;
  }
  activeSection.value = current;
}

function onScroll() {
  updateActive();
}

onMounted(() => {
  scrollContainer = document.querySelector(".app-content");
  scrollContainer?.addEventListener("scroll", onScroll, { passive: true });
  updateActive();
});

onUnmounted(() => {
  scrollContainer?.removeEventListener("scroll", onScroll);
});
</script>

<template>
  <div class="page settings-page page-stagger">
    <div class="page-header">
      <h1>{{ t("settings.title") }}</h1>
    </div>

    <div class="settings-layout">
      <nav class="settings-nav">
        <button
          v-for="s in sections"
          :key="s.id"
          class="settings-nav-item"
          :class="{ active: activeSection === s.id }"
          @click="scrollToSection(s.id)"
        >
          <component :is="s.icon" :size="16" :stroke-width="1.8" />
          <span>{{ t(s.title) }}</span>
        </button>
      </nav>

      <div class="settings-content">
        <SettingsSection :id="targetId('profile')" title="settings.profile" desc="settings.profileDesc">
          <div class="profile-form">
            <label class="profile-field">
              <span>{{ t("settings.profileName") }}</span>
              <input
                :value="settingsStore.profile?.name ?? ''"
                :placeholder="t('onboarding.namePlaceholder')"
                maxlength="30"
                @input="updateProfile({ name: ($event.target as HTMLInputElement).value })"
              />
            </label>
            <div class="profile-field">
              <span>{{ t("settings.profileGender") }}</span>
              <RadioGroup
                :model-value="settingsStore.profile?.gender ?? 'unspecified'"
                :options="genderOptions"
                @update:model-value="updateProfile({ gender: $event as Gender })"
              />
            </div>
            <label class="profile-field">
              <span>{{ t("settings.profileOccupation") }}</span>
              <input
                :value="settingsStore.profile?.occupation ?? ''"
                :placeholder="t('onboarding.workPlaceholder')"
                maxlength="30"
                @input="updateProfile({ occupation: ($event.target as HTMLInputElement).value })"
              />
            </label>
            <button class="btn ghost rerun-btn" @click="settingsStore.onboarded = false">
              {{ t("settings.rerunOnboarding") }}
            </button>
          </div>
        </SettingsSection>

        <SettingsSection :id="targetId('appearance')" title="settings.appearance" desc="settings.appearanceDesc">
          <RadioGroup
            :model-value="settingsStore.theme"
            :options="themeOptions"
            @update:model-value="settingsStore.theme = $event as typeof settingsStore.theme"
          />
        </SettingsSection>

        <SettingsSection :id="targetId('language')" title="settings.language" desc="settings.languageDesc">
          <RadioGroup
            :model-value="settingsStore.locale"
            :options="localeOptions"
            @update:model-value="settingsStore.locale = $event as typeof settingsStore.locale"
          />
        </SettingsSection>

        <SettingsSection :id="targetId('library')" title="settings.library">
          <SettingsRow label="settings.defaultFolder">
            <code
              class="caption"
              style="max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap"
            >
              {{ settingsStore.defaultFolder || t("settings.defaultFolderUnset") }}
            </code>
            <button class="btn small" @click="pickDefaultFolder">{{ t("common.browse") }}</button>
          </SettingsRow>
        </SettingsSection>

        <SettingsSection :id="targetId('behavior')" title="settings.behavior">
          <SettingsRow label="settings.confirmRemove">
            <ToggleSwitch
              :model-value="settingsStore.confirmRemove"
              @update:model-value="settingsStore.confirmRemove = $event"
            />
          </SettingsRow>
        </SettingsSection>

        <SettingsSection :id="targetId('github')" title="settings.github" desc="settings.githubDesc">
          <div class="profile-field" style="max-width: 420px">
            <span>
              {{ t("settings.githubClientId") }}
              <em class="muted">（{{ t("settings.githubOptional") }}）</em>
            </span>
            <input
              :value="settingsStore.githubClientId"
              :placeholder="t('settings.githubClientIdPh')"
              @input="settingsStore.githubClientId = ($event.target as HTMLInputElement).value"
            />
          </div>
          <div class="gh-actions-row">
            <button class="btn ghost" @click="openUrl('https://github.com/settings/applications/new')">
              {{ t("settings.githubCreateApp") }}
            </button>
            <button
              class="btn ghost"
              @click="
                openUrl(
                  'https://docs.github.com/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow'
                )
              "
            >
              {{ t("settings.githubDoc") }}
            </button>
          </div>
        </SettingsSection>

        <SettingsSection :id="targetId('about')" title="settings.about">
          <p class="desc" style="margin-bottom: 0">
            {{ t("settings.aboutText", { version: appVersion }) }}<br />
            {{ t("settings.dataDir") }}<code class="caption">{{ dataDir }}</code><br />
            {{ t("settings.shortcut") }}
          </p>
          <div class="update-row">
            <button
              class="btn small"
              :disabled="updateState === 'checking' || !appVersion"
              @click="checkForUpdate"
            >
              {{ updateState === "checking" ? t("settings.checkingUpdate") : t("settings.checkUpdate") }}
            </button>
            <span v-if="updateState === 'latest'" class="update-hint">
              {{ t("settings.upToDate") }}
            </span>
            <span v-else-if="updateState === 'available' && updateInfo" class="update-hint">
              {{ t("settings.updateAvailable", { version: updateInfo.latestVersion }) }}
              <button class="update-link" @click="openUrl(updateInfo.releaseUrl)">
                {{ t("settings.downloadUpdate") }}
              </button>
            </span>
            <span v-else-if="updateState === 'error'" class="update-hint update-error">
              {{ t("settings.updateFailed") }}：{{ updateError }}
            </span>
          </div>
        </SettingsSection>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-layout {
  display: flex;
  align-items: flex-start;
  gap: 24px;
  margin-top: 8px;
}

.settings-nav {
  position: sticky;
  top: 0;
  flex-shrink: 0;
  width: 176px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 2px 0;
}

.settings-nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: none;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 13.5px;
  text-align: left;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.settings-nav-item:hover {
  background: var(--hover);
  color: var(--text-primary);
}

.settings-nav-item.active {
  background: var(--accent-soft);
  color: var(--accent);
  font-weight: 600;
}

.settings-content {
  flex: 1;
  min-width: 0;
}

.settings-content :deep(.settings-section) {
  scroll-margin-top: 8px;
}

@media (max-width: 720px) {
  .settings-layout {
    flex-direction: column;
    gap: 12px;
  }

  .settings-nav {
    position: static;
    width: 100%;
    flex-direction: row;
    flex-wrap: wrap;
    gap: 4px;
  }

  .settings-nav-item {
    width: auto;
  }
}

.gh-actions-row {
  display: flex;
  gap: 8px;
  margin-top: 10px;
  flex-wrap: wrap;
}

.muted {
  font-style: normal;
  font-weight: 400;
  color: var(--text-tertiary);
  font-size: 12px;
}

.update-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  margin-top: 12px;
}

.update-hint {
  font-size: 12.5px;
  color: var(--text-secondary);
}

.update-link {
  border: none;
  background: none;
  padding: 0 0 0 4px;
  color: var(--accent);
  font-size: 12.5px;
  cursor: pointer;
  text-decoration: underline;
}

.update-error {
  color: var(--danger, #d9534f);
}
</style>
