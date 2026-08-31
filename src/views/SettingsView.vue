<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir } from "@tauri-apps/api/path";
import { getVersion } from "@tauri-apps/api/app";
import { useSettingsStore, type Gender } from "../stores/settings";
import { useI18n } from "../i18n";
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
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1>{{ t("settings.title") }}</h1>
    </div>

    <SettingsSection title="settings.profile" desc="settings.profileDesc">
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

    <SettingsSection title="settings.appearance" desc="settings.appearanceDesc">
      <RadioGroup
        :model-value="settingsStore.theme"
        :options="themeOptions"
        @update:model-value="settingsStore.theme = $event as typeof settingsStore.theme"
      />
    </SettingsSection>

    <SettingsSection title="settings.language" desc="settings.languageDesc">
      <RadioGroup
        :model-value="settingsStore.locale"
        :options="localeOptions"
        @update:model-value="settingsStore.locale = $event as typeof settingsStore.locale"
      />
    </SettingsSection>

    <SettingsSection title="settings.library">
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

    <SettingsSection title="settings.behavior">
      <SettingsRow label="settings.confirmRemove">
        <ToggleSwitch
          :model-value="settingsStore.confirmRemove"
          @update:model-value="settingsStore.confirmRemove = $event"
        />
      </SettingsRow>
    </SettingsSection>

    <SettingsSection title="settings.github" desc="settings.githubDesc">
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

    <SettingsSection title="settings.about">
      <p class="desc" style="margin-bottom: 0">
        {{ t("settings.aboutText", { version: appVersion }) }}<br />
        {{ t("settings.dataDir") }}<code class="caption">{{ dataDir }}</code><br />
        {{ t("settings.shortcut") }}
      </p>
    </SettingsSection>
  </div>
</template>

<style scoped>
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
</style>
