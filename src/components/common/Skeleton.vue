<script setup lang="ts">
defineProps<{
  count?: number;
}>();
</script>

<template>
  <div class="project-grid">
    <div v-for="i in count ?? 6" :key="i" class="skeleton-card">
      <div class="sk-cover skeleton"></div>
      <div class="sk-body">
        <div class="skeleton" style="width: 60%; height: 15px"></div>
        <div class="skeleton" style="width: 90%; height: 11px"></div>
        <div class="skeleton" style="width: 75%; height: 11px"></div>
        <div class="skeleton" style="width: 40%; height: 11px; margin-top: auto"></div>
      </div>
    </div>
  </div>
</template>
