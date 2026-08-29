import { defineStore } from "pinia";
import { ref } from "vue";
import { tauriApi } from "../services/tauri";
import type { Todo, TodoInput } from "../types";

export const useTodoStore = defineStore("todo", () => {
  const todos = ref<Todo[]>([]);
  const loaded = ref(false);

  async function fetchTodos() {
    todos.value = await tauriApi.listTodos();
    loaded.value = true;
  }

  async function createTodo(input: TodoInput) {
    await tauriApi.createTodo(input);
    await fetchTodos();
  }

  async function updateTodo(id: string, input: TodoInput) {
    await tauriApi.updateTodo(id, input);
    await fetchTodos();
  }

  async function toggleTodo(id: string, done: boolean) {
    await tauriApi.toggleTodo(id, done);
    await fetchTodos();
  }

  async function deleteTodo(id: string) {
    await tauriApi.deleteTodo(id);
    await fetchTodos();
  }

  return { todos, loaded, fetchTodos, createTodo, updateTodo, toggleTodo, deleteTodo };
});
