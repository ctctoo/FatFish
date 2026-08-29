<script setup lang="ts">
import ProjectStatus from "./ProjectStatus.vue";
import TagBadge from "../tag/TagBadge.vue";
import type { Project } from "../../types";
import { relativeTime } from "../../types";

defineProps<{
  projects: Project[];
}>();

const emit = defineEmits<{
  open: [project: Project];
}>();
</script>

<template>
  <table class="project-table">
    <thead>
      <tr>
        <th>项目</th>
        <th>标签</th>
        <th>状态</th>
        <th>更新时间</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="project in projects" :key="project.id" @click="emit('open', project)">
        <td class="name-cell">{{ project.name }}</td>
        <td>
          <span style="display: inline-flex; gap: 5px; flex-wrap: wrap">
            <TagBadge v-for="tag in project.tags.slice(0, 3)" :key="tag.id" :tag="tag" small />
          </span>
        </td>
        <td><ProjectStatus :status="project.status" /></td>
        <td class="dim-cell">{{ relativeTime(project.updatedAt) }}</td>
      </tr>
    </tbody>
  </table>
</template>
