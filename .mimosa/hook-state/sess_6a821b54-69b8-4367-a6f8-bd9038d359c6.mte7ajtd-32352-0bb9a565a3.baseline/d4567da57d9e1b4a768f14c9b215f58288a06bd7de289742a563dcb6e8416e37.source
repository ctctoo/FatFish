import { defineStore } from "pinia";
import { ref } from "vue";
import { tauriApi } from "../services/tauri";
import type { Collection, CollectionInput } from "../types";

export const useCollectionStore = defineStore("collection", () => {
  const collections = ref<Collection[]>([]);

  async function fetchCollections() {
    collections.value = await tauriApi.listCollections();
  }

  async function createCollection(input: CollectionInput) {
    const collection = await tauriApi.createCollection(input);
    await fetchCollections();
    return collection;
  }

  async function updateCollection(id: string, input: CollectionInput) {
    await tauriApi.updateCollection(id, input);
    await fetchCollections();
  }

  async function deleteCollection(id: string) {
    await tauriApi.deleteCollection(id);
    await fetchCollections();
  }

  return { collections, fetchCollections, createCollection, updateCollection, deleteCollection };
});
