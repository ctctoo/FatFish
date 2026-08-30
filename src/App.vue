<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import Sidebar from "./components/app/Sidebar.vue";
import CommandPalette from "./components/app/CommandPalette.vue";
import Onboarding from "./components/app/Onboarding.vue";
import { useUiStore } from "./stores/ui";
import { useTagStore } from "./stores/tag";
import { useCollectionStore } from "./stores/collection";
import { useSettingsStore } from "./stores/settings";

const uiStore = useUiStore();
const tagStore = useTagStore();
const collectionStore = useCollectionStore();
const settingsStore = useSettingsStore();

const showPalette = ref(false);

function onGlobalKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
    e.preventDefault();
    showPalette.value = true;
  }
}

onMounted(async () => {
  window.addEventListener("keydown", onGlobalKeydown);
  await Promise.all([
    tagStore.fetchTags(),
    collectionStore.fetchCollections(),
  ]);
});

onUnmounted(() => window.removeEventListener("keydown", onGlobalKeydown));

// 新页面开始入场时把滚动容器归零，避免新页面从旧页面的滚动偏移处进入
function onPageEnter() {
  const el = document.querySelector(".app-content");
  if (el) el.scrollTop = 0;
}
</script>

<template>
  <div class="app-shell">
    <Sidebar @open-search="showPalette = true" />

    <div class="app-main">
      <div class="app-content">
        <router-view v-slot="{ Component }">
          <Transition name="page" mode="out-in" @enter="onPageEnter">
            <component :is="Component" />
          </Transition>
        </router-view>
      </div>
    </div>

    <CommandPalette v-if="showPalette" @close="showPalette = false" />

    <Onboarding v-if="!settingsStore.onboarded" />

    <div class="toast-wrap">
      <div
        v-for="toast in uiStore.toasts"
        :key="toast.id"
        class="toast"
        :class="toast.type"
        @click="uiStore.dismissToast(toast.id)"
      >
        {{ toast.message }}
      </div>
    </div>
  </div>
</template>
