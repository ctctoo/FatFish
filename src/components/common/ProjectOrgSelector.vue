<script setup lang="ts">
import { onMounted } from "vue";
import { useTagStore } from "../../stores/tag";
import { useCollectionStore } from "../../stores/collection";

defineProps<{
  selectedTagIds: string[];
  selectedCollectionIds: string[];
}>();

const emit = defineEmits<{
  toggleTag: [tagId: string];
  toggleCollection: [collectionId: string];
}>();

const tagStore = useTagStore();
const collectionStore = useCollectionStore();

onMounted(async () => {
  if (!tagStore.tags.length) await tagStore.fetchTags();
  if (!collectionStore.collections.length) await collectionStore.fetchCollections();
});
</script>

<template>
  <div class="form-grid" style="gap: 12px">
    <div class="field">
      <label>Collections（人为组织项目）</label>
      <div class="chip-row">
        <button
          v-for="c in collectionStore.collections"
          :key="c.id"
          class="chip"
          :class="{ selected: selectedCollectionIds.includes(c.id) }"
          @click="emit('toggleCollection', c.id)"
        >
          {{ c.name }}
        </button>
        <span v-if="!collectionStore.collections.length" class="caption">暂无集合，可在侧栏创建</span>
      </div>
    </div>

    <div class="field">
      <label>Tags（描述项目属性）</label>
      <div class="chip-row">
        <button
          v-for="t in tagStore.tags"
          :key="t.id"
          class="chip"
          :class="{ selected: selectedTagIds.includes(t.id) }"
          @click="emit('toggleTag', t.id)"
        >
          {{ t.name }}
        </button>
        <span v-if="!tagStore.tags.length" class="caption">暂无标签，可在「管理标签」中创建</span>
      </div>
    </div>
  </div>
</template>
