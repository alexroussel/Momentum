import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import { inTauri } from "@/lib/ipc";

interface Props {
  platform: string;
}

/**
 * Custom title bar (ADR-011). macOS keeps native traffic lights (overlay title bar), so only a
 * drag region with left padding is rendered; Windows/Linux get our own controls on the right.
 */
export function TitleBar({ platform }: Props) {
  const isMac = platform === "macos";
  return (
    <header
      data-tauri-drag-region
      className="relative z-10 flex h-10 shrink-0 items-center"
      style={{ paddingLeft: isMac ? 78 : 14 }}
    >
      <div data-tauri-drag-region className="flex-1" />
      {!isMac && inTauri && <WindowControls />}
    </header>
  );
}

function WindowControls() {
  const { t } = useTranslation();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const win = getCurrentWindow();
    let unlisten: (() => void) | undefined;
    void win.isMaximized().then(setMaximized);
    void win
      .onResized(async () => setMaximized(await win.isMaximized()))
      .then((fn) => (unlisten = fn));
    return () => unlisten?.();
  }, []);

  const win = getCurrentWindow();
  const btn =
    "flex h-10 w-11 items-center justify-center text-2 transition-colors duration-[var(--dur-hover)] hover:bg-hover hover:text-1";

  return (
    <div className="flex h-full">
      <button
        type="button"
        className={btn}
        aria-label={t("window.minimize")}
        onClick={() => void win.minimize()}
      >
        <Minus size={15} strokeWidth={1.5} />
      </button>
      <button
        type="button"
        className={btn}
        aria-label={t(maximized ? "window.restore" : "window.maximize")}
        onClick={() => void win.toggleMaximize()}
      >
        {maximized ? <Copy size={12} strokeWidth={1.5} /> : <Square size={12} strokeWidth={1.5} />}
      </button>
      <button
        type="button"
        className={`${btn} hover:!bg-[#C42B1C] hover:!text-white`}
        aria-label={t("window.close")}
        onClick={() => void win.close()}
      >
        <X size={16} strokeWidth={1.5} />
      </button>
    </div>
  );
}
