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
