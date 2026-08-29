<script setup lang="ts">
import { statusLabel } from "../../i18n";
import { useSettingsStore } from "../../stores/settings";

const props = defineProps<{
  status: string;
}>();

const settings = useSettingsStore();
</script>

<template>
  <span class="status-chip">
    <span class="status-dot" :class="`status-${props.status}`"></span>
    {{ statusLabel(settings.locale, props.status) }}
  </span>
</template>
