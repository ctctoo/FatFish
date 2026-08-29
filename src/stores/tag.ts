import { defineStore } from "pinia";
import { ref } from "vue";
import { tauriApi } from "../services/tauri";
import type { Tag, TagInput } from "../types";

export const useTagStore = defineStore("tag", () => {
  const tags = ref<Tag[]>([]);

  async function fetchTags() {
    tags.value = await tauriApi.listTags();
  }

  async function createTag(input: TagInput) {
    const tag = await tauriApi.createTag(input);
    await fetchTags();
    return tag;
  }

  async function updateTag(id: string, input: TagInput) {
    await tauriApi.updateTag(id, input);
    await fetchTags();
  }

  async function deleteTag(id: string) {
    await tauriApi.deleteTag(id);
    await fetchTags();
  }

  return { tags, fetchTags, createTag, updateTag, deleteTag };
});
