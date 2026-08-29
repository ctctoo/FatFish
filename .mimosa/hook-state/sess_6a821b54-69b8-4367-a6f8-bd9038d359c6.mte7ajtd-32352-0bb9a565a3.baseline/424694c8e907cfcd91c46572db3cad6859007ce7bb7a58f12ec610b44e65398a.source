import { watch } from "vue";
import { useSettingsStore } from "../stores/settings";

const media = window.matchMedia("(prefers-color-scheme: dark)");

function apply(theme: string) {
  const dark = theme === "dark" || (theme === "system" && media.matches);
  document.documentElement.setAttribute("data-theme", dark ? "dark" : "light");
}

export function useTheme() {
  const settings = useSettingsStore();

  watch(
    () => settings.theme,
    (t) => {
      apply(t);
      if (t === "system") {
        media.addEventListener("change", onSystemChange);
      } else {
        media.removeEventListener("change", onSystemChange);
      }
    },
    { immediate: true }
  );
}

function onSystemChange() {
  apply("system");
}
