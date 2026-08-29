<script setup lang="ts">
defineProps<{
  glyph?: string;
  title: string;
  message?: string;
}>();
</script>

<template>
  <div class="empty-state">
    <div class="glyph">{{ glyph ?? "◇" }}</div>
    <h3>{{ title }}</h3>
    <p v-if="message">{{ message }}</p>
    <div class="actions">
      <slot />
    </div>
  </div>
</template>
