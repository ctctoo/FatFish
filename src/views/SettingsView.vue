<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../stores/settings";

const settingsStore = useSettingsStore();
const dataDir = ref("读取中…");

import { appDataDir } from "@tauri-apps/api/path";
appDataDir()
  .then((dir) => (dataDir.value = dir))
  .catch(() => (dataDir.value = "无法获取"));

async function pickDefaultFolder() {
  const dir = await open({ directory: true, multiple: false, title: "选择默认项目目录" });
  if (typeof dir === "string") settingsStore.defaultFolder = dir;
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1>Settings</h1>
    </div>

    <div class="settings-section">
      <h3>Appearance</h3>
      <div class="desc">选择界面主题。跟随系统时会自动响应系统深浅色变化。</div>
      <div class="radio-row">
        <button
          class="radio-pill"
          :class="{ active: settingsStore.theme === 'system' }"
          @click="settingsStore.theme = 'system'"
        >
          System
        </button>
        <button
          class="radio-pill"
          :class="{ active: settingsStore.theme === 'light' }"
          @click="settingsStore.theme = 'light'"
        >
          Light
        </button>
        <button
          class="radio-pill"
          :class="{ active: settingsStore.theme === 'dark' }"
          @click="settingsStore.theme = 'dark'"
        >
          Dark
        </button>
      </div>
    </div>

    <div class="settings-section">
      <h3>Project Library</h3>
      <div class="settings-row">
        <span class="k">Default Project Folder</span>
        <span class="spacer"></span>
        <code class="caption" style="max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
          {{ settingsStore.defaultFolder || "未设置" }}
        </code>
        <button class="btn small" @click="pickDefaultFolder">浏览…</button>
      </div>
    </div>

    <div class="settings-section">
      <h3>Behavior</h3>
      <div class="settings-row">
        <span class="k">删除项目前确认</span>
        <span class="spacer"></span>
        <button class="toggle" :class="{ on: settingsStore.confirmRemove }" @click="settingsStore.confirmRemove = !settingsStore.confirmRemove"></button>
      </div>
    </div>

    <div class="settings-section">
      <h3>About</h3>
      <p class="desc" style="margin-bottom: 0">
        Project Hub v0.1.0 — 你电脑里所有正在做的事，都在这里。<br />
        数据目录：<code class="caption">{{ dataDir }}</code><br />
        快捷键：Ctrl+K 打开全局搜索。
      </p>
    </div>
  </div>
</template>
