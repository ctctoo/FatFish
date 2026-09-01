<script setup lang="ts">
import TodoWidget from "../components/todo/TodoWidget.vue";
import { useI18n } from "../i18n";

const { t } = useI18n();
</script>

<template>
  <div class="page page-stagger" style="max-width: 860px">
    <div class="page-header">
      <h1>{{ t("todo.allTasks") }}</h1>
      <span class="count">{{ t("todo.includingDone") }}</span>
    </div>
    <TodoWidget mode="all" />
  </div>
</template>
