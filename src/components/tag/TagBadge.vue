<script setup lang="ts">
import { computed } from "vue";
import { tagChipFor } from "../../types";
import type { Tag } from "../../types";

const props = defineProps<{
  tag: Tag;
  small?: boolean;
  removable?: boolean;
}>();

const emit = defineEmits<{
  remove: [];
}>();

const chip = computed(() => tagChipFor(props.tag.name, props.tag.color));
</script>

<template>
  <span
    class="tag-badge"
    :class="{ small }"
    :style="{ background: chip.bg, color: chip.text, borderColor: 'transparent' }"
  >
    {{ tag.name }}
    <span v-if="removable" class="tag-x" @click.stop="emit('remove')">✕</span>
  </span>
</template>
