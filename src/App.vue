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
import { useProjectStore } from "./stores/project";

const router = useRouter();
const uiStore = useUiStore();
const tagStore = useTagStore();
const collectionStore = useCollectionStore();
const settingsStore = useSettingsStore();
const githubStore = useGithubStore();
const projectStore = useProjectStore();

const showPalette = ref(false);

// 方向感知的页面切换：进入详情（栈变深）向右滑入，返回（栈变浅）向左滑回。
// 名称对应 motion.css 里的 .page-fwd-* / .page-back-*。
const transitionName = ref("page");

// 同一视图组件间跳转（如 /projects → /projects/:id → /projects）时，
// 用 fullPath 作 key 强制重建组件，保证路由过渡每次都触发。
router.beforeEach((to, from) => {
  const depth = (r: typeof to) => (r.meta?.depth as number) ?? 0;
  // 深入 → 前进（右推）；返回 → 后退（左回）；平级切换 → 柔和上浮，避免方向错乱。
  transitionName.value =
    depth(to) > depth(from) ? "page-fwd" : depth(to) < depth(from) ? "page-back" : "page";
  if (to.path === from.path) return false;
});

router.afterEach(() => {
  // 旧页面退场后新页面入场：归零滚动，避免新页面从旧偏移处出现
  const el = document.querySelector(".app-content");
  if (el) el.scrollTop = 0;
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
    projectStore.refreshStats(),
  ]);
  if (githubStore.account) await githubStore.fetchRepos();
});

onUnmounted(() => window.removeEventListener("keydown", onGlobalKeydown));
</script>

<template>
  <div class="app-shell">
    <Sidebar @open-search="showPalette = true" />

    <div class="app-main">
      <div class="app-content">
        <router-view v-slot="{ Component }">
          <Transition :name="transitionName" mode="out-in">
            <component :is="Component" :key="$route.fullPath" />
          </Transition>
        </router-view>
      </div>
    </div>

    <Transition name="overlay-out">
      <CommandPalette v-if="showPalette" @close="showPalette = false" />
    </Transition>

    <Onboarding v-if="!settingsStore.onboarded" />

    <div class="toast-wrap">
      <TransitionGroup name="toast">
        <div
          v-for="toast in uiStore.toasts"
          :key="toast.id"
          class="toast"
          :class="toast.type"
          @click="uiStore.dismissToast(toast.id)"
        >
          {{ toast.message }}
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>
