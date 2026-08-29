<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir } from "@tauri-apps/api/path";
import { useSettingsStore } from "../stores/settings";
import { useI18n } from "../i18n";

const settingsStore = useSettingsStore();
const { t } = useI18n();
const dataDir = ref(t("misc.loading"));

appDataDir()
  .then((dir) => (dataDir.value = dir))
  .catch(() => (dataDir.value = t("misc.cannotGet")));

async function pickDefaultFolder() {
  const dir = await open({ directory: true, multiple: false, title: t("settings.defaultFolder") });
  if (typeof dir === "string") settingsStore.defaultFolder = dir;
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1>{{ t("settings.title") }}</h1>
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
        {{ t("settings.aboutText") }}<br />
        {{ t("settings.dataDir") }}<code class="caption">{{ dataDir }}</code><br />
        {{ t("settings.shortcut") }}
      </p>
    </div>
  </div>
</template>
