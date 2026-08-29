<script setup lang="ts">
import ProjectCard from "./ProjectCard.vue";
import type { Project } from "../../types";

defineProps<{
  projects: Project[];
}>();

const emit = defineEmits<{
  open: [project: Project];
  openFolder: [project: Project];
  toggleFavorite: [project: Project];
  action: [action: string, project: Project];
}>();
</script>

<template>
  <div class="project-grid">
    <ProjectCard
      v-for="project in projects"
      :key="project.id"
      :project="project"
      @open="(p) => emit('open', p)"
      @open-folder="(p) => emit('openFolder', p)"
      @toggle-favorite="(p) => emit('toggleFavorite', p)"
      @action="(a, p) => emit('action', a, p)"
    />
  </div>
</template>
