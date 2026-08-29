import { defineStore } from "pinia";
import { ref } from "vue";

export interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "info";
}

let toastSeq = 0;

export const useUiStore = defineStore("ui", () => {
  const toasts = ref<Toast[]>([]);

  function showToast(message: string, type: Toast["type"] = "info") {
    const id = ++toastSeq;
    toasts.value.push({ id, message, type });
    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id);
    }, type === "error" ? 5000 : 2500);
  }

  function dismissToast(id: number) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  return { toasts, showToast, dismissToast };
});
