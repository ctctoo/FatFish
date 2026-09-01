<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useRouter } from "vue-router";
import Sidebar from "./components/app/Sidebar.vue";
import CommandPalette from "./components/app/CommandPalette.vue";
import Onboarding from "./components/app/Onboarding.vue";
import { useUiStore } from "./stores/ui";
import { useTagStore } from "./stores/tag";
import { useCollectionStore } from "./stores/collection";
import { useSettingsStore } from "./stores/settings";
import { useGithubStore } from "./stores/github";

const router = useRouter();
const uiStore = useUiStore();
const tagStore = useTagStore();
const collectionStore = useCollectionStore();
const settingsStore = useSettingsStore();
const githubStore = useGithubStore();

const showPalette = ref(false);

// 方向感知的页面切换：进入详情（栈变深）向右滑入，返回（栈变浅）向左滑回。
// 名称对应 motion.css 里的 .page-fwd-* / .page-back-*。
const transitionName = ref("page");

router.afterEach((to, from) => {
  const depth = (r: typeof to) => (r.meta?.depth as number) ?? 0;
  transitionName.value = depth(to) > depth(from) ? "page-fwd" : "page-back";
});

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
    githubStore.fetchStatus(),
  ]);
  if (githubStore.account) await githubStore.fetchRepos();
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
          <Transition :name="transitionName" mode="out-in" @enter="onPageEnter">
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
