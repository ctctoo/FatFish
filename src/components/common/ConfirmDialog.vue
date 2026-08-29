<script setup lang="ts">
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
        <button class="btn" @click="emit('cancel')">取消</button>
        <button class="btn" :class="props.danger ? 'danger' : 'primary'" @click="emit('confirm')">
          {{ props.confirmText ?? "确定" }}
        </button>
      </div>
    </div>
  </div>
</template>
