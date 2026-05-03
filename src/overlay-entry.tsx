import { useEffect } from "react";
import ReactDOM from "react-dom/client";
import { useTranslation } from "react-i18next";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useStore, useStateSync } from "./store";
import type { User } from "./types";
import { applyTheme } from "./theme-utils";
import { disableContextMenu } from "./disableContextMenu";
import "./i18n";
import "./themes.css";
import "../overlay/overlay.css";

disableContextMenu();
document.body.classList.add("tauri-overlay-window");

function OverlayRow({ user, kind }: { user: User; kind: "playing" | "waiting" }) {
  const { t } = useTranslation();

  return (
    <div className={`row ${kind}`}>
      <span className="name">{user.displayName}</span>
      {user.firstTimeToday && <span className="badge">{t("overlay.firstTimeBadge")}</span>}
    </div>
  );
}

function OverlayDragGrip() {
  const startDrag = (e: React.PointerEvent<HTMLButtonElement>) => {
    if (e.button !== 0) return;
    getCurrentWindow().startDragging().catch((err) => console.error("overlay drag failed", err));
  };

  return (
    <button
      className="drag-grip"
      type="button"
      title="Move overlay"
      aria-label="Move overlay"
      onPointerDown={startDrag}
    >
      <div className="grip-icon">
        <span />
        <span />
        <span />
        <span />
        <span />
        <span />
      </div>
    </button>
  );
}

function Overlay() {
  const { t, i18n } = useTranslation();
  const snap = useStore((s) => s.snap);
  const config = useStore((s) => s.config);
  const hydrate = useStore((s) => s.hydrate);
  useStateSync();

  useEffect(() => { hydrate(); }, [hydrate]);

  // Listen for theme preview events from ThemeModal
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<string>("theme-preview", (e) => {
      applyTheme(e.payload as any);
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  // Apply saved theme from snapshot when it changes
  useEffect(() => {
    if (snap?.theme) {
      applyTheme(snap.theme, config?.customColors);
    }
  }, [snap?.theme, config?.customColors]);

  useEffect(() => {
    if (config?.language && i18n.language !== config.language) {
      i18n.changeLanguage(config.language);
    }
  }, [config?.language, i18n]);

  // Auto-fit window height to rendered rows. Width stays locked by Tauri
  // maxWidth, but height should shrink again when the visible row count drops.
  useEffect(() => {
    const root = document.getElementById("root");
    if (!root) return;
    const win = getCurrentWindow();
    let raf = 0;
    let lastHeight = 0;
    const apply = () => {
      const measured = root.scrollHeight;
      const h = Math.max(60, Math.ceil(measured) + 8);
      if (h === lastHeight) return;
      lastHeight = h;
      win.setSize(new LogicalSize(340, h)).catch((e) => {
        console.error("overlay setSize failed", e);
      });
    };
    const schedule = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => requestAnimationFrame(apply));
    };
    const observer = new ResizeObserver(schedule);
    observer.observe(root);
    schedule();
    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
    };
  }, []);

  if (!snap) return null;

  const visibleWaiting = snap.waiting.slice(0, snap.maxWaiting);
  const hidden = snap.waitingTotal - visibleWaiting.length;

  return (
    <>
      <OverlayDragGrip />
      {snap.playing.map((u) => <OverlayRow key={u.id} user={u} kind="playing" />)}
      {visibleWaiting.map((u) => <OverlayRow key={u.id} user={u} kind="waiting" />)}
      {hidden > 0 && <div className="more">{t("overlay.moreWaiting", { n: hidden })}</div>}
    </>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(<Overlay />);
