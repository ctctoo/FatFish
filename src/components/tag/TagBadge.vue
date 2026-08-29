<script setup lang="ts">
import type { Tag } from "../../types";

defineProps<{
  tag: Tag;
  small?: boolean;
  removable?: boolean;
}>();

const emit = defineEmits<{
  remove: [];
}>();
</script>

<template>
  <span class="tag-badge" :class="{ small }">
    {{ tag.name }}
    <span v-if="removable" class="tag-x" @click.stop="emit('remove')">✕</span>
  </span>
</template>
