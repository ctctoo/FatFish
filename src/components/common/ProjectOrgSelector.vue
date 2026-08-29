<script setup lang="ts">
import { onMounted } from "vue";
import { useTagStore } from "../../stores/tag";
import { useCollectionStore } from "../../stores/collection";
import { useI18n } from "../../i18n";

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
const { t } = useI18n();

onMounted(async () => {
  if (!tagStore.tags.length) await tagStore.fetchTags();
  if (!collectionStore.collections.length) await collectionStore.fetchCollections();
});
</script>

<template>
  <div class="form-grid" style="gap: 12px">
    <div class="field">
      <label>{{ t("dialog.project.collectionsHint") }}</label>
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
        <span v-if="!collectionStore.collections.length" class="caption">{{ t("dialog.project.noCollections") }}</span>
      </div>
    </div>

    <div class="field">
      <label>{{ t("dialog.project.tagsHint") }}</label>
      <div class="chip-row">
        <button
          v-for="item in tagStore.tags"
          :key="item.id"
          class="chip"
          :class="{ selected: selectedTagIds.includes(item.id) }"
          @click="emit('toggleTag', item.id)"
        >
          {{ item.name }}
        </button>
        <span v-if="!tagStore.tags.length" class="caption">{{ t("dialog.project.noTags") }}</span>
      </div>
    </div>
  </div>
</template>
