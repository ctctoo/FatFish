import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";
import { useTheme } from "./composables/useTheme";
import "./styles/tokens.css";
import "./styles/global.css";
import "./styles/components.css";

const app = createApp(App);
app.use(createPinia());
useTheme();
app.use(router);
app.mount("#app");

// ---- 屏蔽浏览器交互：让 WebView 表现为原生应用 ----
// 右键菜单 / 拖拽文件导航
window.addEventListener("contextmenu", (e) => e.preventDefault());
window.addEventListener("dragover", (e) => e.preventDefault());
window.addEventListener("drop", (e) => e.preventDefault());

// 刷新 / 开发者工具 / 缩放快捷键
window.addEventListener(
  "keydown",
  (e) => {
    const key = e.key.toLowerCase();
    const ctrl = e.ctrlKey || e.metaKey;
    const blocked =
      e.key === "F5" ||
      e.key === "F12" ||
      (ctrl && ["r", "+", "-", "=", "_"].includes(key)) ||
      (ctrl && e.shiftKey && ["i", "j", "c", "r"].includes(key));
    if (blocked) e.preventDefault();
  },
  true
);

// Ctrl + 滚轮缩放
window.addEventListener(
  "wheel",
  (e) => {
    if (e.ctrlKey) e.preventDefault();
  },
  { passive: false }
);
