import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { BarChart3, Moon, Monitor, Play, Settings, Sun } from "lucide-react";
import { TitleBar } from "@/components/TitleBar";
import { useAppStore, type ThemePreference } from "@/stores/app";
import { ipc } from "@/lib/ipc";
import { SUPPORTED_LANGUAGES } from "@/i18n";

export function App() {
  const { t } = useTranslation();
  const { ready, info, prefs, categories, load, setTheme, setLanguage } = useAppStore();

  useEffect(() => {
    load()
      .then(() => ipc.frontendReady())
      .catch((e: unknown) => console.error("failed to load app state", e));
  }, [load]);

  const category = categories[0];

  return (
    <div
      className="flex h-full flex-col bg-app"
      style={category ? ({ "--cat": category.hexColor } as React.CSSProperties) : undefined}
    >
      <TitleBar platform={info?.platform ?? "web"} />

      <main className="flex flex-1 flex-col items-center justify-center gap-8 px-6 pb-6">
        <GoalDots filled={0} total={8} />

        <h1 className="text-center text-[28px] leading-[34px] font-semibold tracking-[-0.01em]">
          {t("timer.prompt")}
        </h1>

        {category && (
          <span className="inline-flex items-center gap-2 rounded-full border border-line bg-card px-4 py-1.5 text-sm text-2">
            <span className="size-2.5 rounded-full" style={{ background: category.hexColor }} />
            {category.title}
          </span>
        )}

        <button type="button" className="btn-primary" disabled={!ready}>
          {t("timer.start")}
        </button>

        <p className="label-mono">{t("timer.comingSoon")}</p>
      </main>

      <footer className="flex items-center justify-between gap-4 px-5 pb-4">
        <span className="label-mono">
          {info ? `v${info.version} · ${info.platform}/${info.arch} · ` : ""}
          {info ? t("status.schema", { version: info.schemaVersion }) : ""}
        </span>

        <nav
          aria-label="toolbar"
          className="flex items-center gap-1 rounded-full border border-line bg-panel p-1"
        >
          <ToolbarIcon label={t("toolbar.analytics")}>
            <BarChart3 size={16} strokeWidth={1.5} />
          </ToolbarIcon>
          <ToolbarIcon label={t("toolbar.miniPlayer")}>
            <Play size={16} strokeWidth={1.5} />
          </ToolbarIcon>
          <ToolbarIcon label={t("toolbar.settings")}>
            <Settings size={16} strokeWidth={1.5} />
          </ToolbarIcon>
        </nav>

        <div className="flex items-center gap-2">
          <ThemeSwitch value={prefs.theme} onChange={(v) => void setTheme(v)} />
          <select
            aria-label={t("language.label")}
            className="h-8 rounded-full border border-line bg-card px-3 text-xs text-2"
            value={prefs.language ?? ""}
            onChange={(e) => void setLanguage(e.target.value || null)}
          >
            <option value="">{t("language.system")}</option>
            {SUPPORTED_LANGUAGES.map((l) => (
              <option key={l} value={l}>
                {t(`language.names.${l}`)}
              </option>
            ))}
          </select>
        </div>
      </footer>
    </div>
  );
}

function GoalDots({ filled, total }: { filled: number; total: number }) {
  return (
    <div className="flex gap-2" aria-hidden>
      {Array.from({ length: total }, (_, i) => (
        <span
          key={i}
          className="size-2 rounded-full"
          style={{ background: i < filled ? "var(--cat)" : "var(--bg-hover)" }}
        />
      ))}
    </div>
  );
}

function ToolbarIcon({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      className="flex size-8 items-center justify-center rounded-full text-2 transition-colors hover:bg-hover hover:text-1"
    >
      {children}
    </button>
  );
}

const THEMES: { value: ThemePreference; icon: typeof Sun }[] = [
  { value: "dark", icon: Moon },
  { value: "light", icon: Sun },
  { value: "system", icon: Monitor },
];

function ThemeSwitch({
  value,
  onChange,
}: {
  value: ThemePreference;
  onChange: (v: ThemePreference) => void;
}) {
  const { t } = useTranslation();
  return (
    <div
      role="radiogroup"
      aria-label={t("appearance.theme")}
      className="flex rounded-full border border-line bg-card p-0.5"
    >
      {THEMES.map(({ value: v, icon: Icon }) => (
        <button
          key={v}
          type="button"
          role="radio"
          aria-checked={value === v}
          aria-label={t(`appearance.themes.${v}`)}
          title={t(`appearance.themes.${v}`)}
          onClick={() => onChange(v)}
          className={`flex size-7 items-center justify-center rounded-full transition-colors ${
            value === v ? "bg-hover text-1" : "text-3 hover:text-1"
          }`}
        >
          <Icon size={14} strokeWidth={1.5} />
        </button>
      ))}
    </div>
  );
}
