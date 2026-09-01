<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Search } from "lucide-vue-next";
import { tauriApi } from "../../services/tauri";
import type { Project } from "../../types";
import { statusLabel, relativeTime, useI18n } from "../../i18n";
import { useSettingsStore } from "../../stores/settings";

const emit = defineEmits<{ close: [] }>();

const router = useRouter();
const { t } = useI18n();
const settings = useSettingsStore();

const query = ref("");
const results = ref<Project[]>([]);
const recent = ref<Project[]>([]);
const activeIndex = ref(0);

const inputEl = ref<HTMLInputElement | null>(null);

const items = computed(() => (query.value.trim() ? results.value : recent.value));

onMounted(async () => {
  inputEl.value?.focus();
  try {
    recent.value = await tauriApi.listProjects({ recent: true, sort: "opened" });
  } catch {
    recent.value = [];
  }
});

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let searchSeq = 0;

function onInput() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(async () => {
    const q = query.value.trim();
    if (!q) {
      searchSeq++;
      results.value = [];
      activeIndex.value = 0;
      return;
    }
    // 竞态保护：只接受最后一次请求，防止快速输入时旧结果覆盖新结果
    const seq = ++searchSeq;
    const list = await tauriApi.listProjects({ query: q });
    if (seq !== searchSeq) return;
    results.value = list;
    activeIndex.value = 0;
  }, 200);
}

function open(project: Project) {
  emit("close");
  router.push(`/projects/${project.id}`);
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    emit("close");
    return;
  }
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (!items.value.length) return;
    activeIndex.value = (activeIndex.value + 1) % items.value.length;
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (!items.value.length) return;
    activeIndex.value = (activeIndex.value - 1 + items.value.length) % items.value.length;
  } else if (e.key === "Enter") {
    // 防止过滤后索引越界选中错误项
    const item = items.value[Math.min(activeIndex.value, items.value.length - 1)];
    if (item) open(item);
  }
}

function itemSub(project: Project): string {
  const parts: string[] = [];
  if (project.collections.length) parts.push(project.collections.map((c) => c.name).join(" · "));
  if (project.tags.length) parts.push(project.tags.map((tag) => tag.name).join(" · "));
  return parts.join("  ·  ") || statusLabel(settings.locale, project.status);
}
</script>

<template>
  <div class="palette-mask" @click.self="emit('close')" @keydown="onKeydown">
    <div class="palette">
      <div class="palette-input-row">
        <Search :size="17" :stroke-width="1.8" />
        <input
          ref="inputEl"
          v-model="query"
          type="text"
          :placeholder="t('palette.ph')"
          @input="onInput"
        />
      </div>

      <div class="palette-list">
        <div class="palette-group-label">{{ query.trim() ? t("palette.results") : t("palette.recent") }}</div>
        <button
          v-for="(project, i) in items"
          :key="project.id"
          class="palette-item"
          :class="{ active: i === activeIndex }"
          :ref="i === activeIndex ? (el) => (el as HTMLElement)?.scrollIntoView({ block: 'nearest' }) : undefined"
          @click="open(project)"
          @mousemove="activeIndex = i"
        >
          <span class="name">{{ project.name }}</span>
          <span class="sub">{{ itemSub(project) }} · {{ relativeTime(settings.locale, project.updatedAt) }}</span>
        </button>
        <div v-if="!items.length" class="palette-empty caption" style="padding: 14px">
          {{ query.trim() ? t("palette.noMatch") : t("palette.noProjects") }}
        </div>
      </div>

      <div class="palette-footer">
        <span>{{ t("palette.navigate") }}</span>
        <span>{{ t("palette.enter") }}</span>
        <span>{{ t("palette.esc") }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.palette-empty {
  color: var(--text-tertiary);
}
</style>
