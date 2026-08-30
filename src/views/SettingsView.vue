<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir } from "@tauri-apps/api/path";
import { getVersion } from "@tauri-apps/api/app";
import { useSettingsStore } from "../stores/settings";
import { useI18n } from "../i18n";

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
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1>{{ t("settings.title") }}</h1>
    </div>

    <div class="settings-section">
      <h3>{{ t("settings.profile") }}</h3>
      <p class="desc">{{ t("settings.profileDesc") }}</p>
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
          <div class="radio-row">
            <button
              v-for="g in (['male', 'female', 'unspecified'] as const)"
              :key="g"
              class="radio-pill"
              :class="{ active: (settingsStore.profile?.gender ?? 'unspecified') === g }"
              @click="updateProfile({ gender: g })"
            >
              {{ t(`gender.${g}`) }}
            </button>
          </div>
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
    </div>

    <div class="settings-section">
      <h3>{{ t("settings.appearance") }}</h3>
      <p class="desc">{{ t("settings.appearanceDesc") }}</p>
      <div class="radio-row">
        <button
          class="radio-pill"
          :class="{ active: settingsStore.theme === 'system' }"
          @click="settingsStore.theme = 'system'"
        >
          {{ t("settings.themeSystem") }}
        </button>
        <button
          class="radio-pill"
          :class="{ active: settingsStore.theme === 'light' }"
          @click="settingsStore.theme = 'light'"
        >
          {{ t("settings.themeLight") }}
        </button>
        <button
          class="radio-pill"
          :class="{ active: settingsStore.theme === 'dark' }"
          @click="settingsStore.theme = 'dark'"
        >
          {{ t("settings.themeDark") }}
        </button>
      </div>
    </div>

    <div class="settings-section">
      <h3>{{ t("settings.language") }}</h3>
      <p class="desc">{{ t("settings.languageDesc") }}</p>
      <div class="radio-row">
        <button
          class="radio-pill"
          :class="{ active: settingsStore.locale === 'zh' }"
          @click="settingsStore.locale = 'zh'"
        >
          {{ t("settings.langZh") }}
        </button>
        <button
          class="radio-pill"
          :class="{ active: settingsStore.locale === 'en' }"
          @click="settingsStore.locale = 'en'"
        >
          {{ t("settings.langEn") }}
        </button>
      </div>
    </div>

    <div class="settings-section">
      <h3>{{ t("settings.library") }}</h3>
      <div class="settings-row">
        <span class="k">{{ t("settings.defaultFolder") }}</span>
        <span class="spacer"></span>
        <code class="caption" style="max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
          {{ settingsStore.defaultFolder || t("settings.defaultFolderUnset") }}
        </code>
        <button class="btn small" @click="pickDefaultFolder">{{ t("common.browse") }}</button>
      </div>
    </div>

    <div class="settings-section">
      <h3>{{ t("settings.behavior") }}</h3>
      <div class="settings-row">
        <span class="k">{{ t("settings.confirmRemove") }}</span>
        <span class="spacer"></span>
        <button
          class="toggle"
          :class="{ on: settingsStore.confirmRemove }"
          @click="settingsStore.confirmRemove = !settingsStore.confirmRemove"
        ></button>
      </div>
    </div>

    <div class="settings-section">
      <h3>{{ t("settings.about") }}</h3>
      <p class="desc" style="margin-bottom: 0">
        {{ t("settings.aboutText", { version: appVersion }) }}<br />
        {{ t("settings.dataDir") }}<code class="caption">{{ dataDir }}</code><br />
        {{ t("settings.shortcut") }}
      </p>
    </div>
  </div>
</template>
