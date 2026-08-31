<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  Search,
  House,
  LayoutGrid,
  Clock,
  Star,
  FolderCog,
  Tags,
  Settings,
  PanelLeftClose,
  PanelLeftOpen,
  Plus,
} from "lucide-vue-next";
import { useProjectStore } from "../../stores/project";
import { useTagStore } from "../../stores/tag";
import { useCollectionStore } from "../../stores/collection";
import { useUiStore } from "../../stores/ui";
import { useI18n } from "../../i18n";

const emit = defineEmits<{ openSearch: [] }>();

const route = useRoute();
const router = useRouter();
const projectStore = useProjectStore();
const tagStore = useTagStore();
const collectionStore = useCollectionStore();
const uiStore = useUiStore();
const { t } = useI18n();

const collapsed = ref(false);
const newCollectionName = ref("");
const addingCollection = ref(false);

const activeNav = computed(() => {
  if (route.path === "/home") return "home";
  if (route.path === "/projects") return "projects";
  if (route.path === "/recent") return "recent";
  if (route.path === "/favorites") return "favorites";
  if (route.path.startsWith("/collections")) return route.params.id as string;
  if (route.path.startsWith("/tags/")) return `tag:${route.params.id}`;
  if (route.path === "/tags") return "manage-tags";
  if (route.path === "/settings") return "settings";
  return "";
});

function go(path: string) {
  if (route.path !== path) router.push(path);
}

async function addCollection() {
  const name = newCollectionName.value.trim();
  if (!name) {
    addingCollection.value = false;
    return;
  }
  try {
    const created = await collectionStore.createCollection({ name });
    newCollectionName.value = "";
    addingCollection.value = false;
    router.push(`/collections/${created.id}`);
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}
</script>

<template>
  <aside class="sidebar" :class="{ collapsed }">
    <div class="sidebar-logo">
      <img src="../../assets/logo.png" alt="FatFish" />
      <span class="logo-text">FatFish</span>
    </div>

    <button class="sidebar-item" @click="emit('openSearch')">
      <Search :size="16" :stroke-width="1.8" />
      <span class="label">{{ t("side.search") }}</span>
      <span class="count">⌘K</span>
    </button>

    <div class="sidebar-section">
      <div class="sidebar-label">{{ t("side.home") }}</div>
      <button class="sidebar-item" :class="{ active: activeNav === 'home' }" @click="go('/home')">
        <House :size="16" :stroke-width="1.8" />
        <span class="label">{{ t("side.home") }}</span>
      </button>
    </div>

    <div class="sidebar-section">
      <div class="sidebar-label">{{ t("side.allProjects") }}</div>
      <button class="sidebar-item" :class="{ active: activeNav === 'projects' }" @click="go('/projects')">
        <LayoutGrid :size="16" :stroke-width="1.8" />
        <span class="label">{{ t("side.allProjects") }}</span>
        <span class="count">{{ projectStore.projects.length }}</span>
      </button>
      <button class="sidebar-item" :class="{ active: activeNav === 'recent' }" @click="go('/recent')">
        <Clock :size="16" :stroke-width="1.8" />
        <span class="label">{{ t("side.recent") }}</span>
      </button>
      <button class="sidebar-item" :class="{ active: activeNav === 'favorites' }" @click="go('/favorites')">
        <Star :size="16" :stroke-width="1.8" />
        <span class="label">{{ t("side.favorites") }}</span>
      </button>
    </div>

    <div class="sidebar-section">
      <div class="sidebar-label">
        {{ t("side.collections") }}
        <button class="section-add-btn" :title="t('side.newCollection')" @click="addingCollection = !addingCollection">
          <Plus :size="13" :stroke-width="1.8" />
        </button>
      </div>
      <input
        v-if="addingCollection"
        v-model="newCollectionName"
        class="collection-input"
        type="text"
        :placeholder="t('side.collectionPh')"
        @keyup.enter="addCollection"
        @blur="addCollection"
      />
      <button
        v-for="c in collectionStore.collections"
        :key="c.id"
        class="sidebar-item"
        :class="{ active: activeNav === c.id }"
        @click="go(`/collections/${c.id}`)"
      >
        <FolderCog :size="16" :stroke-width="1.8" />
        <span class="label">{{ c.name }}</span>
      </button>
      <button v-if="!collectionStore.collections.length && !addingCollection" class="sidebar-item" @click="addingCollection = true">
        <Plus :size="16" :stroke-width="1.8" />
        <span class="label">{{ t("side.newCollection") }}</span>
      </button>
    </div>

    <div class="sidebar-section">
      <div class="sidebar-label">{{ t("side.tags") }}</div>
      <button
        v-for="tag in tagStore.tags"
        :key="tag.id"
        class="sidebar-item"
        :class="{ active: activeNav === `tag:${tag.id}` }"
        @click="go(`/tags/${tag.id}`)"
      >
        <Tags :size="16" :stroke-width="1.8" />
        <span class="label">{{ tag.name }}</span>
      </button>
      <button class="sidebar-item" :class="{ active: activeNav === 'manage-tags' }" @click="go('/tags')">
        <span class="label manage-tags">{{ t("side.manageTags") }}</span>
      </button>
    </div>

    <div class="sidebar-footer">
      <button class="sidebar-item" :class="{ active: activeNav === 'settings' }" @click="go('/settings')">
        <Settings :size="16" :stroke-width="1.8" />
        <span class="label">{{ t("side.settings") }}</span>
      </button>
      <button class="sidebar-item collapse-btn" @click="collapsed = !collapsed">
        <PanelLeftClose v-if="!collapsed" :size="16" :stroke-width="1.8" />
        <PanelLeftOpen v-else :size="16" :stroke-width="1.8" />
        <span class="label">{{ t("side.collapse") }}</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.collection-input {
  margin: 2px 0 4px;
  width: 100%;
  padding: 6px 10px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 12.5px;
  outline: none;
}

.sidebar.collapsed .collection-input {
  display: none;
}

.manage-tags {
  padding-left: 26px;
}
</style>
