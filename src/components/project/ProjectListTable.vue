<script setup lang="ts">
import ProjectStatus from "./ProjectStatus.vue";
import TagBadge from "../tag/TagBadge.vue";
import type { Project } from "../../types";
import { relativeTime, useI18n } from "../../i18n";
import { useSettingsStore } from "../../stores/settings";

const { t } = useI18n();
const settings = useSettingsStore();

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
        <th>{{ t("side.allProjects") }}</th>
        <th>{{ t("side.tags") }}</th>
        <th>{{ t("projects.filterStatus") }}</th>
        <th>{{ t("card.updated") }}</th>
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
        <td class="dim-cell">{{ relativeTime(settings.locale, project.updatedAt) }}</td>
      </tr>
    </tbody>
  </table>
</template>
