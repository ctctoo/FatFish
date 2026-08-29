<script setup lang="ts">
import { statusLabel } from "../../types";

const props = defineProps<{
  status: string;
}>();
</script>

<template>
  <span class="status-chip">
    <span class="status-dot" :class="`status-${props.status}`"></span>
    {{ statusLabel(props.status) }}
  </span>
</template>
