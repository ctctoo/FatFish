<script setup lang="ts">
import { useI18n } from "../../i18n";

const props = defineProps<{
  title: string;
  message?: string;
  confirmText?: string;
  danger?: boolean;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="modal-mask" @click.self="emit('cancel')">
    <div class="modal" style="width: min(400px, 90vw)">
      <div class="modal-header">
        <h2>{{ props.title }}</h2>
      </div>
      <p v-if="props.message" class="text-secondary" style="font-size: 13.5px; line-height: 1.6">
        {{ props.message }}
      </p>
      <div class="modal-actions">
        <button class="btn" @click="emit('cancel')">{{ t("common.cancel") }}</button>
        <button class="btn" :class="props.danger ? 'danger' : 'primary'" @click="emit('confirm')">
          {{ props.confirmText ?? t("common.confirm") }}
        </button>
      </div>
    </div>
  </div>
</template>
