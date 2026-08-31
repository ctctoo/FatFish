<script setup lang="ts">
import { useI18n } from "../../i18n";

defineProps<{
  /** i18n 键，如 "settings.profile" */
  title: string;
  /** i18n 键，可选描述 */
  desc?: string;
  /** 锚点 id，用于左侧导航定位 */
  id?: string;
}>();

const { t } = useI18n();
</script>

<template>
  <section :id="id" class="settings-section">
    <h3>{{ t(title) }}</h3>
    <p v-if="desc" class="desc">{{ t(desc) }}</p>
    <slot />
  </section>
</template>
