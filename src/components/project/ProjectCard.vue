<script setup lang="ts">
import { computed, ref } from "vue";
import { MoreHorizontal, Star, Folder } from "lucide-vue-next";
import ProjectStatus from "./ProjectStatus.vue";
import ProjectMenu from "./ProjectMenu.vue";
import TagBadge from "../tag/TagBadge.vue";
import { useI18n } from "../../i18n";

const { t } = useI18n();
import type { Project } from "../../types";
import { coverColorFor } from "../../types";
import { relativeTime } from "../../i18n";
import { useSettingsStore } from "../../stores/settings";

const props = defineProps<{
  project: Project;
}>();

const settings = useSettingsStore();

const emit = defineEmits<{
  open: [project: Project];
  openFolder: [project: Project];
  toggleFavorite: [project: Project];
  action: [action: string, project: Project];
}>();

const menuRef = ref<InstanceType<typeof ProjectMenu> | null>(null);

function openMenu(e: MouseEvent) {
  menuRef.value?.open(e);
}

function onMenuAction(action: string) {
  emit("action", action, props.project);
}

const githubLink = () => props.project.links.some((l) => l.linkType === "github");

const tileColor = computed(() => coverColorFor(props.project.name, props.project.coverColor));
const tileBg = computed(() => `color-mix(in srgb, ${tileColor.value} 15%, transparent)`);
const tileLetter = computed(() => props.project.name.charAt(0).toUpperCase() || "◇");
</script>

<template>
  <div
    class="project-card"
    @click="emit('open', props.project)"
    @contextmenu="openMenu"
  >
    <div class="card-main">
      <div class="card-tile" :style="{ background: tileBg, color: tileColor }">
        <span v-if="project.coverEmoji" style="font-size: 28px">{{ project.coverEmoji }}</span>
        <span v-else style="font-size: 26px; font-weight: 700">{{ tileLetter }}</span>
      </div>

      <div class="card-content">
        <div class="card-title-row">
          <span class="card-title" :title="project.name">{{ project.name }}</span>
          <button
            class="card-star"
            :class="{ on: project.favorite }"
            title="收藏"
            @click.stop="emit('toggleFavorite', props.project)"
          >
            <Star :size="16" :stroke-width="1.8" :fill="project.favorite ? 'currentColor' : 'none'" />
          </button>
        </div>

        <div class="card-desc">{{ project.description || "暂无描述" }}</div>

        <div class="card-tags">
          <TagBadge v-for="tag in project.tags.slice(0, 3)" :key="tag.id" :tag="tag" />
        </div>
      </div>
    </div>

    <div class="card-footer">
      <ProjectStatus :status="project.status" />
      <span class="card-divider"></span>
      <span class="card-local">
        <Folder :size="13" :stroke-width="1.8" />
        本地项目
      </span>
      <span class="card-updated">{{ t("card.updated") }} {{ relativeTime(settings.locale, project.updatedAt) }}</span>
      <button class="card-more" title="更多操作" @click.stop="openMenu($event)">
        <MoreHorizontal :size="16" :stroke-width="1.8" />
      </button>
    </div>

    <ProjectMenu ref="menuRef" :hasGitHubLink="githubLink()" @action="onMenuAction" />
  </div>
</template>
