<script setup lang="ts">
import { useI18n } from "../../i18n";

defineProps<{
  /** i18n 键，如 "settings.defaultFolder" */
  label: string;
}>();

const { t } = useI18n();
</script>

<template>
  <div class="settings-row">
    <span class="k">{{ t(label) }}</span>
    <span class="spacer" />
    <slot />
  </div>
</template>
