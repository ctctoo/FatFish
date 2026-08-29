import { defineStore } from "pinia";
import { ref, watch } from "vue";

type Theme = "system" | "light" | "dark";
type ViewMode = "grid" | "list";
type SortKey = "updated" | "name" | "opened";
export type Locale = "zh" | "en";

const KEY = "fatfish.settings";
const LEGACY_KEY = "project-hub.settings";

interface Persisted {
  theme: Theme;
  viewMode: ViewMode;
  sort: SortKey;
  defaultFolder: string;
  scanDirs: string[];
  confirmRemove: boolean;
  locale: Locale;
}

function load(): Persisted {
  const fallback: Persisted = {
    theme: "system",
    viewMode: "grid",
    sort: "updated",
    defaultFolder: "",
    scanDirs: [],
    confirmRemove: true,
    locale: "zh",
  };
  try {
    const raw = localStorage.getItem(KEY) ?? localStorage.getItem(LEGACY_KEY);
    return raw ? { ...fallback, ...(JSON.parse(raw) as Persisted) } : fallback;
  } catch {
    return fallback;
  }
}

export const useSettingsStore = defineStore("settings", () => {
  const initial = load();

  const theme = ref<Theme>(initial.theme);
  const viewMode = ref<ViewMode>(initial.viewMode);
  const sort = ref<SortKey>(initial.sort);
  const defaultFolder = ref(initial.defaultFolder);
  const scanDirs = ref<string[]>(initial.scanDirs);
  const confirmRemove = ref(initial.confirmRemove);
  const locale = ref<Locale>(initial.locale);

  watch(
    [theme, viewMode, sort, defaultFolder, scanDirs, confirmRemove, locale],
    () => {
      localStorage.setItem(
        KEY,
        JSON.stringify({
          theme: theme.value,
          viewMode: viewMode.value,
          sort: sort.value,
          defaultFolder: defaultFolder.value,
          scanDirs: scanDirs.value,
          confirmRemove: confirmRemove.value,
          locale: locale.value,
        } satisfies Persisted)
      );
    },
    { deep: true }
  );

  function addScanDir(dir: string) {
    if (!dir || scanDirs.value.includes(dir)) return;
    scanDirs.value = [dir, ...scanDirs.value].slice(0, 5);
  }

  return { theme, viewMode, sort, defaultFolder, scanDirs, confirmRemove, locale, addScanDir };
});
