<script setup lang="ts">
import { ref } from "vue";
import { MoreHorizontal, Star } from "lucide-vue-next";
import ProjectCover from "./ProjectCover.vue";
import ProjectStatus from "./ProjectStatus.vue";
import ProjectMenu from "./ProjectMenu.vue";
import TagBadge from "../tag/TagBadge.vue";
import type { Project } from "../../types";
import { relativeTime } from "../../types";

const props = defineProps<{
  project: Project;
}>();

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
</script>

<template>
  <div
    class="project-card"
    @click="emit('open', props.project)"
    @contextmenu="openMenu"
  >
    <div class="card-cover-wrap" style="position: relative">
      <ProjectCover :name="project.name" :emoji="project.coverEmoji" :color="project.coverColor" size="card" />
      <button
        class="card-fav"
        :class="{ on: project.favorite }"
        title="收藏"
        @click.stop="emit('toggleFavorite', props.project)"
      >
        <Star :size="15" :stroke-width="1.8" :fill="project.favorite ? 'currentColor' : 'none'" />
      </button>
      <button class="card-open" @click.stop="emit('openFolder', props.project)">Open</button>
    </div>

    <div class="card-body">
      <div class="card-title-row">
        <span class="card-title" :title="project.name">{{ project.name }}</span>
        <button class="card-more" title="更多操作" @click.stop="openMenu($event)">
          <MoreHorizontal :size="16" :stroke-width="1.8" />
        </button>
      </div>

      <div class="card-desc">{{ project.description || "暂无描述" }}</div>

      <div class="card-tags">
        <TagBadge v-for="tag in project.tags.slice(0, 3)" :key="tag.id" :tag="tag" small />
      </div>

      <div class="card-footer">
        <ProjectStatus :status="project.status" />
        <span class="caption">Updated {{ relativeTime(project.updatedAt) }}</span>
      </div>
    </div>

    <ProjectMenu ref="menuRef" :hasGitHubLink="githubLink()" @action="onMenuAction" />
  </div>
</template>

<style scoped>
.card-cover-wrap :deep(.project-cover.card) {
  border-radius: 0;
}
</style>
