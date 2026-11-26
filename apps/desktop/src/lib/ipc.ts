/**
 * Thin wrapper over the generated bindings so the UI also runs in a plain browser
 * (Vitest, Playwright screenshots) with an in-memory fallback. Command errors reject.
 */
import { isTauri } from "@tauri-apps/api/core";
import {
  commands,
  type AppInfo,
  type CategoryDto,
  type Preferences_Serialize as Preferences,
} from "@/bindings";

export type { AppInfo, CategoryDto, Preferences };

const browserFallback = {
  prefs: { theme: "dark", language: null } as Preferences,
  info: {
    version: "0.0.0-web",
    platform: "web",
    arch: "web",
    schemaVersion: 1,
    vibrancy: false,
  } as AppInfo,
  categories: [
    { id: "focus", title: "Focus", hexColor: "#E5484D", archived: false, position: 0 },
  ] as CategoryDto[],
};

export const inTauri = isTauri();

export const ipc = {
  appInfo: (): Promise<AppInfo> =>
    inTauri ? commands.getAppInfo() : Promise.resolve(browserFallback.info),
  preferences: (): Promise<Preferences> =>
    inTauri ? commands.getPreferences() : Promise.resolve(browserFallback.prefs),
  setPreferences: (prefs: Preferences): Promise<Preferences> =>
    inTauri ? commands.setPreferences(prefs) : Promise.resolve((browserFallback.prefs = prefs)),
  categories: (): Promise<CategoryDto[]> =>
    inTauri ? commands.listCategories() : Promise.resolve(browserFallback.categories),
  frontendReady: (): Promise<void> => (inTauri ? commands.frontendReady() : Promise.resolve()),
};
