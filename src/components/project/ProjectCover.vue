<script setup lang="ts">
import { computed } from "vue";
import { coverColorFor } from "../../types";

const props = defineProps<{
  name: string;
  emoji: string | null;
  color: string | null;
  size?: "card" | "detail";
}>();

const bg = computed(() => coverColorFor(props.name, props.color));
const letter = computed(() => props.name.charAt(0).toUpperCase() || "◇");
</script>

<template>
  <div class="project-cover" :class="props.size ?? 'card'" :style="{ background: bg }">
    <span v-if="props.emoji">{{ props.emoji }}</span>
    <span v-else class="cover-letter">{{ letter }}</span>
  </div>
</template>

<style scoped>
.project-cover {
  display: grid;
  place-items: center;
  user-select: none;
}

.project-cover.card {
  height: 100px;
  width: 100%;
}

.project-cover.detail {
  width: 96px;
  height: 96px;
  border-radius: 20px;
  font-size: 38px;
}

.cover-letter {
  font-size: 34px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.92);
  text-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
  line-height: 1;
}
</style>
