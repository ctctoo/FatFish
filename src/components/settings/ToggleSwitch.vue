<script setup lang="ts">
withDefaults(defineProps<{ modelValue: boolean; disabled?: boolean }>(), {
  disabled: false,
});

const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();
</script>

<template>
  <button
    class="toggle"
    :class="{ on: modelValue }"
    role="switch"
    :aria-checked="modelValue"
    :disabled="disabled"
    @click="emit('update:modelValue', !modelValue)"
  />
</template>
