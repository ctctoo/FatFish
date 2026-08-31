<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { ChevronRight, Plus, Trash2, CalendarDays, Folder } from "lucide-vue-next";
import { useTodoStore } from "../../stores/todo";
import { useProjectStore } from "../../stores/project";
import { useUiStore } from "../../stores/ui";
import { useI18n } from "../../i18n";

const props = defineProps<{
  /** compact：首页组件只展示未完成任务；all：任务页展示全部 */
  mode?: "compact" | "all";
}>();

const router = useRouter();
const todoStore = useTodoStore();
const projectStore = useProjectStore();
const uiStore = useUiStore();
const { t } = useI18n();

const visibleTodos = computed(() =>
  props.mode === "all" ? todoStore.todos : todoStore.todos.filter((todo) => !todo.done).slice(0, 6)
);

const doneCount = computed(() => todoStore.todos.filter((todo) => todo.done).length);

const adding = ref(false);
const newTitle = ref("");
const newProjectId = ref("");
const newDueDate = ref("");

onMounted(async () => {
  await Promise.all([
    todoStore.fetchTodos(),
    projectStore.projects.length ? Promise.resolve() : projectStore.fetchProjects(),
  ]);
});

async function addTodo() {
  const title = newTitle.value.trim();
  if (!title) {
    adding.value = false;
    return;
  }
  try {
    await todoStore.createTodo({
      title,
      projectId: newProjectId.value || null,
      dueDate: newDueDate.value || null,
    });
    newTitle.value = "";
    newProjectId.value = "";
    newDueDate.value = "";
    uiStore.showToast(t("todo.addedToast"), "success");
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

async function toggle(id: string, done: boolean) {
  try {
    await todoStore.toggleTodo(id, done);
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

async function remove(id: string) {
  try {
    await todoStore.deleteTodo(id);
  } catch (e) {
    uiStore.showToast(String(e), "error");
  }
}

function toDateStr(d: Date): string {
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${m}-${day}`;
}

function dueLabel(due: string | null): string {
  if (!due) return "";
  const now = new Date();
  if (due === toDateStr(now)) return t("rel.today");
  if (due === toDateStr(new Date(now.getTime() + 86_400_000))) return t("rel.tomorrow");
  const d = new Date(due + "T00:00:00");
  if (isNaN(d.getTime())) return due;
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

function isOverdue(due: string | null): boolean {
  if (!due) return false;
  return due < toDateStr(new Date());
}

function goProject(projectId: string | null) {
  if (projectId) router.push(`/projects/${projectId}`);
}
</script>

<template>
  <div class="todo-widget">
    <div class="todo-header">
      <span class="todo-title">{{ t("todo.title") }}</span>
      <span class="todo-count">{{ mode === "all" ? todoStore.todos.length : todoStore.todos.length - doneCount }}</span>
      <button v-if="mode !== 'all'" class="todo-all" @click="router.push('/todos')">
        {{ t("todo.allTasks") }} <ChevronRight :size="14" :stroke-width="1.8" />
      </button>
      <span class="spacer"></span>
      <button class="link-btn" @click="adding = !adding">
        <Plus :size="14" :stroke-width="1.8" /> {{ t("todo.addTask") }}
      </button>
    </div>

    <div v-if="adding" class="todo-add">
      <input
        v-model="newTitle"
        type="text"
        :placeholder="t('todo.placeholder')"
        autofocus
        @keyup.enter="addTodo"
      />
      <select v-model="newProjectId">
        <option value="">{{ t("todo.projectPh") }}</option>
        <option v-for="p in projectStore.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
      <input v-model="newDueDate" type="date" />
      <button class="btn small primary" @click="addTodo">{{ t("common.save") }}</button>
      <button class="btn small" @click="adding = false">{{ t("common.cancel") }}</button>
    </div>

    <div v-if="visibleTodos.length" class="todo-grid" :class="{ single: mode === 'all' }">
      <div v-for="todo in visibleTodos" :key="todo.id" class="todo-row" :class="{ done: todo.done }">
        <button
          class="todo-check"
          :class="{ on: todo.done }"
          :title="todo.done ? t('todo.markUndone') : t('todo.markDone')"
          @click="toggle(todo.id, !todo.done)"
        >
          <svg v-if="todo.done" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20 6 9 17l-5-5" />
          </svg>
        </button>

        <div class="todo-main" @click="goProject(todo.projectId)">
          <div class="todo-text" :class="{ strike: todo.done }">{{ todo.title }}</div>
          <div v-if="todo.projectName" class="todo-project">
            <span class="status-dot" :class="`status-${todo.projectStatus ?? 'ARCHIVED'}`"></span>
            <Folder :size="12" :stroke-width="1.8" style="color: var(--text-tertiary)" />
            <span>{{ todo.projectName }}</span>
          </div>
        </div>

        <span v-if="todo.dueDate" class="todo-due" :class="{ overdue: !todo.done && isOverdue(todo.dueDate) }">
          <CalendarDays :size="13" :stroke-width="1.8" />
          {{ dueLabel(todo.dueDate) }}
        </span>

        <button class="todo-del" :title="t('todo.del')" @click="remove(todo.id)">
          <Trash2 :size="13" :stroke-width="1.8" />
        </button>
      </div>
    </div>
    <div v-else class="todo-empty caption">
      {{ mode === "all" ? t("todo.emptyAll") : t("todo.emptyHome") }}
    </div>
  </div>
</template>

<style scoped>
.todo-widget {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 18px 20px;
}

.todo-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}

.todo-title {
  font-size: 16px;
  font-weight: 600;
}

.todo-count {
  min-width: 22px;
  height: 20px;
  padding: 0 7px;
  border-radius: 10px;
  background: var(--surface-muted);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  font-size: 11.5px;
  display: inline-grid;
  place-items: center;
}

.todo-all {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  border: none;
  background: none;
  color: var(--text-tertiary);
  font-size: 12.5px;
  padding: 2px 4px;
  border-radius: 5px;
}

.todo-all:hover {
  color: var(--text-primary);
  background: var(--hover);
}

.spacer {
  flex: 1;
}

.todo-add {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.todo-add input[type="text"] {
  flex: 2;
  min-width: 180px;
  padding: 7px 11px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 13px;
  outline: none;
}

.todo-add input[type="text"]:focus {
  border-color: var(--border-strong);
}

.todo-add select,
.todo-add input[type="date"] {
  padding: 7px 9px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 12.5px;
  color: var(--text-secondary);
  outline: none;
}

.todo-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  column-gap: 28px;
}

.todo-grid.single {
  grid-template-columns: 1fr;
}

@media (max-width: 1000px) {
  .todo-grid {
    grid-template-columns: 1fr;
  }
}

.todo-row {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 10px 4px;
  border-bottom: 1px solid var(--border);
}

.todo-row:last-child {
  border-bottom: none;
}

.todo-check {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  border-radius: 50%;
  border: 1.6px solid var(--border-strong);
  background: transparent;
  display: inline-grid;
  place-items: center;
  padding: 0;
  transition: border-color 0.15s, background 0.15s;
}

.todo-check:hover {
  border-color: var(--status-in-progress);
}

.todo-check.on {
  background: var(--status-in-progress);
  border-color: var(--status-in-progress);
}

.todo-main {
  min-width: 0;
  cursor: pointer;
}

.todo-text {
  font-size: 13.5px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.todo-text.strike {
  text-decoration: line-through;
  color: var(--text-tertiary);
}

.todo-project {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.todo-due {
  margin-left: auto;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12.5px;
  color: var(--text-secondary);
}

.todo-due.overdue {
  color: #c0554f;
}

.todo-del {
  flex-shrink: 0;
  border: none;
  background: none;
  color: var(--text-tertiary);
  padding: 3px;
  border-radius: 5px;
  display: inline-flex;
  opacity: 0;
  transition: opacity 0.15s;
}

.todo-row:hover .todo-del {
  opacity: 1;
}

.todo-del:hover {
  color: #c0554f;
  background: var(--hover);
}

.todo-empty {
  padding: 8px 4px;
}
</style>
