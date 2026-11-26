import { create } from "zustand";
import i18n, { resolveLanguage } from "@/i18n";
import { ipc, type AppInfo, type CategoryDto, type Preferences } from "@/lib/ipc";

export type ThemePreference = Preferences["theme"];

interface AppStore {
  ready: boolean;
  info: AppInfo | null;
  prefs: Preferences;
  categories: CategoryDto[];
  load: () => Promise<void>;
  setTheme: (theme: ThemePreference) => Promise<void>;
  setLanguage: (language: string | null) => Promise<void>;
}

const systemDark = () => window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? true;

/** Applies theme, vibrancy and language to the document. */
export function applyDocumentPreferences(prefs: Preferences, info: AppInfo | null) {
  const root = document.documentElement;
  const theme = prefs.theme === "system" ? (systemDark() ? "dark" : "light") : prefs.theme;
  root.dataset.theme = theme;
  root.dataset.vibrancy = info?.vibrancy ? "on" : "off";
  root.dataset.platform = info?.platform ?? "web";
  const lang = resolveLanguage(prefs.language);
  root.lang = lang;
  if (i18n.language !== lang) void i18n.changeLanguage(lang);
}

export const useAppStore = create<AppStore>((set, get) => ({
  ready: false,
  info: null,
  prefs: { theme: "dark", language: null },
  categories: [],

  async load() {
    const [info, prefs, categories] = await Promise.all([
      ipc.appInfo(),
      ipc.preferences(),
      ipc.categories(),
    ]);
    applyDocumentPreferences(prefs, info);
    set({ info, prefs, categories, ready: true });
  },

  async setTheme(theme) {
    const prefs = await ipc.setPreferences({ ...get().prefs, theme });
    applyDocumentPreferences(prefs, get().info);
    set({ prefs });
  },

  async setLanguage(language) {
    const prefs = await ipc.setPreferences({ ...get().prefs, language });
    applyDocumentPreferences(prefs, get().info);
    set({ prefs });
  },
}));

// Follow OS theme changes live when the preference is "system".
window.matchMedia?.("(prefers-color-scheme: dark)").addEventListener?.("change", () => {
  const { prefs, info } = useAppStore.getState();
  if (prefs.theme === "system") applyDocumentPreferences(prefs, info);
});
