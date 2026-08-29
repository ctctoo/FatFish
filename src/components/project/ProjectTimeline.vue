<script setup lang="ts">
import { computed } from "vue";
import { Plus, Pencil, Link2, SquareCheck } from "lucide-vue-next";
import type { Activity } from "../../types";

const props = defineProps<{
  activities: Activity[];
}>();

const groups = computed(() => {
  const map = new Map<string, Activity[]>();
  for (const activity of props.activities) {
    const label = dateGroupLabel(activity.createdAt);
    const list = map.get(label) ?? [];
    list.push(activity);
    map.set(label, list);
  }
  return [...map.entries()].map(([date, items]) => ({ date, items }));
});

function dateGroupLabel(iso: string): string {
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;
  const now = new Date();
  const sameDay = (a: Date, b: Date) =>
    a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
  const yesterday = new Date(now.getTime() - 86_400_000);
  if (sameDay(d, now)) return "今天";
  if (sameDay(d, yesterday)) return "昨天";
  const md = `${d.getMonth() + 1}月${d.getDate()}日`;
  return d.getFullYear() === now.getFullYear() ? md : `${d.getFullYear()}年${md}`;
}

function timeOf(iso: string): string {
  const d = new Date(iso);
  if (isNaN(d.getTime())) return "";
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

function iconFor(kind: string) {
  switch (kind) {
    case "created": return Plus;
    case "link": return Link2;
    case "todo": return SquareCheck;
    default: return Pencil;
  }
}
</script>

<template>
  <div class="timeline">
    <div v-for="group in groups" :key="group.date" class="timeline-group">
      <div class="timeline-date">{{ group.date }}</div>
      <div v-for="activity in group.items" :key="activity.id" class="timeline-item">
        <span class="timeline-icon" :class="`kind-${activity.kind}`">
          <component :is="iconFor(activity.kind)" :size="12" :stroke-width="2" />
        </span>
        <span class="timeline-message">{{ activity.message }}</span>
        <span class="timeline-time">{{ timeOf(activity.createdAt) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.timeline-group + .timeline-group {
  margin-top: 14px;
}

.timeline-date {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  padding-left: 26px;
  margin-bottom: 6px;
}

.timeline-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 5px 0 5px 26px;
}

/* 竖向时间轴线 */
.timeline-item::before {
  content: "";
  position: absolute;
  left: 9px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--border);
}

.timeline-group .timeline-item:first-of-type::before {
  top: 50%;
}

.timeline-group .timeline-item:last-of-type::before {
  bottom: 50%;
}

.timeline-group .timeline-item:only-of-type::before {
  display: none;
}

.timeline-icon {
  position: relative;
  z-index: 1;
  width: 19px;
  height: 19px;
  border-radius: 50%;
  display: inline-grid;
  place-items: center;
  flex-shrink: 0;
  background: var(--surface-muted);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.timeline-icon.kind-created {
  background: var(--accent-soft);
  color: var(--accent);
  border-color: transparent;
}

.timeline-icon.kind-link {
  background: #e8eefb;
  color: #3b6fd4;
  border-color: transparent;
}

[data-theme="dark"] .timeline-icon.kind-link {
  background: rgba(96, 140, 220, 0.18);
  color: #7ea3e0;
}

.timeline-icon.kind-todo {
  background: #e5f3ea;
  color: #3d8b5f;
  border-color: transparent;
}

[data-theme="dark"] .timeline-icon.kind-todo {
  background: rgba(76, 158, 108, 0.18);
  color: #6fbd91;
}

.timeline-message {
  font-size: 13.5px;
  color: var(--text-primary);
}

.timeline-time {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-tertiary);
}
</style>
