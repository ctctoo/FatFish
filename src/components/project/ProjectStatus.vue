<script setup lang="ts">
import { statusLabel } from "../../i18n";
import { useSettingsStore } from "../../stores/settings";
import type { ProjectStatus } from "../../types";

const props = defineProps<{
  status: ProjectStatus;
}>();

const settings = useSettingsStore();
</script>

<template>
  <span class="status-chip">
    <span class="status-dot" :class="`status-${props.status}`"></span>
    {{ statusLabel(settings.locale, props.status) }}
  </span>
</template>
