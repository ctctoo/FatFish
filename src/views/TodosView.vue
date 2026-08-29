<script setup lang="ts">
import TodoWidget from "../components/todo/TodoWidget.vue";
</script>

<template>
  <div class="page" style="max-width: 860px">
    <div class="page-header">
      <h1>全部任务</h1>
      <span class="count">包含已完成</span>
    </div>
    <TodoWidget mode="all" />
  </div>
</template>
