<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import Sidebar from "./components/app/Sidebar.vue";
import CommandPalette from "./components/app/CommandPalette.vue";
import { useUiStore } from "./stores/ui";
import { useTagStore } from "./stores/tag";
import { useCollectionStore } from "./stores/collection";

const uiStore = useUiStore();
const tagStore = useTagStore();
const collectionStore = useCollectionStore();

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
</script>

<template>
  <div class="app-shell">
    <Sidebar @open-search="showPalette = true" />

    <div class="app-main">
      <div class="app-content">
        <router-view v-slot="{ Component }">
          <Transition name="page" mode="out-in">
            <component :is="Component" />
          </Transition>
        </router-view>
      </div>
    </div>

    <CommandPalette v-if="showPalette" @close="showPalette = false" />

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
